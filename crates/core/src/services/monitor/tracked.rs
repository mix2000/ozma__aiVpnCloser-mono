//! Процессы, над которыми монитор что-то сделал (заморозил, закрыл).

use crate::models::{AgentProc, ProcInfo, TargetKind};

/// Процесс, над которым мы что-то сделали.
#[derive(Debug, Clone)]
pub struct Tracked {
    pub pid: u32,
    pub start_time: u64,
    pub target: TargetKind,
    pub name: String,
    pub root: u32,
    pub depth: u32,
    /// false — в режиме «только смотреть» (ничего не послано).
    pub real: bool,
}

impl Tracked {
    pub fn from(a: &AgentProc, real: bool) -> Tracked {
        Tracked {
            pid: a.pid,
            start_time: a.start_time,
            target: a.target,
            name: a.name.clone(),
            root: a.root,
            depth: a.depth,
            real,
        }
    }

    pub fn as_agent(&self) -> AgentProc {
        AgentProc {
            target: self.target,
            pid: self.pid,
            name: self.name.clone(),
            start_time: self.start_time,
            root: self.root,
            depth: self.depth,
        }
    }

    /// Тот же процесс ещё жив (pid не переиспользован).
    pub fn alive_in(&self, procs: &[ProcInfo]) -> bool {
        procs
            .iter()
            .any(|p| p.pid == self.pid && p.start_time == self.start_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alive_only_with_same_start_time() {
        let a = AgentProc {
            target: TargetKind::Grok,
            pid: 7,
            name: "grok".into(),
            start_time: 100,
            root: 7,
            depth: 0,
        };
        let t = Tracked::from(&a, true);
        assert_eq!(t.as_agent(), a);
        let p = |start_time| ProcInfo {
            pid: 7,
            start_time,
            ..Default::default()
        };
        assert!(t.alive_in(&[p(100)]));
        assert!(!t.alive_in(&[p(101)]), "pid переиспользован");
        assert!(!t.alive_in(&[]));
    }
}
