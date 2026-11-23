//! Монитор: программу убили, пока агенты заморожены, — следующий запуск их не бросает.

mod common;

use std::sync::Arc;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, EventKind, Status};
use vpn_closer_core::testing::{FakeClock, Monitor};

/// Заморозить и «упасть»: новый монитор на том же хранилище, журнал действий пуст.
fn crashed(cfg: Config) -> (Monitor, Shared, Arc<FakeClock>) {
    let (mut m, sh, c) = setup(live_cfg());
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    assert!(sh.frozen.lock().unwrap().is_empty());
    tick(&mut m, &c);
    assert_eq!(sh.frozen.lock().unwrap().len(), 4);
    drop(m);
    sh.log.lock().unwrap().clear();
    let m = monitor(cfg, &sh, c.clone());
    (m, sh, c)
}

fn resumed(sh: &Shared) -> Vec<u32> {
    let l = log(sh);
    assert_eq!(l.len(), 1, "{l:?}");
    assert_eq!(l[0].0, Action::Resume);
    let mut pids = l[0].1.clone();
    pids.sort();
    pids
}

#[test]
fn vpn_up_after_restart_resumes() {
    let (mut m, sh, c) = crashed(live_cfg());
    // pid 300 за это время занял другой процесс.
    sh.procs.lock().unwrap()[4].start_time += 1;
    set_vpn(&sh, Some(true));
    let out = tick(&mut m, &c);
    assert_eq!(resumed(&sh), vec![200, 201, 202]);
    assert!(out.events.iter().any(|e| e.kind == EventKind::Resume));
    assert!(out.state.agents.iter().all(|a| !a.frozen));
    assert!(sh.frozen.lock().unwrap().is_empty());
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), 1, "повторно не трогаем");
}

#[test]
fn vpn_down_after_restart_stays_frozen_and_is_frozen_again() {
    let (mut m, sh, c) = crashed(live_cfg());
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Down);
    assert!(log(&sh).is_empty(), "пока не подтверждено — держим");
    assert!(out.state.agents.iter().all(|a| a.frozen));
    assert_eq!(tick(&mut m, &c).state.status, Status::Countdown);
    let l = log(&sh);
    assert_eq!(
        (l[0].0, l[1].0, l.len()),
        (Action::Resume, Action::Freeze, 2)
    );
    assert_eq!(l[1].1, vec![200, 201, 202, 300]);
    assert_eq!(sh.frozen.lock().unwrap().len(), 4);
    // Второе падение подряд — запись снова на месте, возврат VPN размораживает.
    sh.log.lock().unwrap().clear();
    let mut m = monitor(live_cfg(), &sh, c.clone());
    set_vpn(&sh, Some(true));
    tick(&mut m, &c);
    assert_eq!(resumed(&sh).len(), 4);
}

#[test]
fn unknown_after_restart_holds_for_limited_time() {
    let (mut m, sh, c) = crashed(live_cfg());
    set_vpn(&sh, None);
    for _ in 0..30 {
        tick(&mut m, &c);
        assert!(log(&sh).is_empty());
    }
    tick(&mut m, &c);
    assert_eq!(resumed(&sh).len(), 4);
}

#[test]
fn monitoring_off_or_dry_run_or_shutdown_releases() {
    let off = Config {
        monitoring: false,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(off);
    tick(&mut m, &c);
    assert_eq!(resumed(&sh).len(), 4);

    let dry = Config {
        dry_run: true,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(dry);
    for _ in 0..3 {
        tick(&mut m, &c);
    }
    assert_eq!(resumed(&sh).len(), 4, "«только смотреть» никого не держит");

    let (mut m, sh, _) = crashed(live_cfg());
    m.shutdown();
    assert_eq!(resumed(&sh).len(), 4);
    assert!(sh.frozen.lock().unwrap().is_empty());
}

#[test]
fn closing_clears_record() {
    let cfg = Config {
        close_after_sec: 1,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(sh.frozen.lock().unwrap().len(), 4);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Closed);
    assert!(sh.frozen.lock().unwrap().is_empty());
}
