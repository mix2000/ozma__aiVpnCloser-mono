//! Связка ядра с оболочкой: состояние → событие `state-changed` для UI и
//! значок трея; уведомления ядра → уведомления ОС; сохранённые настройки →
//! автозапуск ОС.

use tauri::{AppHandle, Emitter};
use vpn_closer_core::{AppState, Engine};

use crate::status::StateView;
use crate::{autostart, notifications, tray};

/// Событие, которое слушает UI (`ui/src/entities/monitor/monitor.api.ts`).
pub const STATE_EVENT: &str = "state-changed";

/// Подписать оболочку на ядро.
pub fn connect(engine: &Engine, app: &AppHandle) {
    let h = app.clone();
    engine.subscribe(move |s| on_state(&h, s));
    let h = app.clone();
    engine.subscribe_notices(move |n| notifications::show(&h, n));
    let h = app.clone();
    engine.subscribe_config(move |c| autostart::apply(&h, c.autostart));
}

fn on_state(app: &AppHandle, s: &AppState) {
    if let Err(e) = app.emit(STATE_EVENT, s) {
        log::warn!("emit {STATE_EVENT}: {e}");
    }
    tray::update(app, &StateView::from_state(s.clone()));
}
