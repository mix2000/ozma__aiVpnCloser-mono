//! Учёт замороженных на случай падения: что записано в хранилище и кто остался
//! замороженным с прошлого запуска.

use crate::models::{AgentProc, FrozenProc};
use crate::storage::FrozenStore;

pub struct FrozenRecord {
    store: Box<dyn FrozenStore>,
    saved: Vec<FrozenProc>,
    /// Заморожены прошлым запуском и ещё не отпущены.
    leftover: Vec<FrozenProc>,
    /// Об ошибке записи уже сообщили — повторно, пока не получится, не пишем.
    failing: bool,
}

impl FrozenRecord {
    pub fn open(store: Box<dyn FrozenStore>) -> FrozenRecord {
        let leftover = store.load();
        FrozenRecord {
            store,
            saved: leftover.clone(),
            leftover,
            failing: false,
        }
    }

    pub fn has_leftover(&self) -> bool {
        !self.leftover.is_empty()
    }

    pub fn is_leftover(&self, a: &AgentProc) -> bool {
        self.leftover
            .iter()
            .any(|f| f.pid == a.pid && f.start_time == a.start_time)
    }

    pub fn take_leftover(&mut self) -> Vec<FrozenProc> {
        std::mem::take(&mut self.leftover)
    }

    /// Записать `frozen` и оставшихся, если список изменился. Текст ошибки —
    /// только о первой неудаче подряд.
    pub fn sync(&mut self, frozen: impl Iterator<Item = FrozenProc>) -> Result<(), String> {
        let mut want = self.leftover.clone();
        for f in frozen {
            if !want.contains(&f) {
                want.push(f);
            }
        }
        if want == self.saved {
            return Ok(());
        }
        match self.store.save(&want) {
            Ok(()) => {
                self.saved = want;
                self.failing = false;
                Ok(())
            }
            Err(_) if self.failing => Ok(()),
            Err(e) => {
                self.failing = true;
                Err(e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct Broken;
    impl FrozenStore for Broken {
        fn load(&self) -> Vec<FrozenProc> {
            Vec::new()
        }
        fn save(&mut self, _: &[FrozenProc]) -> io::Result<()> {
            Err(io::Error::other("диск полон"))
        }
    }

    #[test]
    fn write_error_reported_once() {
        let mut r = FrozenRecord::open(Box::new(Broken));
        let one = || {
            std::iter::once(FrozenProc {
                pid: 7,
                start_time: 1,
            })
        };
        assert_eq!(r.sync(one()), Err("диск полон".to_string()));
        assert_eq!(r.sync(one()), Ok(()));
    }
}
