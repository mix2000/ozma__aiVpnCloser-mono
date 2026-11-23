//! Ядро ozma AI VPN Closer: следит за VPN и, если он пропал, замораживает и
//! закрывает сессии агентов (Claude Code, Codex, Grok). Без GUI.
//!
//! Слои (наружу не торчат — публичное API ниже):
//! - [`models`] — доменные типы и DTO контракта (serde);
//! - `services` — бизнес-логика: проверки, агенты, автомат монитора, действия;
//! - `platform` — всё, что зависит от ОС, за trait'ами;
//! - `storage` — файлы: настройки, журнал, атомарная запись;
//! - [`Engine`] — фасад для app и корень композиции.
//!
//! Внутренности для интеграционных тестов и примеров — модуль [`testing`]
//! под фичей `test-util`.

mod engine;
mod error;
mod platform;
mod services;
mod storage;

pub mod models;

pub use engine::{Engine, SubscriptionId};
pub use error::{Error, Result};
pub use models::{
    AgentInfo, AppState, Config, Discovery, Event, EventKind, Limits, Notice, Status, StatusColor,
    TargetKind,
};
pub use services::agents::sessions_text;

/// Внутренности ядра для интеграционных тестов (`tests/`) и примеров.
#[cfg(feature = "test-util")]
pub mod testing {
    pub use crate::engine::{PlatformParts, ProcFactory};
    pub use crate::platform::{
        local_addr_for, snapshot, DnsDone, DnsResolver, NetProbe, OsDns, OsNet, OsOwnerProbe,
        OsProcessControl, OwnerProbe, ProcSource, ProcessControl, SysProcSource,
    };
    pub use crate::services::actions::{apply_with, guard, Actor, SystemActor};
    pub use crate::services::agents::{collect, summary};
    pub use crate::services::checks::{CheckSource, SystemChecks};
    pub use crate::services::discovery::{discover, interfaces};
    pub use crate::services::monitor::{
        Clock, FakeClock, Me, Monitor, MonitorParts, SystemClock, TickOutput,
    };
    pub use crate::services::owners::OwnerResolver;
    pub use crate::storage::{FrozenFile, FrozenStore};
}
