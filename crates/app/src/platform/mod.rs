//! Различия ОС в оболочке: сигналы завершения, значок в Dock, клики по трею.
//! `#[cfg(target_os)]` — только здесь.

pub mod signals;

use tauri::{AppHandle, RunEvent};

/// Меню значка по левому клику: на macOS — как у всех значков меню-бара;
/// на Windows левый клик открывает окно, правый — меню.
pub const MENU_ON_LEFT_CLICK: bool = cfg!(target_os = "macos");

/// Показать или убрать значок в Dock (только macOS).
pub fn set_dock_visible(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

/// Клик по значку в Dock при скрытом окне (только macOS).
pub fn is_reopen(event: &RunEvent) -> bool {
    #[cfg(target_os = "macos")]
    {
        matches!(event, RunEvent::Reopen { .. })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = event;
        false
    }
}
