//! Часы монитора: монотонное время для отсчётов и «настенное» для журнала.

#[cfg(any(test, feature = "test-util"))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use chrono::{DateTime, SecondsFormat, Utc};

pub trait Clock: Send + Sync {
    /// Монотонное время в миллисекундах.
    fn now_ms(&self) -> u64;
    fn wall(&self) -> DateTime<Utc>;

    /// Настенное время строкой RFC 3339 (как в журнале и `lastChangeAt`).
    fn stamp(&self) -> String {
        self.wall().to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

pub struct SystemClock {
    start: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        SystemClock {
            start: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
    fn wall(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Ручные часы для тестов.
#[cfg(any(test, feature = "test-util"))]
pub struct FakeClock {
    ms: AtomicU64,
    base: DateTime<Utc>,
}

#[cfg(any(test, feature = "test-util"))]
impl Default for FakeClock {
    fn default() -> Self {
        FakeClock {
            ms: AtomicU64::new(0),
            base: DateTime::from_timestamp(1_800_000_000, 0).expect("valid ts"),
        }
    }
}

#[cfg(any(test, feature = "test-util"))]
impl FakeClock {
    pub fn advance_ms(&self, ms: u64) {
        self.ms.fetch_add(ms, Ordering::SeqCst);
    }
}

#[cfg(any(test, feature = "test-util"))]
impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.ms.load(Ordering::SeqCst)
    }
    fn wall(&self) -> DateTime<Utc> {
        self.base + chrono::Duration::milliseconds(self.now_ms() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_clock_moves_both_times() {
        let c = FakeClock::default();
        assert_eq!(c.now_ms(), 0);
        let before = c.stamp();
        c.advance_ms(1500);
        assert_eq!(c.now_ms(), 1500);
        assert_ne!(c.stamp(), before);
        assert!(c.stamp().ends_with('Z'));
    }
}
