//! Файлы данных: папка приложения, атомарная запись, настройки, журнал.

mod atomic;
mod config_store;
mod event_log;

use std::path::Path;

use crate::error::{Error, Result};

pub use config_store::{load_config, save_config, set_aside};
pub use event_log::EventLog;

pub const CONFIG_FILE: &str = "config.json";
pub const EVENTS_FILE: &str = "events.jsonl";

/// Создать папку данных, если её нет.
pub fn ensure_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|source| Error::DataDir {
        path: dir.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_dir_creates_nested() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a/b");
        ensure_dir(&p).unwrap();
        assert!(p.is_dir());
        let file = dir.path().join("f");
        std::fs::write(&file, "x").unwrap();
        let err = ensure_dir(&file.join("x")).unwrap_err().to_string();
        assert!(err.starts_with("Не удалось создать папку данных"), "{err}");
    }
}
