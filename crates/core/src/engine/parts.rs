//! Корень композиции: настоящие реализации платформы собираются только здесь.

use std::path::Path;
use std::sync::Arc;

use crate::platform::{
    DnsResolver, NetProbe, OsDns, OsNet, OsOwnerProbe, OsProcessControl, OwnerProbe, ProcSource,
    ProcessControl, SysProcSource,
};
use crate::services::actions::SystemActor;
use crate::services::checks::SystemChecks;
use crate::services::monitor::{Clock, Me, MonitorParts, SystemClock};
use crate::services::owners::OwnerResolver;
use crate::storage::{FrozenFile, FROZEN_FILE};

/// Новый источник снимков процессов (у монитора и у команд — свои).
pub type ProcFactory = Arc<dyn Fn() -> Box<dyn ProcSource> + Send + Sync>;

/// Всё, что ядро берёт у ОС. В тестах — подставное.
#[derive(Clone)]
pub struct PlatformParts {
    pub net: Arc<dyn NetProbe>,
    pub dns: Arc<dyn DnsResolver>,
    pub owners: Arc<dyn OwnerProbe>,
    pub control: Arc<dyn ProcessControl>,
    pub procs: ProcFactory,
    pub clock: Arc<dyn Clock>,
    /// Свой pid: себя и своих потомков не трогаем.
    pub self_pid: u32,
}

impl PlatformParts {
    /// Настоящая ОС.
    pub fn os() -> Self {
        PlatformParts {
            net: Arc::new(OsNet),
            dns: Arc::new(OsDns),
            owners: Arc::new(OsOwnerProbe),
            control: Arc::new(OsProcessControl),
            procs: Arc::new(|| Box::new(SysProcSource::new())),
            clock: Arc::new(SystemClock::default()),
            self_pid: std::process::id(),
        }
    }

    /// Зависимости монитора из этого набора; список замороженных — в `data_dir`.
    pub fn monitor_parts(&self, data_dir: &Path) -> MonitorParts {
        MonitorParts {
            checks: Box::new(SystemChecks::new(
                Arc::clone(&self.net),
                self.owner_resolver(),
                Arc::clone(&self.dns),
            )),
            procs: (self.procs)(),
            actor: Box::new(SystemActor::new(Arc::clone(&self.control), self.self_pid)),
            clock: Arc::clone(&self.clock),
            frozen: Box::new(FrozenFile::new(data_dir.join(FROZEN_FILE))),
            me: Me::Pid(self.self_pid),
        }
    }

    pub fn owner_resolver(&self) -> OwnerResolver {
        OwnerResolver::new(Arc::clone(&self.owners), self.self_pid)
    }
}
