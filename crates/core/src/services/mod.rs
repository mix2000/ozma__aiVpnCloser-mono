//! Бизнес-логика: чистые функции и автоматы, проверяются без ОС (ОС — через
//! trait'ы `platform`).

pub mod actions;
pub mod agents;
pub mod checks;
pub mod config_changes;
pub mod config_validation;
pub mod discovery;
pub mod monitor;
pub mod net;
pub mod owners;
pub mod vpn_processes;
