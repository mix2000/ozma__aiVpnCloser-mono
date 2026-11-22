//! Монитор: «только смотреть», выключение наблюдения, пустой выбор, уведомления.

mod common;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, Event, EventKind, Status};

#[test]
fn dry_run_sends_nothing() {
    let cfg = Config {
        dry_run: true,
        close_after_sec: 2,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    let mut events = Vec::new();
    for _ in 0..6 {
        events.extend(tick(&mut m, &c).events);
    }
    assert_eq!(m.state().status, Status::Closed);
    sh.procs.lock().unwrap().push(proc_(400, 1, "grok"));
    for _ in 0..3 {
        events.extend(tick(&mut m, &c).events);
    }
    set_vpn(&sh, Some(true));
    events.extend(tick(&mut m, &c).events);
    assert!(
        log(&sh).is_empty(),
        "в режиме «только смотреть» сигналов нет"
    );
    let dry: Vec<&Event> = events
        .iter()
        .filter(|e| e.kind == EventKind::DryRun)
        .collect();
    assert!(dry.iter().any(|e| e.message.starts_with("Заморозил бы")));
    assert!(dry.iter().any(|e| e.message.starts_with("Закрыл бы")));
    assert_eq!(
        dry.iter()
            .filter(|e| e.message.contains("Запущены без VPN"))
            .count(),
        1,
        "о новом процессе сообщаем один раз"
    );
    assert!(m.state().agents.iter().all(|a| !a.frozen));
}

#[test]
fn turning_monitoring_off_resumes() {
    let (mut m, sh, c) = setup(live_cfg());
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c);
    let mut cfg = m.config().clone();
    cfg.monitoring = false;
    m.set_config(cfg);
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Paused);
    assert_eq!(log(&sh).last().unwrap().0, Action::Resume);
    assert!(out.state.countdown_left_sec.is_none());
}

#[test]
fn empty_watch_is_unknown_and_paused_when_off() {
    let cfg = Config {
        watch: vec![],
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    for _ in 0..5 {
        assert_eq!(tick(&mut m, &c).state.status, Status::Unknown);
    }
    assert!(log(&sh).is_empty());
    let (mut m, _sh, c) = setup(Config::default());
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Paused);
    assert!(out.state.dry_run && !out.state.monitoring);
    assert!(out.state.last_change_at.is_some());
}

#[test]
fn notices_respect_notify_flag() {
    let cfg = Config {
        notify: false,
        ..live_cfg()
    };
    let (mut m, sh, c) = setup(cfg);
    set_vpn(&sh, Some(false));
    let mut notices = Vec::new();
    for _ in 0..3 {
        notices.extend(tick(&mut m, &c).notices);
    }
    assert!(notices.is_empty());
}
