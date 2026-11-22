//! Где лежат данные программы: переменная окружения или папка настроек ОС.

use std::ffi::OsString;
use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "ozma-ai-vpn-closer";

/// Переменная окружения, переопределяющая папку данных (запуск на временной папке).
pub const DATA_DIR_ENV: &str = "AI_VPN_CLOSER_DATA_DIR";

/// `$AI_VPN_CLOSER_DATA_DIR`, если задана и не пуста, иначе
/// `dirs::config_dir()/ozma-ai-vpn-closer`.
pub fn default_data_dir() -> Option<PathBuf> {
    data_dir_from(std::env::var_os(DATA_DIR_ENV))
}

fn data_dir_from(env: Option<OsString>) -> Option<PathBuf> {
    match env {
        Some(v) if !v.is_empty() => Some(PathBuf::from(v)),
        _ => dirs::config_dir().map(|d| d.join(APP_DIR_NAME)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_env_override() {
        assert_eq!(
            data_dir_from(Some("/tmp/x".into())),
            Some(PathBuf::from("/tmp/x"))
        );
        let def = data_dir_from(None).unwrap();
        assert!(def.ends_with(APP_DIR_NAME));
        assert_eq!(data_dir_from(Some("".into())), Some(def));
    }
}
