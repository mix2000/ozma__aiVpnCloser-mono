//! Настройки в `config.json`.

use std::path::{Path, PathBuf};

use super::atomic::write_atomic;
use crate::error::{Error, Result};
use crate::models::Config;

/// Прочитать (проверка — у вызывающего: `services::config_validation`); нет
/// файла — значения по умолчанию. Битый файл — ошибка (не затираем его молча).
pub fn load_config(path: &Path) -> Result<Config> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|source| Error::ConfigParse {
            path: path.to_path_buf(),
            source,
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(source) => Err(Error::ConfigRead {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Отложить битый конфиг в `<имя>.bad` рядом; вернуть новый путь.
pub fn set_aside(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let bad = path.with_file_name(format!("{name}.bad"));
    let _ = std::fs::rename(path, &bad);
    bad
}

/// Записать атомарно (tmp + rename).
pub fn save_config(cfg: &Config, path: &Path) -> Result<()> {
    let text = serde_json::to_string_pretty(cfg).map_err(Error::ConfigEncode)?;
    write_atomic(path, text.as_bytes()).map_err(|source| Error::ConfigWrite {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_save_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("config.json");
        assert_eq!(load_config(&path).unwrap(), Config::default());
        let c = Config {
            monitoring: true,
            close_after_sec: 0,
            ..Config::default()
        };
        save_config(&c, &path).unwrap();
        assert_eq!(load_config(&path).unwrap(), c);
        std::fs::write(&path, "{").unwrap();
        let bad = set_aside(&path);
        assert!(bad.ends_with("config.json.bad") && bad.exists() && !path.exists());
        std::fs::write(&path, "{").unwrap();
        let err = load_config(&path).unwrap_err().to_string();
        assert!(err.starts_with("Не удалось прочитать настройки"), "{err}");
    }
}
