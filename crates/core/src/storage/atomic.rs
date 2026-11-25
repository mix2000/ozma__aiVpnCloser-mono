//! Атомарная запись файла: во временный рядом, fsync (если нужен), rename.

use std::io::Write;
use std::path::Path;

pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    write(path, data, true, || Ok(()))
}

/// Без fsync: переживает падение программы, но не отключение питания.
/// `before` — перед самой заменой, когда новое уже записано; ошибка — не заменяем.
pub fn write_atomic_no_sync(
    path: &Path,
    data: &[u8],
    before: impl FnOnce() -> std::io::Result<()>,
) -> std::io::Result<()> {
    write(path, data, false, before)
}

fn write(
    path: &Path,
    data: &[u8],
    sync: bool,
    before: impl FnOnce() -> std::io::Result<()>,
) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(data)?;
        if sync {
            f.sync_all()?;
        }
    }
    match before().and_then(|()| std::fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a/b.json");
        write_atomic(&p, b"1").unwrap();
        write_atomic_no_sync(&p, b"22", || Ok(())).unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"22");
        let stop = || Err(std::io::Error::other("нельзя"));
        assert!(write_atomic_no_sync(&p, b"333", stop).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"22", "не заменили");
        let leftovers: Vec<_> = std::fs::read_dir(p.parent().unwrap())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }
}
