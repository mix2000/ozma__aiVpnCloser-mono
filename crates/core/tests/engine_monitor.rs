//! Engine на подставной платформе: монитор, маршрут и владелец туннеля,
//! действия через SystemActor (в журнал, без сигналов), подписки, discover.

mod common;

use std::sync::{Arc, Mutex};

use common::platform::{wait_for, Machine};
use vpn_closer_core::models::{Action, AppState, Config, EventKind, Notice, Status, WatchItem};
use vpn_closer_core::Engine;

fn live_config() -> Config {
    Config {
        watch: vec![
            WatchItem::Interface {
                name: "utun233".into(),
                addr: Some("172.19.0.1".into()),
            },
            WatchItem::Route,
        ],
        hosts: vec!["10.10.10.10".into()],
        monitoring: true,
        dry_run: false,
        poll_ms: 200,
        confirm_failures: 1,
        ..Config::default()
    }
}

fn started(machine: &Machine) -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let e = Engine::with_platform(dir.path(), machine.parts()).unwrap();
    e.start();
    e.save_config(live_config()).unwrap();
    (dir, e)
}

#[test]
fn route_through_tunnel_with_owner() {
    let machine = Machine::new();
    let (_dir, e) = started(&machine);
    assert!(wait_for(|| e.get_state().status == Status::Ok));
    let st = e.get_state();
    let owner = st.checks[0].owner.as_ref().expect("владелец utun233");
    assert_eq!((owner.name.as_str(), owner.pid), ("sing-box", Some(75686)));
    assert!(
        st.checks[1].detail.contains("utun233"),
        "{}",
        st.checks[1].detail
    );
    assert_eq!(st.title, "VPN включён");
    assert_eq!(st.agents.len(), 1, "в состоянии только сессии");
}

#[test]
fn traffic_bypass_freezes_then_monitoring_off_resumes() {
    let machine = Machine::new();
    let (_dir, e) = started(&machine);
    let notices: Arc<Mutex<Vec<Notice>>> = Arc::default();
    let n = notices.clone();
    e.subscribe_notices(move |x| n.lock().unwrap().push(x.clone()));
    assert!(wait_for(|| e.get_state().status == Status::Ok));

    machine.set_via_tunnel(false);
    assert!(wait_for(|| e.get_state().status == Status::Countdown));
    assert!(machine.did(Action::Freeze));
    assert!(e.get_state().agents[0].frozen);
    assert!(e.get_state().checks[1].detail.contains("не через VPN"));
    assert!(wait_for(|| !notices.lock().unwrap().is_empty()));

    let st = e.set_monitoring(false).unwrap();
    assert_eq!(st.status, Status::Paused);
    assert!(machine.did(Action::Resume));
    let kinds: Vec<EventKind> = e.recent_events(50).iter().map(|x| x.kind).collect();
    assert!(kinds.contains(&EventKind::VpnDown) && kinds.contains(&EventKind::Resume));
    e.shutdown();
}

#[test]
fn state_subscribers_get_changes() {
    let machine = Machine::new();
    let (_dir, e) = started(&machine);
    let states: Arc<Mutex<Vec<AppState>>> = Arc::default();
    let s = states.clone();
    e.subscribe(move |st| s.lock().unwrap().push(st.clone()));
    machine.set_via_tunnel(false);
    assert!(wait_for(|| states
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.status == Status::Countdown)));
}

#[test]
fn discover_and_test_targets_use_engine_platform() {
    let machine = Machine::new();
    let dir = tempfile::tempdir().unwrap();
    let e = Engine::with_platform(dir.path(), machine.parts()).unwrap();
    let d = e.discover();
    let tun = d.interfaces.iter().find(|i| i.name == "utun233").unwrap();
    assert_eq!(tun.owner.as_ref().unwrap().name, "sing-box");
    assert!(d
        .processes
        .iter()
        .any(|p| p.name == "sing-box" && p.known_vpn));
    let targets = e.test_targets();
    let pids: Vec<u32> = targets.iter().map(|a| a.pid).collect();
    assert_eq!(pids, vec![4242, 4243], "сессия и её потомок");
    assert!(targets[0].root && !targets[1].root);
}
