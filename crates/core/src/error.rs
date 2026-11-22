//! Ошибки ядра. Текст (`Display`) — по-русски, UI показывает его как есть.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Настройки не прошли проверку (`save_config`).
    #[error("{0}")]
    Invalid(String),

    #[error("Не удалось прочитать настройки {}: {source}", path.display())]
    ConfigParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("Не удалось открыть настройки {}: {source}", path.display())]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Не удалось сохранить настройки: {0}")]
    ConfigEncode(serde_json::Error),

    #[error("Не удалось сохранить настройки {}: {source}", path.display())]
    ConfigWrite {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Не удалось создать папку данных {}: {source}", path.display())]
    DataDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Не найдена папка настроек ОС")]
    NoDataDir,
}

impl Error {
    pub(crate) fn invalid(msg: impl Into<String>) -> Error {
        Error::Invalid(msg.into())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_in_russian() {
        assert_eq!(Error::invalid("Плохо").to_string(), "Плохо");
        assert_eq!(Error::NoDataDir.to_string(), "Не найдена папка настроек ОС");
        let e = Error::DataDir {
            path: PathBuf::from("/x"),
            source: std::io::Error::other("нет прав"),
        };
        assert_eq!(
            e.to_string(),
            "Не удалось создать папку данных /x: нет прав"
        );
    }
}
