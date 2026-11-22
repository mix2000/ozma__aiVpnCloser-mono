//! Правила сопоставления: агент ли процесс и какой.
//!
//! Смотрим на исполняемый файл / argv[0] и на скрипт node-подобного рантайма.
//! Слово в остальных аргументах (`echo claude`, `grep codex`, `vim claude.md`)
//! не считается.

use crate::models::{base_name, ProcInfo, TargetKind};

/// Рантаймы, которые запускают агента скриптом (`node …/claude`).
const RUNTIMES: &[&str] = &["node", "nodejs", "bun", "deno", "tsx", "npx"];

/// Пути приложений: (подстрока пути в нижнем регистре, цель).
const APP_PATHS: &[(&str, TargetKind)] = &[
    ("/claude.app/", TargetKind::ClaudeDesktop),
    ("/anthropicclaude/", TargetKind::ClaudeDesktop),
    ("/windowsapps/claude_", TargetKind::ClaudeDesktop),
    ("/windowsapps/anthropic.claude", TargetKind::ClaudeDesktop),
    ("/codex.app/", TargetKind::CodexDesktop),
    ("/windowsapps/openai.codex", TargetKind::CodexDesktop),
];

/// Пакеты npm, по пути скрипта: (подстрока, цель).
const SCRIPT_PACKAGES: &[(&str, TargetKind)] = &[
    ("/@anthropic-ai/claude-code/", TargetKind::ClaudeCode),
    ("/@openai/codex/", TargetKind::CodexCli),
    ("/grok-cli/", TargetKind::Grok),
];

/// Определить, агент ли это.
pub fn classify(p: &ProcInfo) -> Option<TargetKind> {
    let path = lower_path(p);
    if let Some((_, k)) = APP_PATHS.iter().find(|(sub, _)| path.contains(sub)) {
        return Some(*k);
    }

    let bases = exe_bases(p);
    if let Some(k) = bases.iter().find_map(|b| kind_by_base(b)) {
        return Some(k);
    }

    if bases.iter().any(|b| RUNTIMES.contains(&b.as_str())) {
        return classify_script(runtime_script(p)?);
    }
    None
}

/// Цель по скрипту рантайма: пакет npm или имя файла (`claude`, `codex.js`).
fn classify_script(script: &str) -> Option<TargetKind> {
    let sl = script.replace('\\', "/").to_lowercase();
    if let Some((_, k)) = SCRIPT_PACKAGES.iter().find(|(sub, _)| sl.contains(sub)) {
        return Some(*k);
    }
    let sb = base_name(&sl);
    kind_by_base(strip_script_ext(&sb))
}

/// Имена исполняемого файла: exe, argv[0], имя процесса.
fn exe_bases(p: &ProcInfo) -> Vec<String> {
    let mut bases: Vec<String> = Vec::new();
    if let Some(e) = &p.exe {
        bases.push(base_name(e));
    }
    if let Some(a0) = p.cmd.first() {
        bases.push(base_name(a0));
    }
    bases.push(base_name(&p.name));
    bases
}

fn lower_path(p: &ProcInfo) -> String {
    let mut s = p.exe.clone().unwrap_or_default();
    if let Some(a0) = p.cmd.first() {
        s.push('\n');
        s.push_str(a0);
    }
    s.replace('\\', "/").to_lowercase()
}

fn strip_script_ext(s: &str) -> &str {
    for ext in [".js", ".mjs", ".cjs", ".ts"] {
        if let Some(r) = s.strip_suffix(ext) {
            return r;
        }
    }
    s
}

/// Скрипт, который запускает node/bun/deno: первый аргумент не-флаг.
fn runtime_script(p: &ProcInfo) -> Option<&str> {
    let mut first = true;
    for a in p.cmd.iter().skip(1) {
        if a.starts_with('-') {
            continue;
        }
        if first && (a == "run" || a == "x") {
            first = false;
            continue;
        }
        return Some(a.as_str());
    }
    None
}

