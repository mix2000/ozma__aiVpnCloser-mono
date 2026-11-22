//! SIGTERM/SIGINT/SIGHUP (kill, Ctrl+C, выход из сеанса) → обычный выход
//! через `app.exit(0)`, чтобы ядро успело разморозить агентов (`shutdown`).
//! Без этого процесс умер бы сразу, оставив замороженных агентов висеть.

#[cfg(unix)]
mod imp {
    use std::sync::OnceLock;

    static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

    fn signal_set() -> libc::sigset_t {
        // SAFETY: sigset_t — обычная структура; sigemptyset/sigaddset её заполняют.
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            libc::sigaddset(&mut set, libc::SIGTERM);
            libc::sigaddset(&mut set, libc::SIGINT);
            libc::sigaddset(&mut set, libc::SIGHUP);
            set
        }
    }

    /// Вызвать в main до создания любых потоков: маска наследуется ими всеми,
    /// и сигналы получает только наш поток через sigwait.
    pub fn block() {
        let set = signal_set();
        // SAFETY: меняем маску текущего (главного) потока.
        unsafe {
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        }
        std::thread::spawn(move || loop {
            let mut sig: libc::c_int = 0;
            // SAFETY: set заблокирован во всех потоках, sigwait забирает сигнал.
            let rc = unsafe { libc::sigwait(&set, &mut sig) };
            if rc != 0 {
                continue;
            }
            match APP.get() {
                Some(app) => app.exit(0),
                None => std::process::exit(0),
            }
        });
    }

    pub fn attach(app: &tauri::AppHandle) {
        let _ = APP.set(app.clone());
    }
}

#[cfg(not(unix))]
mod imp {
    pub fn block() {}
    pub fn attach(_app: &tauri::AppHandle) {}
}

pub use imp::{attach, block};
