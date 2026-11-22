//! Деревья потомков: агенты включённых видов + все их потомки.

use std::collections::{HashMap, HashSet, VecDeque};

use super::classify::classify;
use crate::models::{AgentProc, ProcInfo, SelfCtx, TargetKind, Targets};

/// Индекс снимка: процесс по pid и дети по pid родителя (по возрастанию).
struct ProcTree<'a> {
    by_pid: HashMap<u32, &'a ProcInfo>,
    children: HashMap<u32, Vec<u32>>,
}

impl<'a> ProcTree<'a> {
    fn new(procs: &'a [ProcInfo]) -> Self {
        let by_pid = procs.iter().map(|p| (p.pid, p)).collect();
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        for p in procs {
            if let Some(pp) = p.ppid {
                if pp != p.pid {
                    children.entry(pp).or_default().push(p.pid);
                }
            }
        }
        for list in children.values_mut() {
            list.sort_unstable();
        }
        ProcTree { by_pid, children }
    }

    fn parent(&self, pid: u32) -> Option<u32> {
        self.by_pid.get(&pid).and_then(|p| p.ppid)
    }

    /// pid и все его потомки.
    fn subtree(&self, pid: u32) -> HashSet<u32> {
        let mut out = HashSet::new();
        let mut stack = vec![pid];
        while let Some(pid) = stack.pop() {
            if out.insert(pid) {
                if let Some(ch) = self.children.get(&pid) {
                    stack.extend(ch);
                }
            }
        }
        out
    }

    /// Есть ли среди предков pid из `set` (циклы не зацикливают).
    fn has_ancestor_in(&self, pid: u32, set: &HashMap<u32, TargetKind>) -> bool {
        let mut seen = HashSet::from([pid]);
        let mut cur = self.parent(pid);
        while let Some(pp) = cur {
            if !seen.insert(pp) {
                break;
            }
            if set.contains_key(&pp) {
                return true;
            }
            cur = self.parent(pp);
        }
        false
    }
}

/// Собрать цели: агенты включённых видов + все их потомки (одного с нами
/// пользователя). Себя, своих потомков и чужих не трогаем. Порядок — родители
/// раньше детей; для закрытия разворачивать.
pub fn collect(procs: &[ProcInfo], targets: &Targets, me: &SelfCtx) -> Vec<AgentProc> {
    let tree = ProcTree::new(procs);
    let excluded = tree.subtree(me.pid);

    let same_user = |p: &ProcInfo| match (&me.uid, &p.uid) {
        (Some(a), Some(b)) => a == b,
        (Some(_), None) => false,
        (None, _) => true,
    };
    let allowed = |p: &ProcInfo| {
        p.pid > 1 && p.pid <= i32::MAX as u32 && !excluded.contains(&p.pid) && same_user(p)
    };

    let roots: HashMap<u32, TargetKind> = procs
        .iter()
        .filter(|p| allowed(p))
        .filter_map(|p| classify(p).map(|k| (p.pid, k)))
        .filter(|(_, k)| k.enabled_in(targets))
        .collect();

    // Корень, у которого предок тоже корень, войдёт в дерево предка.
    let mut top: Vec<(u32, TargetKind)> = roots
        .iter()
        .filter(|(pid, _)| !tree.has_ancestor_in(**pid, &roots))
        .map(|(p, k)| (*p, *k))
        .collect();
    top.sort_unstable();

    let mut out = Vec::new();
    let mut taken: HashSet<u32> = HashSet::new();
    for (root, kind) in top {
        // Обход в ширину: родители раньше детей.
        let mut queue = VecDeque::from([(root, 0u32)]);
        while let Some((pid, depth)) = queue.pop_front() {
            let Some(p) = tree.by_pid.get(&pid) else {
                continue;
            };
            if !allowed(p) || !taken.insert(pid) {
                continue;
            }
            out.push(AgentProc {
                target: kind,
                pid,
                name: p.name.clone(),
                start_time: p.start_time,
                root,
                depth,
            });
            if let Some(ch) = tree.children.get(&pid) {
                queue.extend(ch.iter().map(|c| (*c, depth + 1)));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agents::summary::summary;
    use crate::services::agents::tests::{me, p, sample};

    #[test]
    fn collect_trees_and_exclusions() {
        let all = collect(&sample(), &Targets::default(), &me());
        let pids: Vec<u32> = all.iter().map(|a| a.pid).collect();
        for want in [200, 201, 202, 203, 300, 400, 800, 801] {
            assert!(pids.contains(&want), "нет {want}: {pids:?}");
        }
        for bad in [1, 100, 500, 600, 700, 701] {
            assert!(!pids.contains(&bad), "лишний {bad}");
        }
        // 801 — потомок Desktop, идёт в его дереве.
        let a801 = all.iter().find(|a| a.pid == 801).unwrap();
        assert_eq!(a801.target, TargetKind::ClaudeDesktop);
        assert_eq!(a801.root, 800);
        // Родители раньше детей.
        let pos = |pid| pids.iter().position(|p| *p == pid).unwrap();
        assert!(pos(200) < pos(202) && pos(202) < pos(203));
        assert_eq!(all.iter().find(|a| a.pid == 203).unwrap().depth, 2);
        assert_eq!(
            summary(&all),
            "claude ×1 сессия, Claude Desktop ×1 сессия, codex ×1 сессия, grok ×1 сессия \
             (+4 вспомогательных процесса)"
        );
    }

    #[test]
    fn collect_respects_disabled_targets() {
        let t = Targets {
            claude_desktop: false,
            codex_cli: false,
            grok: false,
            ..Targets::default()
        };
        let all = collect(&sample(), &t, &me());
        let roots: Vec<(u32, TargetKind)> = all
            .iter()
            .filter(|a| a.is_root())
            .map(|a| (a.pid, a.target))
            .collect();
        // Desktop выключен — Claude Code внутри него становится своим корнем.
        assert_eq!(
            roots,
            vec![(200, TargetKind::ClaudeCode), (801, TargetKind::ClaudeCode)]
        );
        assert!(!all.iter().any(|a| a.pid == 800 || a.pid == 300));
        assert!(collect(
            &sample(),
            &Targets {
                claude_code: false,
                claude_desktop: false,
                codex_cli: false,
                codex_desktop: false,
                grok: false
            },
            &me()
        )
        .is_empty());
    }

    #[test]
    fn collect_handles_cycles_and_bad_pids() {
        let mut v = vec![p(10, 11, "claude", &["claude"]), p(11, 10, "zsh", &["zsh"])];
        v.push(p(u32::MAX, 1, "claude", &["claude"]));
        let all = collect(&v, &Targets::default(), &me());
        let pids: Vec<u32> = all.iter().map(|a| a.pid).collect();
        assert!(pids.contains(&10));
        assert!(!pids.contains(&u32::MAX));
    }
}
