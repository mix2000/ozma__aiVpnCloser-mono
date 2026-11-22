//! Результаты проверок пунктов `WatchItem`.

use serde::{Deserialize, Serialize};

use super::config::WatchItem;
use super::discovery::IfOwner;

/// Результат по одному пункту (поле `checks` в `AppState`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub item: WatchItem,
    /// true — ок, false — нет, null — пока неизвестно.
    pub ok: Option<bool>,
    /// Название пункта для окна: «Программа xray», «Интерфейс utun233 (…)».
    pub label: String,
    pub detail: String,
    /// Для пункта `interface` — программа-владелец найденного туннеля.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<IfOwner>,
}

impl CheckResult {
    /// Результат по пункту; название берётся из пункта.
    pub fn new(item: WatchItem, ok: Option<bool>, detail: impl Into<String>) -> CheckResult {
        CheckResult {
            label: item.label(),
            item,
            ok,
            detail: detail.into(),
            owner: None,
        }
    }

    pub fn with_owner(mut self, owner: Option<IfOwner>) -> CheckResult {
        self.owner = owner;
        self
    }
}

/// Итог по всем пунктам с учётом правила all/any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Up,
    Down,
    Unknown,
}
