//! Всё, что зависит от ОС, — за trait'ами. `#[cfg(target_os)]` — только здесь.
//!
//! - [`NetProbe`] — сетевые интерфейсы, системные VPN, выбор исходного адреса;
//! - [`DnsResolver`] — резолв имени в фоне;
//! - [`ProcessControl`] — сигналы / NtSuspendProcess по одному pid;
//! - [`ProcSource`] — снимок процессов;
//! - [`OwnerProbe`] — откуда узнать программу-владельца туннеля (только чтение).
//!
//! Сервисы получают их параметрами и проверяются подменой; настоящие (`os`)
//! собирает только корень композиции — `Engine`.

use std::io;
use std::net::IpAddr;
use std::path::Path;

use crate::models::{Action, NetInterface, ProcInfo, SystemVpn};

mod os;
pub mod parse;
pub mod paths;
mod procs;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(feature = "test-util")]
pub use os::local_addr_for;
pub use os::{OsDns, OsNet, OsOwnerProbe, OsProcessControl};
#[cfg(feature = "test-util")]
pub use procs::snapshot;
pub use procs::SysProcSource;

/// Сеть машины.
pub trait NetProbe: Send + Sync {
    /// Интерфейсы ОС как есть (без фильтрации link-local и сортировки).
    fn interfaces(&self) -> Vec<NetInterface>;
    /// Системные VPN-сервисы (macOS scutil / Windows RAS).
    fn system_vpns(&self) -> Vec<SystemVpn>;
    /// Какой локальный адрес ОС выбрала бы для пакета на `ip:443`.
    fn local_addr_for(&self, ip: IpAddr) -> io::Result<IpAddr>;
}

/// Ответ резолва: адреса или ошибка.
pub type DnsDone = Box<dyn FnOnce(io::Result<Vec<IpAddr>>) + Send>;

/// Резолв имени хоста, не блокируя вызывающего: ответ придёт в `done`.
pub trait DnsResolver: Send + Sync {
    fn resolve_in_background(&self, host: String, done: DnsDone);
}

/// Действие над одним процессом. Проверку pid делает вызывающий
/// (`services::actions`).
pub trait ProcessControl: Send + Sync {
    fn apply(&self, action: Action, pid: u32) -> io::Result<()>;
    /// Жив ли процесс (без сигнала).
    fn is_alive(&self, pid: u32) -> bool;
}

/// Источник снимков процессов (в тестах — подставной).
pub trait ProcSource: Send {
    fn snapshot(&mut self) -> Vec<ProcInfo>;
}

/// Источники для определения владельца туннеля. Всё — только чтение, без sudo.
pub trait OwnerProbe: Send + Sync {
    /// Kernel control `utun_control` у процессов: (pid, номер utun). Только macOS,
    /// только процессы своего пользователя; на прочих ОС — пусто.
    fn utun_units(&self, pids: &[u32]) -> Vec<(u32, u32)>;
    /// Текст конфига VPN-клиента (до 1 МБ); нет доступа — None.
    fn read_config(&self, path: &Path) -> Option<String>;
    /// Интерфейс подключённого системного VPN (macOS `scutil --nc status`).
    fn service_interface(&self, id: &str) -> Option<String>;
    /// Аргументы запуска, если снимок процессов их не дал (чужой процесс).
    fn process_args(&self, _pid: u32) -> Option<Vec<String>> {
        None
    }
}

// ------------------------------------------------------------ выбор реализации

#[cfg(unix)]
use unix::{apply as os_apply, interfaces as os_interfaces, is_alive as os_is_alive};
#[cfg(windows)]
use windows::{
    apply as os_apply, interfaces as os_interfaces, is_alive as os_is_alive,
    system_vpns as os_system_vpns,
};

#[cfg(target_os = "macos")]
use macos::{
    process_args as os_process_args, service_interface as os_service_interface,
    system_vpns as os_system_vpns, utun_units as os_utun_units,
};

/// На прочих Unix системных VPN не ищем.
#[cfg(all(unix, not(target_os = "macos")))]
fn os_system_vpns() -> Vec<SystemVpn> {
    Vec::new()
}

/// utun_control и scutil есть только на macOS.
#[cfg(not(target_os = "macos"))]
fn os_utun_units(_pids: &[u32]) -> Vec<(u32, u32)> {
    Vec::new()
}

#[cfg(not(target_os = "macos"))]
fn os_service_interface(_id: &str) -> Option<String> {
    None
}

/// На прочих ОС аргументы даёт снимок процессов (sysinfo).
#[cfg(not(target_os = "macos"))]
fn os_process_args(_pid: u32) -> Option<Vec<String>> {
    None
}
