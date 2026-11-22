//! Тексты по списку целей: «claude ×15 сессий (+93 вспомогательных процесса)».

use crate::models::{AgentProc, TargetKind};

/// Русское склонение по числу: `plural(n, "сессия", "сессии", "сессий")`.
pub fn plural<'a>(n: usize, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let n100 = n % 100;
    let n10 = n % 10;
    if (11..=14).contains(&n100) {
        many
    } else if n10 == 1 {
        one
    } else if (2..=4).contains(&n10) {
        few
    } else {
        many
    }
}

/// «5 вспомогательных процессов».
pub fn helpers_text(n: usize) -> String {
    format!(
        "{n} {}",
        plural(
            n,
            "вспомогательный процесс",
            "вспомогательных процесса",
            "вспомогательных процессов"
        )
    )
}

/// «15 сессий» (подпись трея).
pub fn sessions_text(n: usize) -> String {
    format!("{n} {}", plural(n, "сессия", "сессии", "сессий"))
}

/// «5 процессов».
pub fn procs_text(n: usize) -> String {
    format!("{n} {}", plural(n, "процесс", "процесса", "процессов"))
}

/// По сессиям (корням) с числом вспомогательных процессов:
/// «claude ×15 сессий, codex ×1 сессия (+93 вспомогательных процесса)».
/// Корней нет — «2 вспомогательных процесса».
pub fn summary(agents: &[AgentProc]) -> String {
    let mut counts: Vec<(TargetKind, usize)> = Vec::new();
    for a in agents.iter().filter(|a| a.is_root()) {
        match counts.iter_mut().find(|(k, _)| *k == a.target) {
            Some((_, n)) => *n += 1,
            None => counts.push((a.target, 1)),
        }
    }
    counts.sort();
    let helpers = agents.iter().filter(|a| !a.is_root()).count();
    if counts.is_empty() {
        return if helpers == 0 {
            "нет агентов".into()
        } else {
            helpers_text(helpers)
        };
    }
    let mut text = counts
        .iter()
        .map(|(k, n)| {
            format!(
                "{} ×{n} {}",
                k.short(),
                plural(*n, "сессия", "сессии", "сессий")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    if helpers > 0 {
        text.push_str(&format!(" (+{})", helpers_text(helpers)));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plurals() {
        let s = |n| plural(n, "сессия", "сессии", "сессий");
        assert_eq!(
            [1, 2, 4, 5, 11, 12, 14, 15, 21, 22, 25, 101, 111].map(s),
            [
                "сессия",
                "сессии",
                "сессии",
                "сессий",
                "сессий",
                "сессий",
                "сессий",
                "сессий",
                "сессия",
                "сессии",
                "сессий",
                "сессия",
                "сессий"
            ]
        );
        assert_eq!(helpers_text(1), "1 вспомогательный процесс");
        assert_eq!(helpers_text(93), "93 вспомогательных процесса");
        assert_eq!(helpers_text(5), "5 вспомогательных процессов");
        assert_eq!(procs_text(21), "21 процесс");
        assert_eq!(sessions_text(15), "15 сессий");
        assert_eq!(sessions_text(22), "22 сессии");
        assert_eq!(summary(&[]), "нет агентов");
    }

    #[test]
    fn helpers_only() {
        let a = AgentProc {
            target: TargetKind::Grok,
            pid: 5,
            name: "bash".into(),
            start_time: 0,
            root: 4,
            depth: 1,
        };
        assert_eq!(summary(&[a.clone(), a]), "2 вспомогательных процесса");
    }
}
