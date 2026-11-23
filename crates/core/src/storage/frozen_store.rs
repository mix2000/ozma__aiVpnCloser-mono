//! Список замороженных процессов в `frozen.json`: если программа упала или её
//! убили, следующий запуск разморозит оставшихся.

use std::io;
use std::path::PathBuf;

use super::atomic::write_atomic;
use crate::models::FrozenProc;

/// Хранилище списка замороженных (в тестах — в памяти).
pub trait FrozenStore: Send {
    /// Кто остался замороженным с прошлого запуска; нет записи или она битая — никого.
    fn load(&self) -> Vec<FrozenProc>;
    fn save(&mut self, list: &[FrozenProc]) -> io::Result<()>;
}

/// Список в файле; пустой список — файла нет.
pub struct FrozenFile {
    path: PathBuf,
}

impl FrozenFile {
    pub fn new(path: PathBuf) -> FrozenFile {
        FrozenFile { path }
    }
}

impl FrozenStore for FrozenFile {
    fn load(&self) -> Vec<FrozenProc> {
        std::fs::read(&self.path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn save(&mut self, list: &[FrozenProc]) -> io::Result<()> {
        if list.is_empty() {
            return match std::fs::remove_file(&self.path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        write_atomic(&self.path, &serde_json::to_vec(list)?)
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
        assert!(f.load().is_empty());
        let list = [FrozenProc {
            pid: 7,
            start_time: 100,
        }];
        f.save(&list).unwrap();
        assert_eq!(FrozenFile::new(path.clone()).load(), list);
        f.save(&[]).unwrap();
        assert!(!path.exists(), "никого — файла нет");
        f.save(&[]).unwrap();
        std::fs::write(&path, "[{").unwrap();
        assert!(f.load().is_empty());
    }
}
