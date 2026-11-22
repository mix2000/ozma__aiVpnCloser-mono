//! Стенд монитора на подставных источниках: без сети и без сигналов.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use vpn_closer_core::models::{
    Action, ActionReport, CheckResult, Config, ProcInfo, SelfCtx, WatchItem,
};
use vpn_closer_core::testing::{
    Actor, CheckSource, FakeClock, Me, Monitor, MonitorParts, ProcSource, TickOutput,
};

pub type ActionLog = Arc<Mutex<Vec<(Action, Vec<u32>)>>>;

#[derive(Clone, Default)]
pub struct Shared {
    pub vpn: Arc<Mutex<Option<bool>>>,
    pub procs: Arc<Mutex<Vec<ProcInfo>>>,
    pub log: ActionLog,
}

pub struct FakeChecks(Arc<Mutex<Option<bool>>>);
impl CheckSource for FakeChecks {
    fn check(&mut self, cfg: &Config, _: &[ProcInfo]) -> Vec<CheckResult> {
        let ok = *self.0.lock().unwrap();
        cfg.watch
            .iter()
            .map(|w| CheckResult::new(w.clone(), ok, "тест"))
            .collect()
    }
}

pub struct FakeProcs(Arc<Mutex<Vec<ProcInfo>>>);
impl ProcSource for FakeProcs {
    fn snapshot(&mut self) -> Vec<ProcInfo> {
        self.0.lock().unwrap().clone()
    }
}

pub struct RecActor(ActionLog);
impl Actor for RecActor {
    fn apply(&mut self, action: Action, pids: &[u32]) -> ActionReport {
        self.0.lock().unwrap().push((action, pids.to_vec()));
        ActionReport {
            ok: pids.to_vec(),
            ..Default::default()
        }
    }
}

pub fn proc_(pid: u32, ppid: u32, name: &str) -> ProcInfo {
    ProcInfo {
        pid,
        ppid: Some(ppid),
        uid: Some("501".into()),
        name: name.into(),
        exe: None,
        cmd: vec![name.into()],
        start_time: 10_000 + pid as u64,
    }
}

pub fn base_procs() -> Vec<ProcInfo> {
    vec![
        proc_(1, 0, "launchd"),
        proc_(200, 1, "claude"),
        proc_(201, 200, "zsh"),
        proc_(202, 201, "sleep"),
        proc_(300, 1, "codex"),
    ]
}

pub fn setup(cfg: Config) -> (Monitor, Shared, Arc<FakeClock>) {
    let sh = Shared::default();
    *sh.vpn.lock().unwrap() = Some(true);
    *sh.procs.lock().unwrap() = base_procs();
    let clock = Arc::new(FakeClock::default());
    let parts = MonitorParts {
        checks: Box::new(FakeChecks(sh.vpn.clone())),
        procs: Box::new(FakeProcs(sh.procs.clone())),
        actor: Box::new(RecActor(sh.log.clone())),
        clock: clock.clone(),
        me: Me::Fixed(SelfCtx {
            pid: 99_999,
            uid: Some("501".into()),
        }),
    };
    (Monitor::new(cfg, parts), sh, clock)
}

pub fn live_cfg() -> Config {
    Config {
        watch: vec![WatchItem::Route],
        monitoring: true,
        dry_run: false,
        ..Config::default()
    }
}

pub fn set_vpn(sh: &Shared, v: Option<bool>) {
    *sh.vpn.lock().unwrap() = v;
}

pub fn log(sh: &Shared) -> Vec<(Action, Vec<u32>)> {
    sh.log.lock().unwrap().clone()
}

pub fn tick(m: &mut Monitor, clock: &FakeClock) -> TickOutput {
    let out = m.tick();
    clock.advance_ms(1000);
    out
}
