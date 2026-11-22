//! Доменные типы и DTO контракта (serde, JSON camelCase). Без ввода-вывода.

mod action;
mod agent;
mod check;
mod config;
mod discovery;
mod event;
mod limits;
mod process;
mod state;

pub use action::{Action, ActionReport};
pub use agent::{AgentProc, SelfCtx, TargetKind};
pub use check::{CheckResult, Verdict};
pub use config::{Config, Mode, Targets, WatchItem, DEFAULT_HOSTS};
pub use discovery::{
    Discovery, IfAddr, IfOwner, NetInterface, OwnerConfidence, SystemVpn, VpnProcess,
};
pub use event::{Event, EventKind};
pub use limits::{Limits, Range};
pub use process::{base_name, ProcInfo};
pub use state::{AgentInfo, AppState, Notice, Status, StatusColor};
