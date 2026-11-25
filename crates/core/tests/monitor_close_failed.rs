//! Монитор: закрыть не удалось — повторяем, замороженных не теряем, врать не врём.

mod common;

use std::sync::Arc;

use common::monitor::*;
use vpn_closer_core::models::{Action, Config, EventKind, Status};
use vpn_closer_core::testing::{FakeClock, Monitor, TickOutput};

fn cfg() -> Config {
    Config {
        close_after_sec: 1,
        kill_grace_sec: 5,
        ..live_cfg()
    }
}

/// Время вышло; `unkillable` закрыть не удалось.
fn closed(cfg: Config, unkillable: &[u32]) -> (Monitor, Shared, Arc<FakeClock>, TickOutput) {
    let (mut m, sh, c) = setup(cfg);
    *sh.unkillable.lock().unwrap() = unkillable.to_vec();
    set_vpn(&sh, Some(false));
    tick(&mut m, &c);
    tick(&mut m, &c);
    let out = tick(&mut m, &c);
    assert_eq!(out.state.status, Status::Closed);
    // Закрытые исчезли.
    sh.procs
        .lock()
        .unwrap()
        .retain(|p| p.pid == 1 || unkillable.contains(&p.pid));
    (m, sh, c, out)
}

fn frozen_pids(sh: &Shared) -> Vec<u32> {
    sh.frozen.lock().unwrap().iter().map(|f| f.pid).collect()
}

fn texts(out: &TickOutput, kind: EventKind) -> Vec<&str> {
    out.events
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| e.message.as_str())
        .collect()
}

#[test]
fn retried_until_closed() {
    let (mut m, sh, c, out) = closed(cfg(), &[300]);
    assert_eq!(texts(&out, EventKind::Error).len(), 1);
    assert!(texts(&out, EventKind::Close)[0].contains("claude ×1 сессия"));
    assert!(!texts(&out, EventKind::Close)[0].contains("codex"));
    assert_eq!(out.notices[0].title, "Не все агенты закрыты");
    assert!(out.notices[0]
        .body
        .contains("Не удалось закрыть: 1 процесс"));
    for _ in 0..3 {
        let out = tick(&mut m, &c);
        assert_eq!(log(&sh).last().unwrap(), &(Action::Terminate, vec![300]));
        assert!(out.events.is_empty() && out.notices.is_empty(), "не шумим");
    }
    sh.unkillable.lock().unwrap().clear();
    let out = tick(&mut m, &c);
    assert_eq!(texts(&out, EventKind::Close)[0], "Закрыто: codex ×1 сессия");
    assert!(frozen_pids(&sh).is_empty());
    sh.procs.lock().unwrap().retain(|p| p.pid == 1);
    let n = log(&sh).len();
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}

#[test]
fn frozen_is_not_forgotten() {
    let (mut m, sh, c, out) = closed(cfg(), &[300]);
    assert!(out.state.agents.iter().any(|a| a.pid == 300 && a.frozen));
    assert_eq!(frozen_pids(&sh), vec![300], "достанется и новому запуску");
    set_vpn(&sh, Some(true));
    let out = tick(&mut m, &c);
    assert_eq!(log(&sh).last().unwrap(), &(Action::Resume, vec![300]));
    assert!(out.state.agents.iter().all(|a| !a.frozen));
    assert!(frozen_pids(&sh).is_empty());
    let n = log(&sh).len();
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n, "VPN вернулся — больше не закрываем");
    // Снова пропал: закрываем один раз, а не ещё и по старой памяти.
    set_vpn(&sh, Some(false));
    for _ in 0..3 {
        tick(&mut m, &c);
    }
    assert_eq!(m.state().status, Status::Closed);
    let l = log(&sh)[n..].to_vec();
    assert_eq!(
        l.iter().map(|x| x.0).collect::<Vec<_>>(),
        [Action::Freeze, Action::Terminate]
    );
}

#[test]
fn running_one_is_retried_but_not_marked_frozen() {
    let no_freeze = Config {
        freeze_immediately: false,
        block_while_down: false,
        ..cfg()
    };
    let (mut m, sh, c, out) = closed(no_freeze, &[300]);
    assert!(out.state.agents.iter().all(|a| !a.frozen));
    assert!(frozen_pids(&sh).is_empty());
    tick(&mut m, &c);
    assert_eq!(log(&sh).last().unwrap(), &(Action::Terminate, vec![300]));
    // «Только смотреть» — не трогаем; наблюдение выключено — забыли.
    for change in [true, false] {
        let mut now = m.config().clone();
        now.dry_run = change;
        now.monitoring = change;
        m.set_config(now);
        let n = log(&sh).len();
        tick(&mut m, &c);
        assert_eq!(log(&sh).len(), n);
    }
    let on = Config {
        monitoring: true,
        confirm_failures: 100,
        ..m.config().clone()
    };
    m.set_config(on);
    let n = log(&sh).len();
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}

#[test]
fn nobody_closed() {
    let (_, _, _, out) = closed(cfg(), &[200, 201, 202, 300]);
    assert!(texts(&out, EventKind::Close).is_empty());
    assert_eq!(texts(&out, EventKind::Error).len(), 4);
    assert!(out.notices[0].body.starts_with("Не удалось закрыть: 4"));
}

#[test]
fn pid_reuse_is_not_closed() {
    let (mut m, sh, c, _) = closed(cfg(), &[300]);
    let n = log(&sh).len();
    {
        let mut p = sh.procs.lock().unwrap();
        p[1].start_time += 1;
        p[1].name = "bash".into();
        p[1].cmd = vec!["bash".into()];
    }
    tick(&mut m, &c);
    assert_eq!(log(&sh).len(), n);
}
