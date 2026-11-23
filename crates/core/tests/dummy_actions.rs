//! Действия на НАСТОЯЩИХ сигналах — только над собственными дочерними
//! процессами-пустышками (ссылка на `sleep` под именем агента во временной папке),
//! и только по pid, который тест сам запустил. Снимок процессов для монитора
//! фильтруется до этих pid, так что живые сессии пользователя не видны вообще.
#![cfg(unix)]

use std::path::Path;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::{Duration, Instant};

use vpn_closer_core::models::{
    Action, ActionReport, CheckResult, Config, ProcInfo, SelfCtx, Status, WatchItem,
};
use vpn_closer_core::testing::{
    apply_with, CheckSource, FakeClock, FrozenFile, Me, Monitor, MonitorParts, OsProcessControl,
    ProcSource, ProcessControl, SysProcSource, SystemActor,
};

const ME: u32 = 1;

fn act(action: Action, pids: &[u32], dry_run: bool) -> ActionReport {
    apply_with(&OsProcessControl, action, pids, dry_run, ME)
}

fn is_alive(pid: u32) -> bool {
    OsProcessControl.is_alive(pid)
}

/// Пустышка: sleep, запущенный по ссылке с именем агента. Drop добивает и забирает её.
struct Dummy {
    child: Option<Child>,
    pid: u32,
}

impl Dummy {
    fn spawn(dir: &Path, name: &str) -> Dummy {
        let exe = dir.join(name);
        if !exe.exists() {
            // Символическая ссылка, а не копия: копию системного бинарника
            // macOS убивает при запуске (подпись).
            std::os::unix::fs::symlink("/bin/sleep", &exe).expect("symlink sleep");
        }
        let child = Command::new(&exe).arg("120").spawn().expect("spawn dummy");
        let pid = child.id();
        Dummy {
            child: Some(child),
            pid,
        }
    }

    /// waitpid с флагами, ждём до 3 с изменения статуса.
    fn wait_status(&self, flags: libc::c_int) -> Option<libc::c_int> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let mut status: libc::c_int = 0;
            // SAFETY: ждём только свой дочерний процесс.
            let rc = unsafe {
                libc::waitpid(self.pid as libc::pid_t, &mut status, flags | libc::WNOHANG)
            };
            if rc == self.pid as libc::pid_t {
                return Some(status);
            }
            if rc < 0 || Instant::now() > deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn expect_stopped(&self) {
        let st = self.wait_status(libc::WUNTRACED).expect("ждали остановку");
        assert!(libc::WIFSTOPPED(st), "status {st:#x}");
    }

    fn expect_continued(&self) {
        let st = self
            .wait_status(libc::WCONTINUED)
            .expect("ждали продолжение");
        assert!(libc::WIFCONTINUED(st), "status {st:#x}");
    }

    fn expect_signaled(&mut self, sig: libc::c_int) {
        let st = self.wait_status(0).expect("ждали завершение");
        assert!(libc::WIFSIGNALED(st), "status {st:#x}");
        assert_eq!(libc::WTERMSIG(st), sig);
        self.child = None; // уже забран
    }

    fn not_changed(&self) -> bool {
        let mut status: libc::c_int = 0;
        // SAFETY: только свой дочерний процесс.
        let rc = unsafe {
            libc::waitpid(
                self.pid as libc::pid_t,
                &mut status,
                libc::WUNTRACED | libc::WCONTINUED | libc::WNOHANG,
            )
        };
        rc == 0
    }
}

impl Drop for Dummy {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

#[test]
fn freeze_resume_terminate_own_dummy() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = Dummy::spawn(dir.path(), "claude");
    assert!(is_alive(d.pid));

    let rep = act(Action::Freeze, &[d.pid], false);
    assert_eq!(rep.ok, vec![d.pid], "{:?}", rep.failed);
    d.expect_stopped();

    let rep = act(Action::Resume, &[d.pid], false);
    assert_eq!(rep.ok, vec![d.pid]);
    d.expect_continued();

    // Заморозим снова: terminate обязан сначала продолжить процесс.
    act(Action::Freeze, &[d.pid], false);
    d.expect_stopped();
    let rep = act(Action::Terminate, &[d.pid], false);
    assert_eq!(rep.ok, vec![d.pid]);
    d.expect_signaled(libc::SIGTERM);
    assert!(!is_alive(d.pid));
}

#[test]
fn kill_frozen_dummy() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = Dummy::spawn(dir.path(), "codex");
    act(Action::Freeze, &[d.pid], false);
    d.expect_stopped();
    let rep = act(Action::Kill, &[d.pid], false);
    assert_eq!(rep.ok, vec![d.pid]);
    d.expect_signaled(libc::SIGKILL);
}

