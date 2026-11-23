//! Запись о замороженном процессе — переживает падение программы
//! (`storage::FrozenStore`).

use serde::{Deserialize, Serialize};

use super::process::ProcInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrozenProc {
    pub pid: u32,
    /// Как в [`ProcInfo`]: защита от повторного использования pid.
    pub start_time: u64,
}

impl FrozenProc {
    /// Тот же процесс ещё жив (pid не переиспользован).
    pub fn alive_in(&self, procs: &[ProcInfo]) -> bool {
        procs
            .iter()
            .any(|p| p.pid == self.pid && p.start_time == self.start_time)
    }
}
