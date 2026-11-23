//! Монитор: один шаг опроса = снимок процессов → проверки → автомат → действия
//! → состояние для UI.
//!
//! - `machine` — чистый автомат фаз (ok → countdown → closed);
//! - `executor` — применение действий по переходам;
//! - `frozen_record` — список замороженных на диске (на случай падения);
//! - `messages` — тексты журнала и уведомлений;
//! - `clock` — часы (в тестах — ручные).
//!
//! Сценарии монитора — в `crates/core/tests/monitor_*.rs`.
//!
//! Все зависимости внедряются ([`MonitorParts`]), поэтому монитор проверяется
//! без сети и без сигналов. Поток опроса — в `engine`.

mod clock;
mod executor;
mod frozen_record;
mod machine;
mod messages;
mod status_view;
mod tracked;

use std::sync::Arc;

#[cfg(any(test, feature = "test-util"))]
pub use clock::FakeClock;
pub use clock::{Clock, SystemClock};
pub use machine::{Machine, Transition};
pub use status_view::status_view;

use crate::models::{
    AgentInfo, AgentProc, AppState, CheckResult, Config, Event, Notice, ProcInfo, SelfCtx, Status,
    Verdict,
};
use crate::platform::ProcSource;
use crate::services::actions::Actor;
use crate::services::agents;
use crate::services::checks::{combine, CheckSource};
use crate::storage::FrozenStore;
use executor::Executor;

/// Сколько держать оставшихся с прошлого запуска, если итог проверок так и
/// неизвестен: без предела они висели бы вечно.
const LEFTOVER_HOLD_MS: u64 = 30_000;

/// Кто «мы»: себя и своих потомков не трогаем.
#[derive(Debug, Clone)]
pub enum Me {
    /// Процесс с этим pid; пользователь — из снимка на каждом шаге.
    Pid(u32),
    /// Заданный явно (тесты).
    Fixed(SelfCtx),
}

/// Зависимости монитора.
pub struct MonitorParts {
    pub checks: Box<dyn CheckSource>,
    pub procs: Box<dyn ProcSource>,
    pub actor: Box<dyn Actor>,
    pub clock: Arc<dyn Clock>,
    pub frozen: Box<dyn FrozenStore>,
    pub me: Me,
}

/// Что получилось за один шаг.
#[derive(Debug, Clone, Default)]
pub struct TickOutput {
    pub state: AppState,
    pub events: Vec<Event>,
    pub notices: Vec<Notice>,
}

pub struct Monitor {
    cfg: Config,
    checks: Box<dyn CheckSource>,
    procs: Box<dyn ProcSource>,
    clock: Arc<dyn Clock>,
    me: Me,
    machine: Machine,
    executor: Executor,
    /// С какого момента держим оставшихся с прошлого запуска.
    hold_since: Option<u64>,
    status: Option<Status>,
    last_change_at: Option<String>,
    state: AppState,
}