#[test]
fn dry_run_does_not_touch_dummy() {
    let dir = tempfile::tempdir().unwrap();
    let d = Dummy::spawn(dir.path(), "claude");
    for a in [Action::Freeze, Action::Terminate, Action::Kill] {
        let rep = act(a, &[d.pid], true);
        assert_eq!(rep.ok, vec![d.pid]);
    }
    std::thread::sleep(Duration::from_millis(100));
    assert!(d.not_changed());
    assert!(is_alive(d.pid));
}

// ---------------------------------------------------------------- monitor e2e

/// Настоящий снимок процессов, но только наши пустышки.
struct OnlyPids(SysProcSource, Vec<u32>);
impl ProcSource for OnlyPids {
    fn snapshot(&mut self) -> Vec<ProcInfo> {
        self.0
            .snapshot()
            .into_iter()
            .filter(|p| self.1.contains(&p.pid))
            .collect()
    }
}

struct VpnDown;
impl CheckSource for VpnDown {
    fn check(&mut self, cfg: &Config, _: &[ProcInfo]) -> Vec<CheckResult> {
        cfg.watch
            .iter()
            .map(|w| CheckResult::new(w.clone(), Some(false), "тест: VPN нет"))
            .collect()
    }
}

fn monitor_for(dir: &Path, pids: Vec<u32>, cfg: Config, clock: Arc<FakeClock>) -> Monitor {
    Monitor::new(
        cfg,
        MonitorParts {
            checks: Box::new(VpnDown),
            procs: Box::new(OnlyPids(SysProcSource::new(), pids)),
            actor: Box::new(SystemActor::new(Arc::new(OsProcessControl), ME)),
            clock,
            frozen: Box::new(FrozenFile::new(dir.join("frozen.json"))),
            // Мы — родитель пустышек; «себя» указываем несуществующим pid, иначе
            // пустышки как наши потомки были бы исключены.
            me: Me::Fixed(SelfCtx { pid: 0, uid: None }),
        },
    )
}

fn cfg(dry_run: bool) -> Config {
    Config {
        watch: vec![WatchItem::Route],
        monitoring: true,
        dry_run,
        confirm_failures: 1,
        close_after_sec: 1,
        kill_grace_sec: 1,
        ..Config::default()
    }
}

#[test]
fn monitor_freezes_then_closes_dummies() {
    let dir = tempfile::tempdir().unwrap();
    let mut a = Dummy::spawn(dir.path(), "claude");
    let mut b = Dummy::spawn(dir.path(), "codex");
    let clock = Arc::new(FakeClock::default());
    let mut m = monitor_for(dir.path(), vec![a.pid, b.pid], cfg(false), clock.clone());

    let out = m.tick();
    assert_eq!(out.state.status, Status::Countdown, "{:?}", out.events);
    assert_eq!(out.state.agents.len(), 2, "{:?}", out.state.agents);
    assert!(out.state.agents.iter().all(|x| x.frozen));
    a.expect_stopped();
    b.expect_stopped();

    clock.advance_ms(1000);
    let out = m.tick();
    assert_eq!(out.state.status, Status::Closed);
    a.expect_signaled(libc::SIGTERM);
    b.expect_signaled(libc::SIGTERM);
}

#[test]
fn monitor_dry_run_leaves_dummies_alone() {
    let dir = tempfile::tempdir().unwrap();
    let a = Dummy::spawn(dir.path(), "claude");
    let clock = Arc::new(FakeClock::default());
    let mut m = monitor_for(dir.path(), vec![a.pid], cfg(true), clock.clone());
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(m.tick().events);
        clock.advance_ms(1000);
    }
    assert_eq!(m.state().status, Status::Closed);
    std::thread::sleep(Duration::from_millis(100));
    assert!(a.not_changed());
    assert!(is_alive(a.pid));
    assert!(events.iter().any(|e| e.message.starts_with("Закрыл бы")));
}

#[test]
fn shutdown_resumes_frozen_dummy() {
    let dir = tempfile::tempdir().unwrap();
    let a = Dummy::spawn(dir.path(), "grok");
    let clock = Arc::new(FakeClock::default());
    let mut m = monitor_for(
        dir.path(),
        vec![a.pid],
        Config {
            close_after_sec: 60,
            ..cfg(false)
        },
        clock,
    );
    m.tick();
    a.expect_stopped();
    m.shutdown();
    a.expect_continued();
}

#[test]
fn killed_program_leftover_resumed_on_next_start() {
    let dir = tempfile::tempdir().unwrap();
    let a = Dummy::spawn(dir.path(), "grok");
    let long = Config {
        close_after_sec: 60,
        ..cfg(false)
    };
    let clock = Arc::new(FakeClock::default());
    let mut m = monitor_for(dir.path(), vec![a.pid], long.clone(), clock.clone());
    m.tick();
    a.expect_stopped();
    // Падение: ни shutdown, ни разморозки.
    drop(m);
    let off = Config {
        monitoring: false,
        ..long
    };
    let mut m = monitor_for(dir.path(), vec![a.pid], off, clock);
    m.tick();
    a.expect_continued();
    assert!(!dir.path().join("frozen.json").exists());
}
