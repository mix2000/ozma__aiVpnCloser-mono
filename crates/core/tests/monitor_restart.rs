//! Монитор: программу убили, пока агенты заморожены, — следующий запуск их не бросает.

mod common;

use std::sync::Arc;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, EventKind, Status};
use vpn_closer_core::testing::{FakeClock, Monitor, TickOutput};

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
    tick(&mut m, &c);
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

fn errors(out: &TickOutput) -> usize {
    out.events
        .iter()
        .filter(|e| e.kind == EventKind::Error)
        .count()
}

fn frozen_pids(sh: &Shared) -> Vec<u32> {
    sh.frozen.lock().unwrap().iter().map(|f| f.pid).collect()
}

#[test]
fn failed_resume_is_kept_and_retried() {
    let (mut m, sh, c) = crashed(live_cfg());
    set_vpn(&sh, Some(true));
    *sh.stuck.lock().unwrap() = vec![300];
    let out = tick(&mut m, &c);
    assert_eq!(errors(&out), 1);
    assert_eq!(frozen_pids(&sh), vec![300], "запись не потеряна");
    let codex = |o: &TickOutput| o.state.agents.iter().any(|a| a.pid == 300 && a.frozen);
    assert!(codex(&out), "в окне видно, что заморожен");
    let out = tick(&mut m, &c);
    assert_eq!(errors(&out), 0, "об ошибке — один раз");
    assert_eq!(log(&sh).last().unwrap(), &(Action::Resume, vec![300]));
    sh.stuck.lock().unwrap().clear();
    let out = tick(&mut m, &c);
    assert!(!codex(&out));
    assert!(frozen_pids(&sh).is_empty());
    let n = log(&sh).len();
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}

#[test]
fn failed_resume_kept_after_vpn_back_and_shutdown() {
    let (mut m, sh, c) = setup(live_cfg());
    *sh.stuck.lock().unwrap() = vec![300];
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c);
    set_vpn(&sh, Some(true));
    let out = tick(&mut m, &c);
    assert!(out.events.iter().any(|e| e.kind == EventKind::Resume));
    assert_eq!(frozen_pids(&sh), vec![300]);
    // Процесс умер — забыли.
    sh.procs.lock().unwrap().retain(|p| p.pid != 300);
    tick(&mut m, &c);
    assert!(frozen_pids(&sh).is_empty());

    let (mut m, sh, c) = setup(live_cfg());
    *sh.stuck.lock().unwrap() = vec![300];
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c);
    m.shutdown();
    assert_eq!(frozen_pids(&sh), vec![300], "достанется следующему запуску");
}

#[test]
fn stuck_one_is_not_resumed_during_countdown() {
    let (mut m, sh, c) = crashed(live_cfg());
    *sh.stuck.lock().unwrap() = vec![300];
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Countdown);
    let l = log(&sh);
    assert_eq!(
        l[1],
        (Action::Freeze, vec![200, 201, 202]),
        "300 и так заморожен"
    );
    sh.stuck.lock().unwrap().clear();
    for _ in 0..5 {
        tick(&mut m, &c);
    }
    assert_eq!(log(&sh).len(), 2, "VPN нет — никого не отпускаем");
    set_vpn(&sh, Some(true));
    let out = tick(&mut m, &c);
    assert!(out.state.agents.iter().all(|a| !a.frozen));
    assert!(frozen_pids(&sh).is_empty());
}

#[test]
fn stuck_one_is_closed_when_time_is_up() {
    let cfg = Config {
        close_after_sec: 2,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(cfg);
    *sh.stuck.lock().unwrap() = vec![202, 300];
    // Корень 200 умер: 202 остался сиротой и агентом больше не считается.
    sh.procs
        .lock()
        .unwrap()
        .retain(|p| p.pid != 200 && p.pid != 201);
    sh.procs.lock().unwrap()[1].ppid = Some(1);
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Countdown);
    assert_eq!(frozen_pids(&sh), vec![202, 300]);
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Closed);
    assert_eq!(
        log(&sh).last().unwrap(),
        &(Action::Terminate, vec![202, 300])
    );
    assert!(frozen_pids(&sh).is_empty(), "закрыты — забыли");
    let n = log(&sh).len();
    sh.procs.lock().unwrap().retain(|p| p.pid == 1);
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}

#[test]
fn notice_tells_about_stuck_one() {
    let body = |stuck: Vec<u32>| {
        let (mut m, sh, c) = setup(live_cfg());
        *sh.stuck.lock().unwrap() = stuck;
        set_vpn(&sh, Some(false));
        tick(&mut m, &c);
        tick(&mut m, &c);
        set_vpn(&sh, Some(true));
        tick(&mut m, &c).notices[0].body.clone()
    };
    assert_eq!(body(vec![]), "Агенты продолжают работу");
    assert!(body(vec![300]).starts_with("Не всех агентов"));
}

