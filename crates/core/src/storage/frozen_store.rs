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
    fn save(&mut self, list: &[FrozenProc]) -> io::Result<()>;
}

/// Список в файле; пустой список — файла нет.
pub struct FrozenFile {
    path: PathBuf,
    /// Файл есть, но не прочитан: перед записью поверх — отложить.
    unread: bool,
}

impl FrozenFile {
    pub fn new(path: PathBuf) -> FrozenFile {
        FrozenFile {
            path,
            unread: false,
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
            Err(e) => {
                self.unread = true;
                return Err(e);
            }
        };
        self.unread = false;
        if bytes.iter().all(|b| *b == 0) {
            return Ok(Vec::new());
        }
        serde_json::from_slice(&bytes).map_err(|e| {
            set_aside(&self.path);
            e.into()
        })
    }

    fn save(&mut self, list: &[FrozenProc]) -> io::Result<()> {
        if std::mem::take(&mut self.unread) {
            set_aside(&self.path);
        }
        if list.is_empty() {
            return match std::fs::remove_file(&self.path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        // Без fsync: ждать диск перед заморозкой нельзя, а после перезагрузки
        // этих процессов всё равно нет.
        write_atomic_no_sync(&self.path, &serde_json::to_vec(list)?)
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
        f.save(&list).unwrap();
        assert_eq!(FrozenFile::new(path.clone()).load().unwrap(), list);
        f.save(&[]).unwrap();
        assert!(!path.exists(), "никого — файла нет");
        f.save(&[]).unwrap();
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
        // Каталог вместо файла: чтение не удаётся на любой ОС.
        std::fs::create_dir(&path).unwrap();
        let mut f = FrozenFile::new(path.clone());
        assert!(f.load().is_err());
        assert!(path.exists(), "само чтение ничего не откладывает");
        let list = [FrozenProc {
            pid: 7,
            start_time: 100,
        }];
        f.save(&list).unwrap();
        assert!(dir.path().join("frozen.json.bad").is_dir());
        assert_eq!(f.load().unwrap(), list);
    }
}
