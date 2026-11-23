//! Применение действий по переходам автомата: заморозить, разморозить,
//! закрыть, добить. Помнит, кого заморозил и кого уже закрыл, и пишет события
//! и уведомления в [`TickOutput`].

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use super::clock::Clock;
use super::frozen_record::FrozenRecord;
use super::messages as msg;
use super::tracked::Tracked;
use super::TickOutput;
use crate::models::{
    Action, ActionReport, AgentProc, CheckResult, Config, Event, EventKind, FrozenProc, Notice,
    ProcInfo,
};
use crate::services::actions::Actor;
use crate::services::agents::summary;
use crate::storage::FrozenStore;

pub struct Executor {
    actor: Box<dyn Actor>,
    clock: Arc<dyn Clock>,
    frozen: BTreeMap<u32, Tracked>,
    record: FrozenRecord,
    /// Уже закрытые (или «закрыл бы» в режиме «только смотреть») за этот
    /// период без VPN (pid, start_time) — повторно не трогаем и не сообщаем.
    handled: HashSet<(u32, u64)>,
    /// Кому послан TERM, и когда добить KILL.
    pending_kill: Vec<(Tracked, u64)>,
}

impl Executor {
    pub fn new(
        actor: Box<dyn Actor>,
        clock: Arc<dyn Clock>,
        store: Box<dyn FrozenStore>,
    ) -> Executor {
        Executor {
            actor,
            clock,
            frozen: BTreeMap::new(),
            record: FrozenRecord::open(store),
            handled: HashSet::new(),
            pending_kill: Vec::new(),
        }
    }

    // ------------------------------------------------------------ вывод

    fn event(&self, out: &mut TickOutput, kind: EventKind, message: String) {
        out.events.push(Event {
            at: self.clock.stamp(),
            kind,
            message,
        });
    }

    fn notice(cfg: &Config, out: &mut TickOutput, title: &str, body: String) {
        if cfg.notify {
            out.notices.push(Notice {
                title: title.to_string(),
                body,
            });
        }
    }

    fn act(&mut self, out: &mut TickOutput, action: Action, pids: &[u32]) -> ActionReport {
        if pids.is_empty() {
            return ActionReport::default();
        }
        let rep = self.actor.apply(action, pids);
        for (pid, err) in &rep.failed {
            self.event(out, EventKind::Error, msg::action_failed(action, *pid, err));
        }
        rep
    }

    /// pid настоящих замороженных из `tracked`, которые ещё живы.
    fn real_alive(tracked: &[Tracked], procs: &[ProcInfo]) -> Vec<u32> {
        tracked
            .iter()
            .filter(|t| t.real && t.alive_in(procs))
            .map(|t| t.pid)
            .collect()
    }

    // ------------------------------------------------------------ состояние

    fn is_frozen(&self, a: &AgentProc) -> bool {
        self.frozen
            .get(&a.pid)
            .is_some_and(|t| t.start_time == a.start_time)
    }

    /// Заморожен по-настоящему (для `AgentInfo.frozen`), в том числе прошлым запуском.
    pub fn is_really_frozen(&self, a: &AgentProc) -> bool {
        self.record.is_leftover(a)
            || self
                .frozen
                .get(&a.pid)
                .is_some_and(|t| t.real && t.start_time == a.start_time)
    }

    /// Остались ли замороженные прошлым запуском (программа упала или её убили).
    pub fn has_leftover(&self) -> bool {
        self.record.has_leftover()
    }

    /// Записать на диск, кто сейчас заморожен по-настоящему, и ещё `adding`.
    pub fn sync_record(&mut self, adding: &[AgentProc], out: &mut TickOutput) {
        let adding = adding.iter().map(|a| Tracked::from(a, true));
        let real: Vec<FrozenProc> = self
            .frozen
            .values()
            .filter(|t| t.real)
            .cloned()
            .chain(adding)
            .map(|t| t.as_frozen())
            .collect();
        if let Err(e) = self.record.sync(real.into_iter()) {
            self.event(out, EventKind::Error, msg::frozen_not_saved(&e));
        }
    }

