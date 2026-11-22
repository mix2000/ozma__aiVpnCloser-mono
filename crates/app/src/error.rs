//! Ошибки команд. В UI уходят строкой по-русски — как и раньше.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] vpn_closer_core::Error),
    /// Блокирующая задача не завершилась (паника, отмена).
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_as_plain_string() {
        let e = AppError::from(vpn_closer_core::Error::NoDataDir);
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!("Не найдена папка настроек ОС")
        );
        let e = AppError::Internal("panic".into());
        assert_eq!(e.to_string(), "Внутренняя ошибка: panic");
    }
}
