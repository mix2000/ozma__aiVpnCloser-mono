//! Учёт замороженных на случай падения: что записано в хранилище и кого ещё
//! предстоит отпустить — оставшихся с прошлого запуска и тех, кого не удалось
//! разморозить с первого раза.

use crate::models::{AgentProc, FrozenProc};
use crate::storage::FrozenStore;

pub struct FrozenRecord {
    store: Box<dyn FrozenStore>,
    /// Что лежит в хранилище, по возрастанию pid.
    saved: Vec<FrozenProc>,
    /// Что пробовали записать последним (запись могла и не удаться).
    tried: Vec<FrozenProc>,
    /// Заморожены, но не отслеживаются как текущие: ждут разморозки.
    leftover: Vec<FrozenProc>,
    /// О неудачной разморозке этих уже сообщили.
    reported: Vec<FrozenProc>,
    /// Об ошибке записи уже сообщили — повторно, пока не получится, не пишем.
    failing: bool,
    load_error: Option<String>,
    /// Прочитать пока не удалось: повторяем, пока сами не записали поверх.
    unread: bool,
}

fn sorted(mut list: Vec<FrozenProc>) -> Vec<FrozenProc> {
    list.sort_by_key(|f| (f.pid, f.start_time));
    list.dedup();
    list
}

impl FrozenRecord {
    pub fn open(mut store: Box<dyn FrozenStore>) -> FrozenRecord {
        let (list, load_error) = match store.load() {
            Ok(list) => (sorted(list), None),
            Err(e) => (Vec::new(), Some(e.to_string())),
        };
        FrozenRecord {
            store,
            saved: list.clone(),
            tried: list.clone(),
            leftover: list,
            reported: Vec::new(),
            failing: false,
            unread: load_error.is_some(),
            load_error,
        }
    }

    /// Ещё раз прочитать то, что при запуске не прочиталось (файл был занят).
    fn reload(&mut self) {
        if !self.unread {
            return;
        }
        let Ok(list) = self.store.load() else {
            return;
        };
        self.unread = false;
        if list.is_empty() {
            return;
        }
        self.load_error = None;
        self.saved = sorted(list);
        self.leftover.extend(&self.saved);
    }

    /// Ошибка чтения при запуске — один раз.
    pub fn take_load_error(&mut self) -> Option<String> {
        self.load_error.take()
    }

    pub fn has_leftover(&self) -> bool {
        !self.leftover.is_empty()
    }

    pub fn is_leftover(&self, a: &AgentProc) -> bool {
        self.leftover
            .iter()
            .any(|f| f.pid == a.pid && f.start_time == a.start_time)
    }

    pub fn leftover(&self) -> &[FrozenProc] {
        &self.leftover
    }

    /// Закрыт — размораживать больше незачем.
    pub fn forget(&mut self, pid: u32) {
        self.leftover.retain(|f| f.pid != pid);
    }

    pub fn take_leftover(&mut self) -> Vec<FrozenProc> {
        std::mem::take(&mut self.leftover)
    }

    /// Остался замороженным: отпустим, когда будет можно.
    pub fn hold(&mut self, f: FrozenProc) {
        if !self.leftover.contains(&f) {
            self.leftover.push(f);
        }
    }

    /// Разморозить не удалось — попробуем ещё раз. `true` — об этом ещё не сообщали.
    pub fn keep(&mut self, f: FrozenProc) -> bool {
        self.hold(f);
        let first = !self.reported.contains(&f);
        if first {
            self.reported.push(f);
        }
        first
    }

    /// Все ли уже записаны: тогда перед заморозкой писать на диск не нужно.
    /// Неудачную запись тоже не повторяем: следующая попытка — в конце шага.
    pub fn covers(&self, list: &[FrozenProc]) -> bool {
        list.iter().all(|f| self.tried.contains(f))
    }