    /// Новый период: о закрытых ранее можно забыть.
    pub fn forget_handled(&mut self) {
        self.handled.clear();
    }

    // ------------------------------------------------------------ переходы

    /// VPN пропал (подтверждено): заморозить и начать отсчёт или закрыть сразу.
    #[allow(clippy::too_many_arguments)]
    pub fn enter_down(
        &mut self,
        cfg: &Config,
        now: u64,
        checks: &[CheckResult],
        agents_now: &[AgentProc],
        procs: &[ProcInfo],
        close_now: bool,
        out: &mut TickOutput,
    ) {
        self.event(out, EventKind::VpnDown, msg::vpn_down(checks));
        if cfg.freeze_immediately && !agents_now.is_empty() {
            self.freeze(cfg, agents_now, out, false);
        }
        if close_now {
            self.close(cfg, now, agents_now, procs, out, false);
            return;
        }
        let body = msg::countdown_notice(
            cfg.close_after_sec,
            agents_now.is_empty(),
            cfg.freeze_immediately,
            cfg.dry_run,
        );
        Self::notice(cfg, out, msg::TITLE_VPN_DOWN, body);
    }

    /// Заморозить ещё не замороженных (родители раньше детей). `block` — это
    /// новые процессы, запущенные, пока VPN нет.
    fn freeze(&mut self, cfg: &Config, list: &[AgentProc], out: &mut TickOutput, block: bool) {
        let fresh: Vec<AgentProc> = list
            .iter()
            .filter(|a| !self.is_frozen(a))
            .cloned()
            .collect();
        if fresh.is_empty() {
            return;
        }
        let what = summary(&fresh);
        if cfg.dry_run {
            for a in &fresh {
                self.frozen.insert(a.pid, Tracked::from(a, false));
            }
            self.event(out, EventKind::DryRun, msg::freeze(&what, block, true));
            return;
        }
        let pids: Vec<u32> = fresh.iter().map(|a| a.pid).collect();
        // Сначала запись, потом заморозка: упав посередине, никого не забудем.
        self.sync_record(&fresh, out);
        let rep = self.act(out, Action::Freeze, &pids);
        for a in &fresh {
            if rep.ok.contains(&a.pid) {
                self.frozen.insert(a.pid, Tracked::from(a, true));
            }
        }
        let kind = if block {
            EventKind::Block
        } else {
            EventKind::Freeze
        };
        self.event(out, kind, msg::freeze(&what, block, false));
    }

    /// Идёт отсчёт: заморозить тех, кто запустился только что.
    pub fn freeze_new(&mut self, cfg: &Config, agents_now: &[AgentProc], out: &mut TickOutput) {
        let fresh: Vec<AgentProc> = agents_now
            .iter()
            .filter(|a| !self.is_frozen(a))
            .cloned()
            .collect();
        if !fresh.is_empty() {
            self.freeze(cfg, &fresh, out, true);
        }
    }

    /// VPN вернулся во время отсчёта: разморозить.
    pub fn vpn_back(&mut self, cfg: &Config, procs: &[ProcInfo], out: &mut TickOutput) {
        let tracked: Vec<Tracked> = std::mem::take(&mut self.frozen).into_values().collect();
        let real = Self::real_alive(&tracked, procs);
        self.act(out, Action::Resume, &real);
        let text = if real.is_empty() {
            let roots: Vec<AgentProc> = tracked.iter().map(Tracked::as_agent).collect();
            let would = (!tracked.is_empty()).then(|| summary(&roots));
            msg::vpn_back(None, would.as_deref())
        } else {
            let resumed: Vec<AgentProc> = tracked
                .iter()
                .filter(|t| real.contains(&t.pid))
                .map(Tracked::as_agent)
                .collect();
            msg::vpn_back(Some(&summary(&resumed)), None)
        };
        let kind = if real.is_empty() {
            EventKind::VpnUp
        } else {
            EventKind::Resume
        };
        self.event(out, kind, text);
        Self::notice(
            cfg,
            out,
            msg::TITLE_VPN_BACK,
            msg::BODY_BACK_IN_COUNTDOWN.into(),
        );
        self.forget_handled();
    }

