//! Действия над процессами — только по явному списку pid.
//!
//! Собственный процесс, pid 0/1 и pid вне диапазона `pid_t` отвергаются (на
//! Unix `kill(-N)` бил бы по группе процессов). Сами сигналы —
//! `platform::ProcessControl`.

use std::sync::Arc;

use crate::models::{Action, ActionReport};
use crate::platform::ProcessControl;

/// Исполнитель действий для монитора (в тестах — записывающий подставной).
pub trait Actor: Send {
    fn apply(&mut self, action: Action, pids: &[u32]) -> ActionReport;
}

/// Настоящие действия через [`ProcessControl`]; себя (`self_pid`) не трогает.
pub struct SystemActor {
    control: Arc<dyn ProcessControl>,
    self_pid: u32,
}

impl SystemActor {
    pub fn new(control: Arc<dyn ProcessControl>, self_pid: u32) -> Self {
        SystemActor { control, self_pid }
    }
}

impl Actor for SystemActor {
    fn apply(&mut self, action: Action, pids: &[u32]) -> ActionReport {
        apply_with(self.control.as_ref(), action, pids, false, self.self_pid)
    }
}

/// Можно ли трогать pid.
pub fn guard(pid: u32, self_pid: u32) -> Result<(), String> {
    if pid <= 1 || pid > i32::MAX as u32 {
        return Err(format!("pid {pid} не трогаем"));
    }
    if pid == self_pid {
        return Err("себя не трогаем".into());
    }
    Ok(())
}

/// Выполнить действие над каждым pid по порядку через `control`.
/// `dry_run` — ничего не слать.
pub fn apply_with(
    control: &dyn ProcessControl,
    action: Action,
    pids: &[u32],
    dry_run: bool,
    self_pid: u32,
) -> ActionReport {
    let mut rep = ActionReport {
        dry_run,
        ..Default::default()
    };
    for &pid in pids {
        let res = guard(pid, self_pid).and_then(|()| {
            if dry_run {
                Ok(())
            } else {
                control.apply(action, pid).map_err(|e| e.to_string())
            }
        });
        match res {
            Ok(()) => rep.ok.push(pid),
            Err(e) => rep.failed.push((pid, e)),
        }
    }
    rep
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Mutex;

    /// Записывает вызовы, pid 13 «падает».
    #[derive(Default)]
    struct Rec(Mutex<Vec<(Action, u32)>>);
    impl ProcessControl for Rec {
        fn apply(&self, action: Action, pid: u32) -> io::Result<()> {
            self.0.lock().unwrap().push((action, pid));
            if pid == 13 {
                Err(io::Error::other("нет такого процесса"))
            } else {
                Ok(())
            }
        }
        fn is_alive(&self, _: u32) -> bool {
            true
        }
    }

    #[test]
    fn guard_rejects_dangerous_pids() {
        let me = 4000;
        let rec = Rec::default();
        let rep = apply_with(
            &rec,
            Action::Kill,
            &[0, 1, me, u32::MAX, i32::MAX as u32 + 1],
            false,
            me,
        );
        assert!(rep.ok.is_empty());
        assert_eq!(rep.failed.len(), 5);
        assert!(rec.0.lock().unwrap().is_empty(), "до ОС не дошло");
        assert_eq!(rep.failed[2].1, "себя не трогаем");
    }

    #[test]
    fn order_and_errors_reported() {
        let rec = Rec::default();
        let rep = apply_with(&rec, Action::Freeze, &[10, 13, 11], false, 1);
        assert_eq!(rep.ok, vec![10, 11]);
        assert_eq!(rep.failed, vec![(13, "нет такого процесса".to_string())]);
        let calls: Vec<u32> = rec.0.lock().unwrap().iter().map(|c| c.1).collect();
        assert_eq!(calls, [10, 13, 11]);
    }

    #[test]
    fn dry_run_sends_nothing() {
        let rec = Rec::default();
        let rep = apply_with(&rec, Action::Terminate, &[i32::MAX as u32 - 7], true, 1);
        assert!(rep.dry_run);
        assert_eq!(rep.ok.len(), 1);
        assert!(rec.0.lock().unwrap().is_empty());
        // SystemActor — всегда по-настоящему, но через свой ProcessControl.
        let rec = Arc::new(Rec::default());
        let mut actor = SystemActor::new(rec.clone(), 1);
        assert_eq!(actor.apply(Action::Resume, &[10]).ok, vec![10]);
        assert_eq!(rec.0.lock().unwrap().len(), 1);
    }
}
