//! Ядро (`Engine`) в общем состоянии Tauri.

use std::sync::Arc;

use vpn_closer_core::Engine;

use crate::error::{AppError, AppResult};
use crate::status::StateView;

/// Дешёвая копия ручки на ядро: блокирующие вызовы уходят в spawn_blocking.
#[derive(Clone)]
pub struct Backend(pub Arc<Engine>);

impl Backend {
    pub fn engine(&self) -> &Engine {
        &self.0
    }

    pub fn state(&self) -> StateView {
        StateView::from_state(self.0.get_state())
    }

    /// Выполнить блокирующий вызов ядра вне главного потока.
    pub async fn blocking<T, F>(&self, f: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&Engine) -> T + Send + 'static,
    {
        let engine = self.0.clone();
        tauri::async_runtime::spawn_blocking(move || f(&engine))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}