    /// VPN вернулся после закрытия.
    pub fn back_after_close(&mut self, cfg: &Config, out: &mut TickOutput) {
        self.forget_handled();
        self.event(out, EventKind::VpnUp, msg::VPN_BACK_AFTER_CLOSE.into());
        Self::notice(
            cfg,
            out,
            msg::TITLE_VPN_BACK,
            msg::BODY_BACK_AFTER_CLOSE.into(),
        );
    }

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
        if all.is_empty() {
            if !block {
                self.event(out, EventKind::Info, msg::NOTHING_TO_CLOSE.into());
            }
            return;
        }
        // Корни для подсчёта берём из исходного списка.
        let what = summary(&all);
        all.sort_by(|a, b| b.depth.cmp(&a.depth).then(a.pid.cmp(&b.pid)));
        for a in &all {
            self.handled.insert((a.pid, a.start_time));
        }

        if cfg.dry_run {
            // Если что-то было заморожено по-настоящему до включения
            // «только смотреть» — не оставляем его замороженным навсегда.
            let real = Self::real_alive(&frozen, procs);
            self.act(out, Action::Resume, &real);
            self.event(out, EventKind::DryRun, msg::close(&what, block, true));
            return;
        }

        self.terminate(cfg, now, &all, procs, out);
        let text = msg::close(&what, block, false);
        let (kind, title) = if block {
            (EventKind::Block, msg::TITLE_BLOCKED)
        } else {
            (EventKind::Close, msg::TITLE_CLOSED)
        };
        self.event(out, kind, text.clone());
        Self::notice(cfg, out, title, text);
    }

    /// TERM всем (дети раньше родителей), KILL — через `killGraceSec`.
    fn terminate(
        &mut self,
        cfg: &Config,
        now: u64,
        all: &[AgentProc],
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        let pids: Vec<u32> = all.iter().map(|a| a.pid).collect();
        let rep = self.act(out, Action::Terminate, &pids);
        let deadline = now + u64::from(cfg.kill_grace_sec) * 1000;
        for a in all {
            if rep.ok.contains(&a.pid) {
                self.pending_kill.push((Tracked::from(a, true), deadline));
            }
        }
        if cfg.kill_grace_sec == 0 {
            self.run_pending_kills(now, procs, out);
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

    /// Разморозить оставшихся с прошлого запуска. Дальше — обычный путь: если
    /// VPN нет, их тут же заморозит переход автомата.
    pub fn release_leftover(&mut self, procs: &[ProcInfo], out: &mut TickOutput) {
        let pids: Vec<u32> = self
            .record
            .take_leftover()
            .iter()
            .filter(|f| f.alive_in(procs))
            .map(|f| f.pid)
            .collect();
        if pids.is_empty() {
            return;
        }
        let rep = self.act(out, Action::Resume, &pids);
        let text = msg::released(msg::WHY_LEFTOVER, rep.ok.len());
        self.event(out, EventKind::Resume, text);
    }

    /// Разморозить всё, что заморожено по-настоящему (выключение наблюдения,
    /// выход из программы).
    pub fn release_all(&mut self, procs: &[ProcInfo], out: &mut TickOutput, why: &str) {
        if self.frozen.is_empty() {
            return;
        }
        let tracked: Vec<Tracked> = std::mem::take(&mut self.frozen).into_values().collect();
        let real = Self::real_alive(&tracked, procs);
        if real.is_empty() {
            return;
        }
        let rep = self.act(out, Action::Resume, &real);
        self.event(out, EventKind::Resume, msg::released(why, rep.ok.len()));
    }
}
