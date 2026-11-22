//! Что записать в журнал, когда пользователь поменял важные настройки.

use crate::models::{Config, EventKind};

/// События о смене «Следить» и «Только смотреть».
pub fn config_change_events(old: &Config, new: &Config) -> Vec<(EventKind, &'static str)> {
    let mut out = Vec::new();
    if old.monitoring != new.monitoring {
        out.push((
            EventKind::Config,
            if new.monitoring {
                "Наблюдение включено"
            } else {
                "Наблюдение выключено"
            },
        ));
    }
    if old.dry_run != new.dry_run {
        out.push((
            EventKind::Config,
            if new.dry_run {
                "Включён режим «только смотреть»"
            } else {
                "Режим «только смотреть» выключен — действия настоящие"
            },
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_monitoring_and_dry_run() {
        let a = Config::default();
        assert!(config_change_events(&a, &a).is_empty());
        let b = Config {
            monitoring: true,
            dry_run: false,
            poll_ms: 500,
            ..Config::default()
        };
        let texts: Vec<&str> = config_change_events(&a, &b).iter().map(|e| e.1).collect();
        assert_eq!(
            texts,
            [
                "Наблюдение включено",
                "Режим «только смотреть» выключен — действия настоящие"
            ]
        );
        assert_eq!(config_change_events(&b, &a)[0].1, "Наблюдение выключено");
    }
}
