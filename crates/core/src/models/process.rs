//! Снимок процесса. Вся логика над процессами работает с `Vec<ProcInfo>`, чтобы
//! её можно было проверять на подставных списках, а не на живой системе.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProcInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    /// Пользователь (uid на Unix, SID на Windows) строкой; None — неизвестно.
    pub uid: Option<String>,
    pub name: String,
    pub exe: Option<String>,
    pub cmd: Vec<String>,
    /// Время запуска (секунды с эпохи) — защита от повторного использования pid.
    pub start_time: u64,
}

/// Имя файла без пути и без `.exe`, в нижнем регистре.
pub fn base_name(s: &str) -> String {
    let base = s.rsplit(['/', '\\']).next().unwrap_or(s);
    let lower = base.to_lowercase();
    lower
        .strip_suffix(".exe")
        .map(str::to_string)
        .unwrap_or(lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_names() {
        assert_eq!(base_name("/usr/local/bin/claude"), "claude");
        assert_eq!(base_name(r"C:\Users\a\Codex.EXE"), "codex");
        assert_eq!(base_name("xray"), "xray");
    }
}
