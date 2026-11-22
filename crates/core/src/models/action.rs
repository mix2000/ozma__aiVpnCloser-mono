//! Действия над процессами и отчёт о них.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Freeze,
    Resume,
    /// Мягко: Unix SIGCONT + SIGTERM; Windows TerminateProcess.
    Terminate,
    /// Жёстко: SIGKILL / TerminateProcess.
    Kill,
}

impl Action {
    /// Глагол для сообщения об ошибке: «Не удалось заморозить pid …».
    pub fn verb(&self) -> &'static str {
        match self {
            Action::Freeze => "заморозить",
            Action::Resume => "разморозить",
            Action::Terminate | Action::Kill => "закрыть",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionReport {
    pub ok: Vec<u32>,
    /// pid и причина по-русски (или текст ошибки ОС).
    pub failed: Vec<(u32, String)>,
    pub dry_run: bool,
}
