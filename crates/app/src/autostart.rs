//! Автозапуск при входе в систему (tauri-plugin-autostart) — по настройке
//! `autostart`, из подписки на сохранённые настройки (`bridge`).

use tauri::AppHandle;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// Аргумент, с которым ОС запускает программу при входе: окно не показываем.
pub const HIDDEN_ARG: &str = "--hidden";

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![HIDDEN_ARG]))
}

/// Запущены ли мы автозапуском (окно не показывать).
pub fn started_hidden(args: impl IntoIterator<Item = String>) -> bool {
    args.into_iter().any(|a| a == HIDDEN_ARG)
}

/// Что сделать с автозапуском ОС: `Some(on)` — включить/выключить, `None` — ничего.
pub fn change(enabled_now: bool, want: bool) -> Option<bool> {
    (enabled_now != want).then_some(want)
}

/// Синхронизировать автозапуск ОС с настройкой.
pub fn apply(app: &AppHandle, want: bool) {
    let al = app.autolaunch();
    let Some(on) = change(al.is_enabled().unwrap_or(false), want) else {
        return;
    };
    let r = if on { al.enable() } else { al.disable() };
    if let Err(e) = r {
        log::warn!("autostart: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_only_when_different() {
        assert_eq!(change(false, true), Some(true));
        assert_eq!(change(true, false), Some(false));
        assert_eq!(change(true, true), None);
        assert_eq!(change(false, false), None);
    }

    #[test]
    fn hidden_arg() {
        assert!(started_hidden(["app".to_string(), HIDDEN_ARG.to_string()]));
        assert!(!started_hidden(["app".to_string()]));
    }
}
