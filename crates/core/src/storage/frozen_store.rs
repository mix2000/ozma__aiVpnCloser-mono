//! Список замороженных процессов в `frozen.json`: если программа упала или её
//! убили, следующий запуск разморозит оставшихся.

use std::io;
use std::path::PathBuf;

use super::atomic::write_atomic_no_sync;
use super::config_store::set_aside;
use crate::models::FrozenProc;

/// Хранилище списка замороженных (в тестах — в памяти).
pub trait FrozenStore: Send {
    /// Кто остался замороженным с прошлого запуска; нет записи — никого.
    fn load(&mut self) -> io::Result<Vec<FrozenProc>>;
    /// `keep_old` — прежняя запись не прочитана: не затирать, а отложить.
    fn save(&mut self, list: &[FrozenProc], keep_old: bool) -> io::Result<()>;
}

/// Список в файле; пустой список — файла нет.
pub struct FrozenFile {
    path: PathBuf,
}

impl FrozenFile {
    pub fn new(path: PathBuf) -> FrozenFile {
        FrozenFile { path }
    }

    fn set_aside(&self) -> io::Result<()> {
        match set_aside(&self.path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

impl FrozenStore for FrozenFile {
    /// Битый файл откладывается в `frozen.json.bad`. Пустой или из одних нулей
    /// остаётся после отключения питания (пишем без fsync): этих процессов уже нет.
    fn load(&mut self) -> io::Result<Vec<FrozenProc>> {
        let bytes = match std::fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        if bytes.iter().all(|b| *b == 0) {
            return Ok(Vec::new());
        }
        serde_json::from_slice(&bytes).map_err(|e| {
            // Не вышло — отложим перед записью поверх.
            let _ = self.set_aside();
            e.into()
        })
    }

    fn save(&mut self, list: &[FrozenProc], keep_old: bool) -> io::Result<()> {
        if list.is_empty() {
            if keep_old {
                return self.set_aside();
            }
            return match std::fs::remove_file(&self.path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        // Без fsync: ждать диск перед заморозкой нельзя, а после перезагрузки
        // этих процессов всё равно нет.
        write_atomic_no_sync(&self.path, &serde_json::to_vec(list)?, || {
            if keep_old {
                self.set_aside()?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_and_broken() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("frozen.json");
        let mut f = FrozenFile::new(path.clone());
        assert!(f.load().unwrap().is_empty());
        let list = [FrozenProc {
            pid: 7,
            start_time: 100,
        }];
        f.save(&list, false).unwrap();
        assert_eq!(FrozenFile::new(path.clone()).load().unwrap(), list);
        f.save(&[], false).unwrap();
        assert!(!path.exists(), "никого — файла нет");
        f.save(&[], false).unwrap();
        std::fs::write(&path, "[{").unwrap();
        assert!(f.load().is_err(), "битый — не молчим");
        assert!(!path.exists() && dir.path().join("frozen.json.bad").exists());
        assert!(f.load().unwrap().is_empty());
        for lost in [&b""[..], &[0; 16]] {
            std::fs::write(&path, lost).unwrap();
            assert!(f.load().unwrap().is_empty(), "питание отключали");
        }
    }

    #[test]
    fn unread_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("frozen.json");
        let bad = dir.path().join("frozen.json.bad");
        // Каталог вместо файла: чтение не удаётся на любой ОС.
        std::fs::create_dir(&path).unwrap();
        let mut f = FrozenFile::new(path.clone());
        assert!(f.load().is_err());
        assert!(path.exists(), "само чтение ничего не откладывает");
        let list = [FrozenProc {
            pid: 7,
            start_time: 100,
        }];
        // Новый список записать не удалось (имя временного файла занято
        // каталогом) — прежний остаётся на месте.
        let tmp = format!(".frozen.json.{}.tmp", std::process::id());
        let tmp = dir.path().join(tmp);
        std::fs::create_dir(&tmp).unwrap();
        assert!(f.save(&list, true).is_err());
        assert!(path.is_dir() && !bad.exists());
        std::fs::remove_dir(&tmp).unwrap();
        // Отложить не удалось (имя занято непустым каталогом) — не затираем.
        std::fs::remove_dir(&path).unwrap();
        std::fs::write(&path, "[{").unwrap();
        std::fs::create_dir_all(bad.join("x")).unwrap();
        assert!(f.load().is_err());
        assert!(f.save(&list, true).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"[{");
        std::fs::remove_dir_all(&bad).unwrap();
        f.save(&list, true).unwrap();
        assert_eq!(std::fs::read(&bad).unwrap(), b"[{");
        assert_eq!(f.load().unwrap(), list);
        std::fs::remove_file(&bad).unwrap();
        f.save(&[], true).unwrap();
        assert!(!path.exists() && bad.is_file());
        f.save(&[], true).unwrap();
    }
}
