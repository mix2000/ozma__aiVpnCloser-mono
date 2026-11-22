//! Проверки пунктов `WatchItem`: включён ли VPN.
//!
//! - `rules` — чистые проверки по готовым данным и правило all/any;
//! - `resolver` + `route` — проверка маршрута к хостам агентов;
//! - `system` — настоящий источник поверх `platform::NetProbe`.

mod resolver;
mod route;
mod rules;
mod system;

pub use rules::combine;
pub use system::SystemChecks;

use crate::models::{CheckResult, Config, ProcInfo};

/// Источник проверок (в тестах — подставной).
pub trait CheckSource: Send {
    fn check(&mut self, cfg: &Config, procs: &[ProcInfo]) -> Vec<CheckResult>;
}
