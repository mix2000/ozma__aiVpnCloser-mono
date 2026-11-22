//! Какие процессы — агенты (цели), их деревья потомков и тексты о них.
//!
//! Всё здесь — чистые функции над снимком процессов, без системных вызовов.

mod classify;
mod summary;
mod tree;

pub use summary::{procs_text, sessions_text, summary};
pub use tree::collect;

#[cfg(test)]
pub(crate) mod tests {
    use crate::models::{ProcInfo, SelfCtx};

    pub(crate) fn p(pid: u32, ppid: u32, name: &str, cmd: &[&str]) -> ProcInfo {
        ProcInfo {
            pid,
            ppid: Some(ppid),
            uid: Some("501".into()),
            name: name.into(),
            exe: None,
            cmd: cmd.iter().map(|s| s.to_string()).collect(),
            start_time: 1000 + pid as u64,
        }
    }

    pub(crate) fn with_exe(mut pi: ProcInfo, exe: &str) -> ProcInfo {
        pi.exe = Some(exe.into());
        pi
    }

    pub(crate) fn sample() -> Vec<ProcInfo> {
        let mut v = vec![
            p(1, 0, "launchd", &["/sbin/launchd"]),
            p(100, 1, "tmux", &["tmux", "new-session", "claude"]),
            p(200, 100, "claude", &["claude", "--continue"]),
            p(201, 200, "node", &["node", "/x/mcp-server.js"]),
            p(202, 200, "zsh", &["/bin/zsh", "-c", "sleep 100"]),
            p(203, 202, "sleep", &["sleep", "100"]),
            p(300, 100, "codex", &["codex"]),
            p(400, 1, "grok", &["grok"]),
            p(500, 100, "echo", &["echo", "claude"]),
            p(600, 1, "claude", &["claude"]), // чужой пользователь
            p(700, 200, "vpn-closer", &["vpn-closer"]), // это мы — внутри дерева claude
            p(701, 700, "claude", &["claude"]), // наш потомок
            p(
                800,
                1,
                "Claude",
                &["/Applications/Claude.app/Contents/MacOS/Claude"],
            ),
            p(801, 800, "claude", &["claude"]), // Claude Code внутри Desktop
        ];
        v[9].uid = Some("0".into());
        v
    }

    pub(crate) fn me() -> SelfCtx {
        SelfCtx {
            pid: 700,
            uid: Some("501".into()),
        }
    }
}
