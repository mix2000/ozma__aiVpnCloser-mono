//! Команды для UI — ровно по контракту ТЗ (JSON camelCase), по функции на
//! команду, только проброс в `Engine`. Ошибки — строкой по-русски.
//! Блокирующие вызовы ядра (discover, test_targets, save_config,
//! set_monitoring) — в spawn_blocking, чтобы не морозить окно. Значок и
//! автозапуск обновляются из подписок (`bridge`), не здесь.

use tauri::State;
use vpn_closer_core::{AgentInfo, AppState, Config, Discovery, Event, Limits};

use crate::backend::Backend;
use crate::error::AppResult;

/// Сколько событий журнала отдавать, если UI не сказал.
const DEFAULT_EVENTS_LIMIT: usize = 200;

#[tauri::command]
pub fn get_state(backend: State<'_, Backend>) -> AppState {
    backend.engine().get_state()
}

#[tauri::command]
pub async fn discover(backend: State<'_, Backend>) -> AppResult<Discovery> {
    backend.blocking(|e| e.discover()).await
}

#[tauri::command]
pub fn get_config(backend: State<'_, Backend>) -> Config {
    backend.engine().get_config()
}

#[tauri::command]
pub fn get_limits(backend: State<'_, Backend>) -> Limits {
    backend.engine().limits()
}

#[tauri::command]
pub async fn save_config(backend: State<'_, Backend>, config: Config) -> AppResult<Config> {
    Ok(backend.blocking(move |e| e.save_config(config)).await??)
}

#[tauri::command]
pub async fn set_monitoring(backend: State<'_, Backend>, on: bool) -> AppResult<AppState> {
    Ok(backend.blocking(move |e| e.set_monitoring(on)).await??)
}

#[tauri::command]
pub async fn test_targets(backend: State<'_, Backend>) -> AppResult<Vec<AgentInfo>> {
    backend.blocking(|e| e.test_targets()).await
}

#[tauri::command]
pub fn recent_events(backend: State<'_, Backend>, limit: Option<usize>) -> Vec<Event> {
    backend
        .engine()
        .recent_events(limit.unwrap_or(DEFAULT_EVENTS_LIMIT))
}
