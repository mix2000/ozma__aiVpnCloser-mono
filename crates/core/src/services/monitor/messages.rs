//! Тексты журнала и уведомлений монитора. `what` — описание целей
//! (`agents::summary`): «claude ×2 сессии (+5 вспомогательных процессов)».

use crate::models::{Action, CheckResult};
use crate::services::agents::procs_text;

pub const TITLE_VPN_DOWN: &str = "VPN пропал";
pub const TITLE_VPN_BACK: &str = "VPN вернулся";
pub const TITLE_CLOSED: &str = "Агенты закрыты";
pub const TITLE_BLOCKED: &str = "Агент запущен без VPN";

pub const BODY_BACK_IN_COUNTDOWN: &str = "Агенты продолжают работу";
pub const BODY_BACK_STUCK: &str =
    "Не всех агентов удалось разморозить, попробую ещё раз. Подробности — в журнале";
pub const BODY_BACK_AFTER_CLOSE: &str = "Агентов можно запускать снова";

pub const WHY_MONITORING_OFF: &str = "наблюдение выключено";
pub const WHY_NOTHING_WATCHED: &str = "не выбрано, за чем следить";
pub const WHY_SHUTDOWN: &str = "Программа завершается";
pub const WHY_LEFTOVER: &str = "Оставались замороженными";

/// «VPN пропал: процесс xray — xray не запущен; …» — по непрошедшим проверкам.
pub fn vpn_down(checks: &[CheckResult]) -> String {
    let failed: Vec<String> = checks
        .iter()
        .filter(|c| c.ok != Some(true))
        .map(|c| format!("{} — {}", c.label, c.detail))
        .collect();
    format!("VPN пропал: {}", failed.join("; "))
}

/// Уведомление при начале отсчёта.
pub fn countdown_notice(
    close_after_sec: u32,
    no_agents: bool,
    freeze: bool,
    dry_run: bool,
) -> String {
    let n = close_after_sec;
    let body = if no_agents {
        format!("Агентов сейчас нет; новые будут остановлены. Отсчёт {n} с")
    } else if freeze {
        format!("Агенты заморожены, закрою через {n} с")
    } else {
        format!("Закрою агентов через {n} с")
    };
    if dry_run {
        format!("{body} (только смотрю)")
    } else {
        body
    }
}

/// Заморозка: `block` — запущены, пока VPN нет; `dry_run` — «заморозил бы».
pub fn freeze(what: &str, block: bool, dry_run: bool) -> String {
    match (block, dry_run) {
        (true, true) => format!("Запущены без VPN — заморозил бы: {what}"),
        (false, true) => format!("Заморозил бы: {what}"),
        (true, false) => format!("Запущены без VPN — заморожены: {what}"),
        (false, false) => format!("Заморожены: {what}"),
    }
}

/// Закрытие: `block` — запущены, пока VPN нет; `dry_run` — «закрыл бы».
pub fn close(what: &str, block: bool, dry_run: bool) -> String {
    match (block, dry_run) {
        (true, true) => format!("Запущены без VPN — закрыл бы: {what}"),
        (false, true) => format!("Закрыл бы: {what}"),
        (true, false) => format!("Запущены без VPN — закрыты: {what}"),
        (false, false) => format!("Закрыто: {what}"),
    }
}

pub const NOTHING_TO_CLOSE: &str = "Время вышло — агентов нет";

/// VPN вернулся во время отсчёта. `resumed` — кого разморозили по-настоящему,
/// `would` — кого «разморозил бы» (только смотреть); оба `None` — никого.
pub fn vpn_back(resumed: Option<&str>, would: Option<&str>) -> String {
    match (resumed, would) {
        (Some(what), _) => format!("VPN вернулся — агенты продолжают: {what}"),
        (None, Some(what)) => format!("VPN вернулся — разморозил бы: {what}"),
        (None, None) => "VPN вернулся до закрытия".to_string(),
    }
}

pub const VPN_BACK_AFTER_CLOSE: &str = "VPN вернулся";

/// Добили SIGKILL тех, кто не закрылся сам.
pub fn killed_rest(n: usize) -> String {
    format!("Принудительно закрыты оставшиеся: {}", procs_text(n))
}

/// Разморозили при выключении наблюдения / выходе.
pub fn released(why: &str, n: usize) -> String {
    format!("{why} — агенты разморожены ({})", procs_text(n))
}

/// Список замороженных не записался: после падения их не разморозить.
pub fn frozen_not_saved(err: &str) -> String {
    format!("Не удалось записать список замороженных: {err}")
}

/// Список с прошлого запуска не прочитан: кто-то мог остаться замороженным.
pub fn frozen_not_loaded(err: &str) -> String {
    format!("Не удалось прочитать список замороженных с прошлого запуска: {err}. Если агент завис, закройте его вручную")
}

/// Ошибка действия над pid.
pub fn action_failed(action: Action, pid: u32, err: &str) -> String {
    format!("Не удалось {} pid {pid}: {err}", action.verb())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WatchItem;

    #[test]
    fn texts() {
        let checks = vec![
            CheckResult::new(WatchItem::Route, Some(false), "api.x.ai → en0"),
            CheckResult::new(
                WatchItem::Process {
                    name: "xray".into(),
                },
                Some(true),
                "ok",
            ),
            CheckResult::new(
                WatchItem::Process {
                    name: "sing-box".into(),
                },
                None,
                "?",
            ),
        ];
        assert_eq!(
            vpn_down(&checks),
            "VPN пропал: Трафик к агентам идёт через VPN — api.x.ai → en0; Программа sing-box — ?"
        );
        assert_eq!(
            countdown_notice(30, false, true, false),
            "Агенты заморожены, закрою через 30 с"
        );
        assert_eq!(
            countdown_notice(5, true, true, true),
            "Агентов сейчас нет; новые будут остановлены. Отсчёт 5 с (только смотрю)"
        );
        assert_eq!(
            countdown_notice(5, false, false, false),
            "Закрою агентов через 5 с"
        );
        assert_eq!(
            freeze("x", true, true),
            "Запущены без VPN — заморозил бы: x"
        );
        assert_eq!(close("x", false, false), "Закрыто: x");
        assert_eq!(vpn_back(None, None), "VPN вернулся до закрытия");
        assert_eq!(vpn_back(None, Some("x")), "VPN вернулся — разморозил бы: x");
        assert_eq!(
            killed_rest(2),
            "Принудительно закрыты оставшиеся: 2 процесса"
        );
        assert_eq!(
            released(WHY_SHUTDOWN, 1),
            "Программа завершается — агенты разморожены (1 процесс)"
        );
        assert_eq!(
            action_failed(Action::Kill, 5, "EPERM"),
            "Не удалось закрыть pid 5: EPERM"
        );
    }
}