impl Monitor {
    pub fn new(cfg: Config, parts: MonitorParts) -> Monitor {
        let state = AppState::initial(cfg.monitoring, cfg.dry_run);
        Monitor {
            cfg,
            checks: parts.checks,
            procs: parts.procs,
            executor: Executor::new(parts.actor, parts.clock.clone(), parts.frozen),
            clock: parts.clock,
            me: parts.me,
            machine: Machine::default(),
            hold_since: None,
            status: None,
            last_change_at: None,
            state,
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn set_config(&mut self, cfg: Config) {
        self.cfg = cfg;
    }

    #[cfg(any(test, feature = "test-util"))]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    fn me(&self, procs: &[ProcInfo]) -> SelfCtx {
        match &self.me {
            Me::Pid(pid) => SelfCtx::for_pid(*pid, procs),
            Me::Fixed(ctx) => ctx.clone(),
        }
    }

    /// Один шаг опроса.
    pub fn tick(&mut self) -> TickOutput {
        let mut out = TickOutput::default();
        let now = self.clock.now_ms();
        let procs = self.procs.snapshot();
        let me = self.me(&procs);

        self.executor.run_pending_kills(now, &procs, &mut out);

        let checks: Vec<CheckResult> = if self.cfg.watch.is_empty() {
            Vec::new()
        } else {
            self.checks.check(&self.cfg, &procs)
        };
        let agents_now = agents::collect(&procs, &self.cfg.targets, &me);

        let status = if !self.cfg.monitoring || self.cfg.watch.is_empty() {
            self.idle(&procs, &mut out)
        } else {
            let verdict = combine(self.cfg.mode, &checks);
            let transition = self.machine.step(now, verdict, &self.cfg);
            self.settle_leftover(now, verdict, &procs, &mut out);
            self.apply(transition, now, &checks, &agents_now, &procs, &mut out);
            self.machine.status(verdict)
        };

        self.executor.sync_record(&[], &mut out);

        if self.status != Some(status) {
            self.status = Some(status);
            self.last_change_at = Some(self.clock.stamp());
        }
        let countdown_left_sec = self.machine.countdown_left_sec(now);
        let (color, title) = status_view(self.cfg.monitoring, status, countdown_left_sec);
        self.state = AppState {
            monitoring: self.cfg.monitoring,
            dry_run: self.cfg.dry_run,
            status,
            countdown_left_sec,
            checks,
            agents: self.sessions(&agents_now),
            last_change_at: self.last_change_at.clone(),
            color,
            title,
        };
        out.state = self.state.clone();
        out
    }

    /// Наблюдение выключено или не за чем следить: разморозить и сбросить.
    fn idle(&mut self, procs: &[ProcInfo], out: &mut TickOutput) -> Status {
        let why = if self.cfg.monitoring {
            messages::WHY_NOTHING_WATCHED
        } else {
            messages::WHY_MONITORING_OFF
        };
        self.executor.release_leftover(procs, out);
        self.executor.release_all(procs, out, why);
        self.machine.reset();
        self.executor.forget_handled();
        if self.cfg.monitoring {
            Status::Unknown
        } else {
            Status::Paused
        }
    }

    /// Замороженные прошлым запуском: пока не ясно, есть ли VPN, не отпускаем —
    /// иначе их запросы уйдут напрямую.
    fn settle_leftover(
        &mut self,
        now: u64,
        verdict: Verdict,
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        if !self.executor.has_leftover() {
            return;
        }
        let since = *self.hold_since.get_or_insert(now);
        let undecided = matches!(self.machine.status(verdict), Status::Down | Status::Unknown);
        let expired = verdict == Verdict::Unknown && now - since >= LEFTOVER_HOLD_MS;
        if !undecided || expired {
            self.executor.release_leftover(procs, out);
        }
    }

    /// Выполнить действия перехода.
    fn apply(
        &mut self,
        transition: Transition,
        now: u64,
        checks: &[CheckResult],
        agents_now: &[AgentProc],
        procs: &[ProcInfo],
        out: &mut TickOutput,
    ) {
        let cfg = &self.cfg;
        let ex = &mut self.executor;
        match transition {
            Transition::None => {}
            Transition::Down { close_now } => {
                ex.enter_down(cfg, now, checks, agents_now, procs, close_now, out)
            }
            Transition::BackInCountdown => ex.vpn_back(cfg, procs, out),
            Transition::Expired => ex.close(cfg, now, agents_now, procs, out, false),
            Transition::FreezeNew => ex.freeze_new(cfg, agents_now, out),
            Transition::BackAfterClose => ex.back_after_close(cfg, out),
            Transition::CloseNew => ex.close_new(cfg, now, agents_now, procs, out),
        }
    }

    /// В состоянии — только сами сессии (корни деревьев).
    fn sessions(&self, agents_now: &[AgentProc]) -> Vec<AgentInfo> {
        agents_now
            .iter()
            .filter(|a| a.is_root())
            .map(|a| AgentInfo {
                target: a.target,
                pid: a.pid,
                name: a.name.clone(),
                root: true,
                root_pid: a.pid,
                frozen: self.executor.is_really_frozen(a),
            })
            .collect()
    }

    /// Перед выходом: разморозить замороженных.
    pub fn shutdown(&mut self) -> TickOutput {
        let mut out = TickOutput::default();
        let procs = self.procs.snapshot();
        self.executor.release_leftover(&procs, &mut out);
        self.executor
            .release_all(&procs, &mut out, messages::WHY_SHUTDOWN);
        self.executor.sync_record(&[], &mut out);
        out.state = self.state.clone();
        out
    }
}
