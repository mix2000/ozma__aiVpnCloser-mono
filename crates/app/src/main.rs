//! ozma AI VPN Closer — окно и значок в трее/меню-баре поверх `vpn-closer-core`.
//!
//! Здесь только сборка и запуск: плагины, команды, подписки, трей. Логика —
//! в ядре (`Engine`), оболочка — в соседних модулях.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod backend;
mod bridge;
mod commands;
mod error;
mod logger;
mod notifications;
mod platform;
mod status;
mod tray;
mod window;

use std::sync::Arc;

use tauri::{App, Manager, RunEvent, WindowEvent, Wry};
use vpn_closer_core::Engine;

use backend::Backend;
use platform::signals;

fn main() {
    logger::init();
    signals::block();
    let start_hidden = autostart::started_hidden(std::env::args());
    let app = builder()
        .setup(move |app| setup(app, start_hidden))
        .build(tauri::generate_context!())
        .expect("не удалось запустить окно");
    app.run(on_run_event);
}

/// Плагины, команды, закрытие окна = спрятать в трей.
fn builder() -> tauri::Builder<Wry> {
    tauri::Builder::default()
        // Второй запуск — просто показать окно первого.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app)
        }))
        .plugin(autostart::plugin())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::discover,
            commands::get_config,
            commands::get_limits,
            commands::save_config,
            commands::set_monitoring,
            commands::test_targets,
            commands::recent_events,
        ])
        .on_window_event(|w, event| {
            // Закрыть окно = спрятать в трей. Выход — из меню значка.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window::hide(w.app_handle());
            }
        })
}

/// Ядро, подписки, трей, автозапуск, монитор, окно.
fn setup(app: &mut App, start_hidden: bool) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    signals::attach(&handle);
    // Папка данных: AI_VPN_CLOSER_DATA_DIR или папка ОС.
    let engine = Arc::new(Engine::with_default_dir()?);
    bridge::connect(&engine, &handle);
    let backend = Backend(engine.clone());
    app.manage(backend.clone());
    tray::create(&handle)?;
    tray::update(&handle, &backend.state());
    autostart::apply(&handle, engine.get_config().autostart);
    engine.start();
    if start_hidden {
        window::hide(&handle);
    } else {
        window::show(&handle);
    }
    Ok(())
}

fn on_run_event(app: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::Exit => {
            // Разморозить замороженных агентов и остановить монитор.
            if let Some(b) = app.try_state::<Backend>() {
                b.engine().shutdown();
            }
        }
        e if platform::is_reopen(&e) => window::show(app),
        _ => {}
    }
}
