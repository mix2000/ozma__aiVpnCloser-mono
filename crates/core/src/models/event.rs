//! Запись журнала событий (`recent_events`). Хранение — `storage::event_log`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Info,
    VpnDown,
    VpnUp,
    Freeze,
    Resume,
    Close,
    Block,
    DryRun,
    Error,
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// RFC 3339.
    pub at: String,
    pub kind: EventKind,
    pub message: String,
}

impl Event {
    /// Событие на момент `at` (RFC 3339; время даёт `Clock` вызывающего).
    pub fn new(at: impl Into<String>, kind: EventKind, message: impl Into<String>) -> Event {
        Event {
            at: at.into(),
            kind,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_snake_case() {
        let v = serde_json::to_value(Event::new("2026-11-22T10:00:00Z", EventKind::DryRun, "x"))
            .unwrap();
        assert_eq!(v["kind"], "dry_run");
        assert_eq!(v["at"], "2026-11-22T10:00:00Z");
    }
}