fn kind_by_base(base: &str) -> Option<TargetKind> {
    match base {
        "claude" => Some(TargetKind::ClaudeCode),
        "grok" => Some(TargetKind::Grok),
        b if b == "codex" || b.starts_with("codex-") => Some(TargetKind::CodexCli),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agents::tests::{p, with_exe};

    #[test]
    fn classify_cases() {
        use TargetKind::*;
        let cases: Vec<(ProcInfo, Option<TargetKind>)> = vec![
            (
                p(10, 1, "claude", &["claude", "--continue"]),
                Some(ClaudeCode),
            ),
            (
                with_exe(
                    p(11, 1, "2.1.3", &["claude"]),
                    "/Users/u/.local/share/claude/versions/2.1.3",
                ),
                Some(ClaudeCode),
            ),
            (
                p(
                    12,
                    1,
                    "node",
                    &["node", "/usr/local/bin/claude", "-p", "hi"],
                ),
                Some(ClaudeCode),
            ),
            (
                p(
                    13,
                    1,
                    "node",
                    &[
                        "node",
                        "--no-warnings",
                        "/opt/lib/node_modules/@anthropic-ai/claude-code/cli.js",
                    ],
                ),
                Some(ClaudeCode),
            ),
            (
                with_exe(
                    p(
                        14,
                        1,
                        "Claude",
                        &["/Applications/Claude.app/Contents/MacOS/Claude"],
                    ),
                    "/Applications/Claude.app/Contents/MacOS/Claude",
                ),
                Some(ClaudeDesktop),
            ),
            (
                with_exe(
                    p(15, 1, "claude.exe", &[]),
                    r"C:\Users\u\AppData\Local\AnthropicClaude\app-0.9.0\claude.exe",
                ),
                Some(ClaudeDesktop),
            ),
            (
                with_exe(
                    p(16, 1, "claude.exe", &[]),
                    r"C:\Users\u\.local\bin\claude.exe",
                ),
                Some(ClaudeCode),
            ),
            (p(17, 1, "codex", &["codex", "resume"]), Some(CodexCli)),
            (
                p(
                    18,
                    1,
                    "node",
                    &["node", "/usr/lib/node_modules/@openai/codex/bin/codex.js"],
                ),
                Some(CodexCli),
            ),
            (
                with_exe(p(19, 1, "codex.exe", &[]), r"C:\tools\codex.exe"),
                Some(CodexCli),
            ),
            (
                with_exe(
                    p(20, 1, "Codex", &[]),
                    "/Applications/Codex.app/Contents/MacOS/Codex",
                ),
                Some(CodexDesktop),
            ),
            (p(21, 1, "grok", &["grok"]), Some(Grok)),
            (
                p(22, 1, "node", &["node", "/usr/local/bin/grok"]),
                Some(Grok),
            ),
            (
                with_exe(p(23, 1, "grok.exe", &[]), r"C:\bin\grok.exe"),
                Some(Grok),
            ),
            (
                p(24, 1, "bun", &["bun", "run", "/x/grok-cli/dist/index.js"]),
                Some(Grok),
            ),
            // Не агенты: слово только в аргументах.
            (p(30, 1, "echo", &["echo", "claude"]), None),
            (p(31, 1, "grep", &["grep", "codex"]), None),
            (
                p(32, 1, "zsh", &["/bin/zsh", "-c", "claude --resume x"]),
                None,
            ),
            (p(33, 1, "vim", &["vim", "claude.md"]), None),
            (
                p(
                    34,
                    1,
                    "tmux",
                    &["tmux", "new-session", "-d", "'claude' '--continue'"],
                ),
                None,
            ),
            (
                p(35, 1, "node", &["node", "/srv/app/server.js", "claude"]),
                None,
            ),
            (
                p(
                    36,
                    1,
                    "Code Helper",
                    &["/Applications/Visual Studio Code.app/x"],
                ),
                None,
            ),
            (p(37, 1, "node", &["node"]), None),
        ];
        for (pi, want) in cases {
            assert_eq!(classify(&pi), want, "{pi:?}");
        }
    }
}
