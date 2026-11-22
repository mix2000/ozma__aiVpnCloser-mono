//! Пределы числовых настроек (`get_limits`): UI берёт их отсюда, а не держит свои.

use serde::{Deserialize, Serialize};

/// Допустимый диапазон, включительно.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

impl Range {
    pub fn contains(&self, v: u32) -> bool {
        (self.min..=self.max).contains(&v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub poll_ms: Range,
    pub confirm_failures: Range,
    pub close_after_sec: Range,
    pub kill_grace_sec: Range,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_and_contains() {
        let r = Range { min: 1, max: 20 };
        assert!(r.contains(1) && r.contains(20) && !r.contains(0) && !r.contains(21));
        let l = Limits {
            poll_ms: r,
            confirm_failures: r,
            close_after_sec: r,
            kill_grace_sec: r,
        };
        let v = serde_json::to_value(l).unwrap();
        assert_eq!(v["pollMs"]["max"], 20);
        assert_eq!(v["killGraceSec"]["min"], 1);
    }
}