#[test]
fn failed_write_is_not_retried_between_resume_and_freeze() {
    let (mut m, sh, c) = crashed(live_cfg());
    sh.procs.lock().unwrap().push(proc_(150, 1, "grok"));
    tick(&mut m, &c);
    *sh.disk_full.lock().unwrap() = true;
    sh.writes.lock().unwrap().clear();
    let out = tick(&mut m, &c);
    assert_eq!(errors(&out), 1, "о неудаче — один раз");
    let l = log(&sh);
    assert_eq!((l[0].0, l[1].0), (Action::Resume, Action::Freeze));
    assert_eq!(*sh.writes.lock().unwrap(), [0, 2], "до и после, не между");
    *sh.disk_full.lock().unwrap() = false;
    tick(&mut m, &c);
    assert_eq!(frozen_pids(&sh), vec![150, 200, 201, 202, 300]);
}

#[test]
fn dry_run_retries_stuck_one() {
    let dry = Config {
        dry_run: true,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(dry);
    *sh.stuck.lock().unwrap() = vec![300];
    tick(&mut m, &c);
    assert_eq!(frozen_pids(&sh), vec![300]);
    sh.stuck.lock().unwrap().clear();
    tick(&mut m, &c);
    assert_eq!(log(&sh).last().unwrap(), &(Action::Resume, vec![300]));
    assert!(frozen_pids(&sh).is_empty());
    assert!(log(&sh).iter().all(|(a, _)| *a == Action::Resume));
}

#[test]
fn nothing_is_written_between_resume_and_freeze() {
    let (mut m, sh, c) = crashed(live_cfg());
    // Новый агент, которого в записи ещё нет, и умерший из записи.
    sh.procs.lock().unwrap().push(proc_(150, 1, "grok"));
    sh.procs.lock().unwrap().retain(|p| p.pid != 202);
    sh.writes.lock().unwrap().clear();
    tick(&mut m, &c);
    tick(&mut m, &c);
    let l = log(&sh);
    assert_eq!((l[0].0, l[1].0), (Action::Resume, Action::Freeze));
    let writes = sh.writes.lock().unwrap().clone();
    assert!(!writes.contains(&1), "между ними — без диска: {writes:?}");
    assert_eq!(writes[0], 0, "записали до разморозки");
    assert_eq!(frozen_pids(&sh), vec![150, 200, 201, 300]);
}

#[test]
fn down_mixed_with_unknown_is_held_for_limited_time() {
    let (mut m, sh, c) = crashed(live_cfg());
    tick(&mut m, &c); // «VPN нет», ещё не подтверждено
    set_vpn(&sh, None);
    for _ in 0..29 {
        tick(&mut m, &c);
        assert!(log(&sh).is_empty());
    }
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Countdown, "подтвердилось");
    assert!(m.state().agents.iter().all(|a| a.frozen));

    let (mut m, sh, c) = crashed(live_cfg());
    tick(&mut m, &c);
    set_vpn(&sh, None);
    for _ in 0..30 {
        tick(&mut m, &c);
        assert!(log(&sh).is_empty());
    }
    tick(&mut m, &c);
    assert_eq!(resumed(&sh).len(), 4);
}

#[test]
fn one_unknown_after_long_down_does_not_release() {
    let slow = Config {
        confirm_failures: 100,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(slow);
    for _ in 0..60 {
        tick(&mut m, &c);
    }
    set_vpn(&sh, None);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Down);
    assert!(log(&sh).is_empty(), "30 с — с первого «неизвестно»");
    // Чередование не продлевает удержание бесконечно.
    for i in 0..30 {
        set_vpn(&sh, (i % 2 == 0).then_some(false));
        tick(&mut m, &c);
    }
    assert_eq!(resumed(&sh).len(), 4);
}

#[test]
fn restart_without_freeze_and_with_immediate_close() {
    let no_freeze = Config {
        freeze_immediately: false,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(no_freeze);
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Countdown);
    assert_eq!(resumed(&sh).len(), 4, "замораживать не просили — отпущены");

    let close_now = Config {
        close_after_sec: 0,
        ..live_cfg()
    };
    let (mut m, sh, c) = crashed(close_now);
    tick(&mut m, &c);
    tick(&mut m, &c);
    assert_eq!(m.state().status, Status::Closed);
    let l = log(&sh);
    assert_eq!(
        l.last().unwrap(),
        &(Action::Terminate, vec![202, 201, 200, 300])
    );
    assert!(frozen_pids(&sh).is_empty());
}
