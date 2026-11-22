//! Конечный автомат наблюдения: ok → down (дребезг) → countdown → closed → ok.
//!
//! Чистый: на входе итог проверок и время, на выходе — переход. Что делать с
//! процессами при переходе, решает `executor`.

use crate::models::{Config, Status, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Ok,
    Countdown { deadline_ms: u64 },
    Closed,
}

/// Что случилось за шаг.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    None,
    /// Подтверждено: VPN пропал. `close_now` — «закрыть через 0 с».
    Down {
        close_now: bool,
    },
    /// VPN вернулся во время отсчёта.
    BackInCountdown,
    /// Отсчёт кончился — закрыть.
    Expired,
    /// Идёт отсчёт: заморозить только что запущенных.
    FreezeNew,
    /// VPN вернулся после закрытия.
    BackAfterClose,
    /// Агенты закрыты, VPN нет: закрыть только что запущенных.
    CloseNew,
}

#[derive(Debug, Clone)]
pub struct Machine {
    phase: Phase,
    fail_streak: u32,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            phase: Phase::Ok,
            fail_streak: 0,
        }
    }
}

impl Machine {
    #[cfg(test)]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Наблюдение выключено или не за чем следить: всё с начала.
    pub fn reset(&mut self) {
        *self = Machine::default();
    }

    /// Один шаг по итогу проверок.
    pub fn step(&mut self, now_ms: u64, verdict: Verdict, cfg: &Config) -> Transition {
        match verdict {
            Verdict::Down => self.fail_streak = self.fail_streak.saturating_add(1),
            Verdict::Up => self.fail_streak = 0,
            Verdict::Unknown => {}
        }
        match self.phase {
            Phase::Ok if self.fail_streak >= cfg.confirm_failures => {
                let n = cfg.close_after_sec;
                if n == 0 {
                    self.phase = Phase::Closed;
                } else {
                    self.phase = Phase::Countdown {
                        deadline_ms: now_ms + u64::from(n) * 1000,
                    };
                }
                Transition::Down { close_now: n == 0 }
            }
            Phase::Ok => Transition::None,
            Phase::Countdown { deadline_ms } => {
                if verdict == Verdict::Up {
                    self.reset();
                    Transition::BackInCountdown
                } else if now_ms >= deadline_ms {
                    self.phase = Phase::Closed;
                    Transition::Expired
                } else if cfg.block_while_down && cfg.freeze_immediately {
                    Transition::FreezeNew
                } else {
                    Transition::None
                }
            }
            Phase::Closed => {
                if verdict == Verdict::Up {
                    self.reset();
                    Transition::BackAfterClose
                } else if cfg.block_while_down {
                    Transition::CloseNew
                } else {
                    Transition::None
                }
            }
        }
    }

    /// Статус для UI после шага.
    pub fn status(&self, verdict: Verdict) -> Status {
        match self.phase {
            Phase::Ok if self.fail_streak > 0 => Status::Down,
            Phase::Ok if verdict == Verdict::Unknown => Status::Unknown,
            Phase::Ok => Status::Ok,
            Phase::Countdown { .. } => Status::Countdown,
            Phase::Closed => Status::Closed,
        }
    }

    /// Сколько секунд до закрытия (вверх), если идёт отсчёт.
    pub fn countdown_left_sec(&self, now_ms: u64) -> Option<u32> {
        match self.phase {
            Phase::Countdown { deadline_ms } => {
                Some(deadline_ms.saturating_sub(now_ms).div_ceil(1000) as u32)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Transition as T;
    use Verdict::{Down, Unknown, Up};

    fn cfg(confirm: u32, close_after: u32) -> Config {
        Config {
            confirm_failures: confirm,
            close_after_sec: close_after,
            ..Config::default()
        }
    }

    #[test]
    fn debounce_then_countdown_then_back() {
        let c = cfg(2, 3);
        let mut m = Machine::default();
        assert_eq!(m.step(0, Up, &c), T::None);
        assert_eq!(m.status(Up), Status::Ok);
        assert_eq!(m.step(1000, Down, &c), T::None);
        assert_eq!(m.status(Down), Status::Down);
        // Неизвестно не сбрасывает и не увеличивает счётчик.
        assert_eq!(m.step(2000, Unknown, &c), T::None);
        assert_eq!(m.status(Unknown), Status::Down);
        assert_eq!(m.step(3000, Down, &c), T::Down { close_now: false });
        assert_eq!(m.phase(), Phase::Countdown { deadline_ms: 6000 });
        assert_eq!(m.countdown_left_sec(3000), Some(3));
        assert_eq!(m.countdown_left_sec(3001), Some(3));
        assert_eq!(m.countdown_left_sec(5000), Some(1));
        assert_eq!(m.step(4000, Down, &c), T::FreezeNew);
        assert_eq!(m.step(5000, Up, &c), T::BackInCountdown);
        assert_eq!(m.status(Up), Status::Ok);
        assert_eq!(m.countdown_left_sec(5000), None);
    }

    #[test]
    fn expiry_and_back_after_close() {
        let c = cfg(1, 2);
        let mut m = Machine::default();
        assert_eq!(m.step(0, Down, &c), T::Down { close_now: false });
        assert_eq!(m.step(1000, Unknown, &c), T::FreezeNew);
        assert_eq!(m.step(2000, Unknown, &c), T::Expired);
        assert_eq!(m.status(Unknown), Status::Closed);
        assert_eq!(m.step(3000, Down, &c), T::CloseNew);
        assert_eq!(m.step(4000, Up, &c), T::BackAfterClose);
        assert_eq!(m.status(Up), Status::Ok);
    }

    #[test]
    fn zero_delay_and_flags() {
        let c = Config {
            block_while_down: false,
            ..cfg(1, 0)
        };
        let mut m = Machine::default();
        assert_eq!(m.step(0, Down, &c), T::Down { close_now: true });
        assert_eq!(m.phase(), Phase::Closed);
        assert_eq!(m.step(1000, Down, &c), T::None, "без блокировки новых");
        let c = Config {
            freeze_immediately: false,
            ..cfg(1, 5)
        };
        let mut m = Machine::default();
        m.step(0, Down, &c);
        assert_eq!(m.step(1000, Down, &c), T::None, "без заморозки новых");
        m.reset();
        assert_eq!(m.phase(), Phase::Ok);
        assert_eq!(m.status(Up), Status::Ok);
    }
}
