//! Значок в трее (Windows) / меню-баре (macOS): цвет статуса и короткое меню.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::backend::Backend;
use vpn_closer_core::StatusColor;

use crate::status::StateView;
use crate::{platform, window};

const TRAY_ID: &str = "main";

const ICON_GREEN: &[u8] = include_bytes!("../icons/tray/tray-green.png");
const ICON_YELLOW: &[u8] = include_bytes!("../icons/tray/tray-yellow.png");
const ICON_RED: &[u8] = include_bytes!("../icons/tray/tray-red.png");
const ICON_GRAY: &[u8] = include_bytes!("../icons/tray/tray-gray.png");

/// Что делать по пункту меню.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    ToggleWatch,
    Quit,
}

impl MenuAction {
    pub fn from_id(id: &str) -> Option<MenuAction> {
        match id {
            "open" => Some(MenuAction::Open),
            "watch" => Some(MenuAction::ToggleWatch),
            "quit" => Some(MenuAction::Quit),
            _ => None,
        }
    }
}

/// Пункты меню, которые меняются вместе со статусом.
pub struct TrayItems {
    status: MenuItem<Wry>,
    watch: CheckMenuItem<Wry>,
}

fn icon(color: StatusColor) -> tauri::Result<Image<'static>> {
    let bytes = match color {
        StatusColor::Green => ICON_GREEN,
        StatusColor::Yellow => ICON_YELLOW,
        StatusColor::Red => ICON_RED,
        StatusColor::Gray => ICON_GRAY,
    };
    Image::from_bytes(bytes)
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Запуск…", false, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Открыть", true, None::<&str>)?;
    let watch = CheckMenuItem::with_id(app, "watch", "Следить", true, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &watch,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(StatusColor::Gray)?)
        .tooltip("ozma AI VPN Closer")
        .menu(&menu)
        .show_menu_on_left_click(platform::MENU_ON_LEFT_CLICK)
        .on_menu_event(|app, event| match MenuAction::from_id(event.id.as_ref()) {
            Some(MenuAction::Open) => window::show(app),
            Some(MenuAction::ToggleWatch) => toggle_monitoring(app),
            Some(MenuAction::Quit) => app.exit(0),
            None => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if !platform::MENU_ON_LEFT_CLICK {
                    window::show(tray.app_handle());
                }
            }
        })
        .build(app)?;

    app.manage(TrayItems { status, watch });
    Ok(())
}

fn toggle_monitoring(app: &AppHandle) {
    let Some(backend) = app.try_state::<Backend>() else {
        return;
    };
    let backend = backend.inner().clone();
    // Ядро переключает атомарно и само разошлёт новое состояние (bridge обновит
    // значок); set_monitoring ждёт шаг монитора — не в главном потоке.
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = backend.engine().toggle_monitoring() {
            log::warn!("toggle_monitoring: {e}");
        }
    });
}

/// Обновить значок, подсказку и меню по состоянию.
pub fn update(app: &AppHandle, s: &StateView) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(img) = icon(s.color) {
            let _ = tray.set_icon(Some(img));
        }
        let _ = tray.set_tooltip(Some(format!("ozma AI VPN Closer — {}", s.tray_text)));
    }
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.status.set_text(&s.tray_text);
        let _ = items.watch.set_checked(s.state.monitoring);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids() {
        assert_eq!(MenuAction::from_id("open"), Some(MenuAction::Open));
        assert_eq!(MenuAction::from_id("watch"), Some(MenuAction::ToggleWatch));
        assert_eq!(MenuAction::from_id("quit"), Some(MenuAction::Quit));
        assert_eq!(MenuAction::from_id("status"), None);
    }

    #[test]
    fn icons_decode() {
        for c in [
            StatusColor::Green,
            StatusColor::Yellow,
            StatusColor::Red,
            StatusColor::Gray,
        ] {
            assert!(icon(c).is_ok());
        }
    }
}
