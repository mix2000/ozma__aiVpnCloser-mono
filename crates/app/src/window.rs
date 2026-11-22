//! Главное окно: закрыть = спрятать в трей, показать — из трея или вторым запуском.

use tauri::{AppHandle, Manager};

use crate::platform;

const MAIN_WINDOW: &str = "main";

/// Показать окно (и вернуть значок в Dock на macOS).
pub fn show(app: &AppHandle) {
    platform::set_dock_visible(app, true);
    if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Спрятать окно в трей; на macOS — убрать значок из Dock.
pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
        let _ = w.hide();
    }
    platform::set_dock_visible(app, false);
}
