//! Engine: настройки — проверка, сохранение, события, пределы, атомарное переключение.

mod common;

use std::sync::{Arc, Mutex};

use common::platform::Machine;
use vpn_closer_core::models::{Config, EventKind, Status, WatchItem};
use vpn_closer_core::Engine;

fn engine(dir: &std::path::Path) -> Engine {
    Engine::with_platform(dir, Machine::new().parts()).unwrap()
}

#[test]
fn defaults_and_validation_error_in_russian() {
    let dir = tempfile::tempdir().unwrap();
    let e = engine(dir.path());
    assert_eq!(e.get_config(), Config::default());
    assert_eq!(e.get_state().status, Status::Paused);
    let err = e
        .save_config(Config {
            poll_ms: 1,
            ..Config::default()
        })
        .unwrap_err()
        .to_string();
    assert!(err.contains("Интервал"), "{err}");
    assert_eq!(e.get_config(), Config::default(), "конфиг не меняется");
}

#[test]
fn saved_config_and_events_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    {
        let e = engine(dir.path());
        e.save_config(Config {
            watch: vec![WatchItem::Route],
            poll_ms: 200,
            ..Config::default()
        })
        .unwrap();
        e.set_monitoring(true).unwrap();
    }
    let again = engine(dir.path());
    assert_eq!(again.get_config().poll_ms, 200);
    assert!(again.get_config().monitoring);
    let kinds: Vec<EventKind> = again.recent_events(10).iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&EventKind::Config));
}

#[test]
fn broken_config_is_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), "{not json").unwrap();
    let e = engine(dir.path());
    assert_eq!(e.get_config(), Config::default());
    assert!(dir.path().join("config.json.bad").exists());
    let ev = &e.recent_events(1)[0];
    assert_eq!(ev.kind, EventKind::Error);
    assert!(ev.message.contains("настройки сброшены"), "{}", ev.message);
}

#[test]
fn invalid_saved_config_is_set_aside_too() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), r#"{"pollMs": 5}"#).unwrap();
    let e = engine(dir.path());
    assert_eq!(e.get_config().poll_ms, 1000);
    assert!(dir.path().join("config.json.bad").exists());
}

#[test]
fn save_without_start_updates_state_flags() {
    let dir = tempfile::tempdir().unwrap();
    let e = engine(dir.path());
    let st = e.set_monitoring(true).unwrap();
    assert!(st.monitoring);
    assert!(dir.path().join("config.json").exists());
}

#[test]
fn toggle_monitoring_flips_and_notifies_config_subscribers() {
    let dir = tempfile::tempdir().unwrap();
    let e = Arc::new(engine(dir.path()));
    let seen: Arc<Mutex<Vec<bool>>> = Arc::default();
    let s = seen.clone();
    e.subscribe_config(move |c| s.lock().unwrap().push(c.monitoring));
    assert!(e.toggle_monitoring().unwrap().monitoring);
    assert!(!e.toggle_monitoring().unwrap().monitoring);
    // Параллельные переключения не теряются: чётное число — исходное состояние.
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let e = e.clone();
            std::thread::spawn(move || e.toggle_monitoring().unwrap())
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert!(!e.get_config().monitoring);
    assert_eq!(seen.lock().unwrap().len(), 6);
}

#[test]
fn limits_match_validation() {
    let dir = tempfile::tempdir().unwrap();
    let e = engine(dir.path());
    let l = e.limits();
    assert_eq!((l.poll_ms.min, l.poll_ms.max), (200, 60_000));
    let ok = Config {
        poll_ms: l.poll_ms.max,
        close_after_sec: l.close_after_sec.max,
        ..Config::default()
    };
    assert!(e.save_config(ok).is_ok());
    let too_big = Config {
        kill_grace_sec: l.kill_grace_sec.max + 1,
        ..Config::default()
    };
    assert!(e.save_config(too_big).is_err());
}