    /// Записать `frozen` и оставшихся, если список изменился. Текст ошибки —
    /// только о первой неудаче подряд.
    pub fn sync(&mut self, frozen: impl Iterator<Item = FrozenProc>) -> Result<(), String> {
        self.reload();
        let want = sorted(self.leftover.iter().copied().chain(frozen).collect());
        self.reported.retain(|f| want.contains(f));
        self.tried.clone_from(&want);
        if want == self.saved {
            return Ok(());
        }
        match self.store.save(&want, self.unread) {
            Ok(()) => {
                self.saved = want;
                self.failing = false;
                self.unread = false;
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
    use std::sync::{Arc, Mutex};

    fn f(pid: u32) -> FrozenProc {
        FrozenProc { pid, start_time: 1 }
    }

    struct Broken;
    impl FrozenStore for Broken {
        fn load(&mut self) -> io::Result<Vec<FrozenProc>> {
            Err(io::Error::other("нет прав"))
        }
        fn save(&mut self, _: &[FrozenProc], _: bool) -> io::Result<()> {
            Err(io::Error::other("диск полон"))
        }
    }

    #[test]
    fn errors_reported_once() {
        let mut r = FrozenRecord::open(Box::new(Broken));
        assert_eq!(r.take_load_error().as_deref(), Some("нет прав"));
        assert_eq!(r.take_load_error(), None);
        assert_eq!(r.sync([f(7)].into_iter()), Err("диск полон".to_string()));
        assert_eq!(r.sync([f(7)].into_iter()), Ok(()));
        assert!(r.covers(&[f(7)]), "второй раз перед заморозкой не пробуем");
        assert!(r.keep(f(7)) && !r.keep(f(7)));
        assert_eq!(r.take_leftover(), [f(7)]);
    }

    /// Первое чтение не удаётся (файл занят), дальше — как есть.
    struct Busy(bool);
    impl FrozenStore for Busy {
        fn load(&mut self) -> io::Result<Vec<FrozenProc>> {
            if std::mem::replace(&mut self.0, false) {
                return Err(io::Error::other("занят"));
            }
            Ok(vec![f(9), f(3)])
        }
        fn save(&mut self, list: &[FrozenProc], _: bool) -> io::Result<()> {
            assert!(list.contains(&f(3)) && list.contains(&f(9)), "{list:?}");
            Ok(())
        }
    }

    #[test]
    fn unread_list_is_read_again_before_overwriting() {
        let mut r = FrozenRecord::open(Box::new(Busy(true)));
        assert!(!r.has_leftover());
        r.sync([f(7)].into_iter()).unwrap();
        assert_eq!(r.leftover(), [f(3), f(9)]);
        assert_eq!(r.take_load_error(), None, "прочитали — не тревожим");
        r.sync([f(7)].into_iter()).unwrap();
        assert_eq!(r.leftover().len(), 2, "второй раз не читаем");
        r.forget(3);
        assert_eq!(r.leftover(), [f(9)]);
    }

    struct Counting(Arc<Mutex<usize>>);
    impl FrozenStore for Counting {
        fn load(&mut self) -> io::Result<Vec<FrozenProc>> {
            Ok(vec![f(9), f(3)])
        }
        fn save(&mut self, _: &[FrozenProc], _: bool) -> io::Result<()> {
            *self.0.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[test]
    fn order_does_not_matter() {
        let writes: Arc<Mutex<usize>> = Arc::default();
        let mut r = FrozenRecord::open(Box::new(Counting(writes.clone())));
        r.take_leftover();
        r.sync([f(9), f(3), f(9)].into_iter()).unwrap();
        r.sync([f(3), f(9)].into_iter()).unwrap();
        assert_eq!(*writes.lock().unwrap(), 0, "тот же состав — не пишем");
        assert!(r.covers(&[f(9)]) && !r.covers(&[f(9), f(4)]));
        r.sync([f(3)].into_iter()).unwrap();
        assert_eq!(*writes.lock().unwrap(), 1);
    }
}
