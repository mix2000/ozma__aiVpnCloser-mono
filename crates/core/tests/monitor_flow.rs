//! Монитор: дребезг, отсчёт, возврат VPN, закрытие (дети раньше родителей) и добивание.

mod common;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, EventKind, Status};

#[test]
fn debounce_confirm_failures() {
    let (mut m, sh, c) = setup(live_cfg());
    assert_eq!(tick(&mut m, &c).state.status, Status::Ok);
    set_vpn(&sh, Some(false));
    assert_eq!(tick(&mut m, &c).state.status, Status::Down);
    set_vpn(&sh, Some(true));
    assert_eq!(tick(&mut m, &c).state.status, Status::Ok);
    assert!(log(&sh).is_empty(), "один сбой — без действий");
    // Неизвестно не сбрасывает и не увеличивает счётчик.
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    set_vpn(&sh, None);
    assert_eq!(tick(&mut m, &c).state.status, Status::Down);
    assert!(log(&sh).is_empty());
    set_vpn(&sh, Some(false));
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Countdown);
    assert_eq!(out.state.countdown_left_sec, Some(30));
    assert_eq!(
        log(&sh),
        vec![(Action::Freeze, vec![200, 201, 202, 300])],
        "родители раньше детей"
    );
    assert!(out.events.iter().any(|e| e.kind == EventKind::VpnDown));
    assert!(out
        .notices
        .iter()
        .any(|n| n.body.contains("закрою через 30")));
    assert!(out.state.agents.iter().all(|a| a.frozen));
    assert_eq!(out.state.agents.len(), 2, "в состоянии только корни");
}

#[test]
fn vpn_back_before_deadline_resumes() {
    let (mut m, sh, c) = setup(live_cfg());
    tick(&mut m, &c);
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c); // countdown
    for _ in 0..10 {
        assert_eq!(tick(&mut m, &c).state.status, Status::Countdown);
    }
    set_vpn(&sh, Some(true));
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Ok);
    let l = log(&sh);
    assert_eq!(l.len(), 2);
    assert_eq!(l[1].0, Action::Resume);
    let mut resumed = l[1].1.clone();
    resumed.sort();
    assert_eq!(resumed, vec![200, 201, 202, 300]);
    assert!(out.events.iter().any(|e| e.kind == EventKind::Resume));
    assert!(out.state.agents.iter().all(|a| !a.frozen));
}

#[test]
fn countdown_expiry_terminates_children_first_then_kills() {
    let cfg = Config {
        close_after_sec: 3,
        kill_grace_sec: 2,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    tick(&mut m, &c); // t=0 down
    let out = tick(&mut m, &c); // t=1 countdown, deadline=4
    assert_eq!(out.state.countdown_left_sec, Some(3));
    assert_eq!(tick(&mut m, &c).state.countdown_left_sec, Some(2));
    assert_eq!(tick(&mut m, &c).state.countdown_left_sec, Some(1));
    let out = tick(&mut m, &c); // t=4 close
    assert_eq!(out.state.status, Status::Closed);
    assert!(out.notices.iter().any(|n| n.body.contains("claude ×1")));
    let l = log(&sh);
    assert_eq!(l[1], (Action::Terminate, vec![202, 201, 200, 300]));
    // 202 и 300 завершились, остальные висят; через 2 с — SIGKILL им.
    sh.procs
        .lock()
        .unwrap()
        .retain(|p| p.pid != 202 && p.pid != 300);
    tick(&mut m, &c); // t=5
    assert_eq!(log(&sh).len(), 2);
    tick(&mut m, &c); // t=6
    let l = log(&sh);
    assert_eq!(l.len(), 3);
    assert_eq!(l[2].0, Action::Kill);
    let mut killed = l[2].1.clone();
    killed.sort();
    assert_eq!(killed, vec![200, 201]);
}

#[test]
fn close_after_zero_closes_immediately() {
    let cfg = Config {
        close_after_sec: 0,
        kill_grace_sec: 0,
        confirm_failures: 1,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Closed);
    let actions: Vec<Action> = log(&sh).iter().map(|(a, _)| *a).collect();
    assert_eq!(
        actions,
        vec![Action::Freeze, Action::Terminate, Action::Kill]
    );
    // VPN вернулся — снова ok.
    set_vpn(&sh, Some(true));
    assert_eq!(tick(&mut m, &c).state.status, Status::Ok);
}
