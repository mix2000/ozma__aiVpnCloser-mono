//! Уведомления ОС. Ядро уже отфильтровало их по настройке `notify`.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use vpn_closer_core::Notice;

pub fn show(app: &AppHandle, n: &Notice) {
    if let Err(e) = app
        .notification()
        .builder()
        .title(&n.title)
        .body(&n.body)
        .show()
    {
        log::warn!("notification: {e}");
    }
}
