//! Снимок процессов через sysinfo (одинаково на всех ОС).

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::ProcSource;
use crate::models::ProcInfo;

/// Держит `System`, чтобы повторные снимки были дешёвыми.
pub struct SysProcSource {
    sys: Option<System>,
}

impl Default for SysProcSource {
    fn default() -> Self {
        Self::new()
    }
}

impl SysProcSource {
    pub fn new() -> Self {
        SysProcSource {
            sys: System::new().ok(),
        }
    }
}

impl ProcSource for SysProcSource {
    fn snapshot(&mut self) -> Vec<ProcInfo> {
        let Some(sys) = self.sys.as_mut() else {
            return Vec::new();
        };
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::OnlyIfNotSet)
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_user(UpdateKind::OnlyIfNotSet),
        );
        sys.processes()
            .iter()
            .map(|(pid, p)| ProcInfo {
                pid: pid.as_u32(),
                ppid: p.parent().map(|pp| pp.as_u32()),
                uid: p.user_id().map(|u| u.to_string()),
                name: p.name().to_string_lossy().into_owned(),
                exe: p.exe().map(|e| e.to_string_lossy().into_owned()),
                cmd: p
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect(),
                start_time: p.start_time(),
            })
            .collect()
    }
}

/// Разовый снимок.
#[cfg(any(test, feature = "test-util"))]
pub fn snapshot() -> Vec<ProcInfo> {
    SysProcSource::new().snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_contains_self() {
        let me = std::process::id();
        let snap = snapshot();
        let mine = snap.iter().find(|p| p.pid == me).expect("свой процесс");
        assert!(mine.start_time > 0);
    }
}
