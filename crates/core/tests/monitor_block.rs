//! Монитор: блокировка новых агентов, пока VPN нет, и защита от переиспользования pid.

mod common;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, EventKind, Status};

#[test]
fn block_while_down_new_processes() {
    let cfg = Config {
        close_after_sec: 5,
        kill_grace_sec: 0,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c); // countdown, frozen
    sh.procs.lock().unwrap().push(proc_(400, 1, "grok"));
    let out = tick(&mut m, &c);
    assert_eq!(log(&sh).last().unwrap(), &(Action::Freeze, vec![400]));
    assert!(out.events.iter().any(|e| e.kind == EventKind::Block));
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), 2, "повторно не морозим");
    // Отсчёт кончился — закрыты все, включая 400.
    for _ in 0..4 {
        tick(&mut m, &c);
    }
    assert_eq!(m.state().status, Status::Closed);
    sh.procs.lock().unwrap().retain(|p| p.pid == 1);
    tick(&mut m, &c);
    // Новый claude, пока VPN нет — закрыт сразу.
    sh.procs.lock().unwrap().push(proc_(500, 1, "claude"));
    let before = log(&sh).len();
    let out = tick(&mut m, &c);
    let l = log(&sh);
    assert_eq!(l[before], (Action::Terminate, vec![500]));
    assert!(out.events.iter().any(|e| e.kind == EventKind::Block));
}

#[test]
fn no_block_when_disabled() {
    let cfg = Config {
        close_after_sec: 0,
        block_while_down: false,
        confirm_failures: 1,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    sh.procs.lock().unwrap().retain(|p| p.pid == 1);
    tick(&mut m, &c);
    let n = log(&sh).len();
    sh.procs.lock().unwrap().push(proc_(500, 1, "claude"));
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}

#[test]
fn pid_reuse_is_not_touched() {
    let (mut m, sh, c) = setup(live_cfg());
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c); // frozen 200,300,201,202
                      // pid 300 переиспользован другим процессом.
    {
        let mut p = sh.procs.lock().unwrap();
        let x = p.iter_mut().find(|p| p.pid == 300).unwrap();
        x.name = "bash".into();
        x.cmd = vec!["bash".into()];
        x.start_time += 1;
    }
    let out = m.shutdown();
    let l = log(&sh);
    let mut resumed = l.last().unwrap().1.clone();
    resumed.sort();
    assert_eq!(resumed, vec![200, 201, 202]);
    assert!(out.events.iter().any(|e| e.kind == EventKind::Resume));
}
