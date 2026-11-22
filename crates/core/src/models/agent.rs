//! Цели (виды агентов) и процессы, которые попадут под действие.

use serde::{Deserialize, Serialize};

use super::config::Targets;
use super::process::ProcInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetKind {
    ClaudeCode,
    ClaudeDesktop,
    CodexCli,
    CodexDesktop,
    Grok,
}

impl TargetKind {
    pub const ALL: [TargetKind; 5] = [
        TargetKind::ClaudeCode,
        TargetKind::ClaudeDesktop,
        TargetKind::CodexCli,
        TargetKind::CodexDesktop,
        TargetKind::Grok,
    ];

    /// Короткое имя для уведомлений: «claude ×3».
    pub fn short(&self) -> &'static str {
        match self {
            TargetKind::ClaudeCode => "claude",
            TargetKind::ClaudeDesktop => "Claude Desktop",
            TargetKind::CodexCli => "codex",
            TargetKind::CodexDesktop => "Codex Desktop",
            TargetKind::Grok => "grok",
        }
    }

    pub fn enabled_in(&self, t: &Targets) -> bool {
        match self {
            TargetKind::ClaudeCode => t.claude_code,
            TargetKind::ClaudeDesktop => t.claude_desktop,
            TargetKind::CodexCli => t.codex_cli,
            TargetKind::CodexDesktop => t.codex_desktop,
            TargetKind::Grok => t.grok,
        }
    }
}

/// Процесс, который попадёт под действие.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProc {
    pub target: TargetKind,
    pub pid: u32,
    pub name: String,
    pub start_time: u64,
    /// pid корня дерева (самого агента).
    pub root: u32,
    /// Глубина в дереве: 0 — сам агент.
    pub depth: u32,
}

impl AgentProc {
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }
}

/// Кто мы: свой pid и пользователь — себя, своих потомков и чужих не трогаем.
#[derive(Debug, Clone, Default)]
pub struct SelfCtx {
    pub pid: u32,
    pub uid: Option<String>,
}

impl SelfCtx {
    /// Процесс `pid` (обычно — свой); пользователь берётся из снимка.
    pub fn for_pid(pid: u32, procs: &[ProcInfo]) -> SelfCtx {
        SelfCtx {
            pid,
            uid: procs
                .iter()
                .find(|p| p.pid == pid)
                .and_then(|p| p.uid.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_ctx_user_from_snapshot() {
        let procs = vec![ProcInfo {
            pid: 7,
            uid: Some("501".into()),
            ..Default::default()
        }];
        assert_eq!(SelfCtx::for_pid(7, &procs).uid.as_deref(), Some("501"));
        assert_eq!(SelfCtx::for_pid(8, &procs).uid, None);
    }

    #[test]
    fn target_kind_json_and_flags() {
        assert_eq!(
            serde_json::to_value(TargetKind::CodexDesktop).unwrap(),
            "codexDesktop"
        );
        let t = Targets {
            grok: false,
            ..Targets::default()
        };
        assert!(!TargetKind::Grok.enabled_in(&t));
        assert!(TargetKind::ALL
            .iter()
            .filter(|k| **k != TargetKind::Grok)
            .all(|k| k.enabled_in(&t)));
    }
}
