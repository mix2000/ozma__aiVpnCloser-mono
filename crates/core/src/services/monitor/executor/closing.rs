//! Закрытие агентов: TERM, добивание KILL, повтор неудавшегося.

use std::collections::HashSet;

use super::super::messages as msg;
use super::super::tracked::Tracked;
use super::super::TickOutput;
use super::Executor;
use crate::models::{Action, ActionReport, AgentProc, Config, EventKind, ProcInfo};
use crate::services::agents::summary;

impl Executor {
    /// Агенты закрыты, VPN нет: закрыть тех, кто запустился только что.
    pub fn close_new(
        &mut self,
        cfg: &Config,
        now: u64,
        agents_now: &[AgentProc],
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        let pending: HashSet<u32> = self.pending_kill.iter().map(|(t, _)| t.pid).collect();
        let fresh: Vec<AgentProc> = agents_now
            .iter()
            .filter(|a| !pending.contains(&a.pid))
            .filter(|a| !self.handled.contains(&(a.pid, a.start_time)))
            .cloned()
            .collect();
        if !fresh.is_empty() {
            self.close(cfg, now, &fresh, procs, out, true);
        }
    }

    /// Застрявшие замороженными, которых ещё не закрывали и нет среди `known`.
    fn stuck_besides(&self, known: &[AgentProc], procs: &[ProcInfo]) -> Vec<AgentProc> {
        self.record
            .leftover()
            .iter()
            .filter(|f| !self.handled.contains(&(f.pid, f.start_time)))
            .filter(|f| !known.iter().any(|a| a.pid == f.pid))
            .filter_map(|f| Tracked::helper(f, procs))
            .collect()
    }

    /// Закрыть список + всех замороженных (дети раньше родителей).
    pub fn close(
        &mut self,
        cfg: &Config,
        now: u64,
        list: &[AgentProc],
        procs: &[ProcInfo],
        out: &mut TickOutput,
        block: bool,
    ) {
        let mut all: Vec<AgentProc> = list.to_vec();
        let frozen: Vec<Tracked> = std::mem::take(&mut self.frozen).into_values().collect();
        for t in &frozen {
            if t.alive_in(procs) && !all.iter().any(|a| a.pid == t.pid) {
                all.push(t.as_agent());
            }
        }
        all.extend(self.stuck_besides(&all, procs));
        if all.is_empty() {
            if !block {
                self.event(out, EventKind::Info, msg::NOTHING_TO_CLOSE.into());
            }
            return;
        }
        all.sort_by(|a, b| b.depth.cmp(&a.depth).then(a.pid.cmp(&b.pid)));
        for a in &all {
            self.handled.insert((a.pid, a.start_time));
        }

        if cfg.dry_run {
            // Если что-то было заморожено по-настоящему до включения
            // «только смотреть» — не оставляем его замороженным навсегда.
            let real = Self::real_alive(&frozen, procs);
            self.resume(out, &real);
            let text = msg::close(&summary(&all), block, true);
            self.event(out, EventKind::DryRun, text);
            return;
        }

        let failed = self.terminate(cfg, now, &all, procs, out);
        // Замороженные, которых закрыть не удалось, остаются замороженными:
        // не забываем о них.
        for f in Self::real_alive(&frozen, procs) {
            if failed.contains(&f.pid) {
                self.record.hold(f);
            }
        }
        all.retain(|a| !failed.contains(&a.pid));
        self.report_closed(cfg, &all, failed.len(), block, out);
    }

    /// Журнал и уведомление: кого закрыли и скольких не удалось.
    fn report_closed(
        &self,
        cfg: &Config,
        closed: &[AgentProc],
        failed: usize,
        block: bool,
        out: &mut TickOutput,
    ) {
        let text = (!closed.is_empty()).then(|| msg::close(&summary(closed), block, false));
        let (kind, title) = if block {
            (EventKind::Block, msg::TITLE_BLOCKED)
        } else {
            (EventKind::Close, msg::TITLE_CLOSED)
        };
        if let Some(text) = &text {
            self.event(out, kind, text.clone());
        }
        match text {
            Some(text) if failed == 0 => Self::notice(cfg, out, title, text),
            text => {
                let body = msg::not_closed_notice(text.as_deref(), failed);
                Self::notice(cfg, out, msg::TITLE_NOT_CLOSED, body);
            }
        }
    }

    /// TERM всем (дети раньше родителей), KILL — через `killGraceSec`. Вернёт,
    /// кого закрыть не удалось: повторит [`Executor::retry_close`].
    fn terminate(
        &mut self,
        cfg: &Config,
        now: u64,
        all: &[AgentProc],
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) -> Vec<u32> {
        let pids: Vec<u32> = all.iter().map(|a| a.pid).collect();
        let rep = self.act(out, Action::Terminate, &pids);
        self.after_terminate(cfg, now, all, &rep, procs, out);
        rep.failed.iter().map(|(pid, _)| *pid).collect()
    }

    fn after_terminate(
        &mut self,
        cfg: &Config,
        now: u64,
        all: &[AgentProc],
        rep: &ActionReport,
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        let deadline = now + u64::from(cfg.kill_grace_sec) * 1000;
        for a in all {
            let t = Tracked::from(a, true);
            if rep.ok.contains(&a.pid) {
                self.pending_kill.push((t, deadline));
                self.record.forget(a.pid);
            } else if rep.failed.iter().any(|(pid, _)| *pid == a.pid) {
                self.close_failed.push(t);
            }
        }
        if cfg.kill_grace_sec == 0 {
            self.run_pending_kills(now, procs, out);
        }
    }

    /// Агенты закрыты, VPN нет: ещё раз закрыть тех, кого не удалось. Об ошибке
    /// уже сообщили — повторно не пишем.
    pub fn retry_close(
        &mut self,
        cfg: &Config,
        now: u64,
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        if cfg.dry_run {
            return;
        }
        let mut list: Vec<AgentProc> = std::mem::take(&mut self.close_failed)
            .iter()
            .filter(|t| t.alive_in(procs))
            .map(Tracked::as_agent)
            .collect();
        if list.is_empty() {
            return;
        }
        let pids: Vec<u32> = list.iter().map(|a| a.pid).collect();
        let rep = self.actor.apply(Action::Terminate, &pids);
        self.after_terminate(cfg, now, &list, &rep, procs, out);
        list.retain(|a| rep.ok.contains(&a.pid));
        if !list.is_empty() {
            let text = msg::close(&summary(&list), false, false);
            self.event(out, EventKind::Close, text);
        }
    }

    /// Добить KILL тех, кто после TERM не закрылся за `killGraceSec`.
    pub fn run_pending_kills(&mut self, now: u64, procs: &[ProcInfo], out: &mut TickOutput) {
        if self.pending_kill.is_empty() {
            return;
        }
        let (due, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending_kill)
            .into_iter()
            .partition(|(_, d)| now >= *d);
        self.pending_kill = rest;
        let pids: Vec<u32> = due
            .iter()
            .filter(|(t, _)| t.alive_in(procs))
            .map(|(t, _)| t.pid)
            .collect();
        if pids.is_empty() {
            return;
        }
        let rep = self.act(out, Action::Kill, &pids);
        if !rep.ok.is_empty() {
            self.event(out, EventKind::Close, msg::killed_rest(rep.ok.len()));
        }
    }
}
