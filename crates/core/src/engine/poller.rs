//! Поток опроса: монитор шагает раз в `pollMs`, принимает новые настройки и
//! команду остановки, итог каждого шага уходит в [`Hub`].

use std::sync::mpsc::{self, RecvTimeoutError, Sender, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use super::hub::Hub;
use crate::models::{AppState, Config};
use crate::services::monitor::{Monitor, MonitorParts};

/// Нижняя граница паузы между шагами (защита от `pollMs` = 0 в тестах).
const MIN_WAIT_MS: u64 = 100;
/// Сколько ждать шага с новыми настройками.
const APPLY_TIMEOUT: Duration = Duration::from_secs(3);

enum Cmd {
    Config(Config, Option<SyncSender<AppState>>),
    Stop,
}

/// Запущенный поток монитора.
pub struct Poller {
    tx: Sender<Cmd>,
    thread: JoinHandle<()>,
}

impl Poller {
    pub fn spawn(hub: Arc<Hub>, cfg: Config, parts: MonitorParts) -> Poller {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let thread = std::thread::Builder::new()
            .name("vpn-closer-monitor".into())
            .spawn(move || {
                let mut monitor = Monitor::new(cfg, parts);
                hub.publish(monitor.tick());
                loop {
                    let wait =
                        Duration::from_millis(u64::from(monitor.config().poll_ms).max(MIN_WAIT_MS));
                    match rx.recv_timeout(wait) {
                        Ok(Cmd::Config(c, reply)) => {
                            monitor.set_config(c);
                            let out = monitor.tick();
                            let st = out.state.clone();
                            hub.publish(out);
                            if let Some(r) = reply {
                                let _ = r.send(st);
                            }
                        }
                        Ok(Cmd::Stop) | Err(RecvTimeoutError::Disconnected) => {
                            hub.publish(monitor.shutdown());
                            break;
                        }
                        Err(RecvTimeoutError::Timeout) => hub.publish(monitor.tick()),
                    }
                }
            })
            .expect("spawn monitor thread");
        Poller { tx, thread }
    }

    /// Ручка для отправки настроек без удержания самого `Poller`.
    pub fn handle(&self) -> PollerHandle {
        PollerHandle {
            tx: self.tx.clone(),
        }
    }

    /// Остановить (монитор разморозит агентов) и дождаться потока.
    pub fn stop(self) {
        let _ = self.tx.send(Cmd::Stop);
        let _ = self.thread.join();
    }
}

/// Отправка настроек в поток монитора.
#[derive(Clone)]
pub struct PollerHandle {
    tx: Sender<Cmd>,
}

impl PollerHandle {
    /// Отправить настройки и дождаться шага с ними (до 3 с).
    pub fn apply(&self, cfg: Config) {
        let (rtx, rrx) = mpsc::sync_channel(1);
        if self.tx.send(Cmd::Config(cfg, Some(rtx))).is_ok() {
            let _ = rrx.recv_timeout(APPLY_TIMEOUT);
        }
    }
}
