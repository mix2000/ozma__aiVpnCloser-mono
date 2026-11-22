//! Цвет и строка статуса — одни для значка в трее и шапки окна.

use crate::models::{Status, StatusColor};

/// Цвет и заголовок по состоянию наблюдения.
pub fn status_view(monitoring: bool, status: Status, left: Option<u32>) -> (StatusColor, String) {
    if !monitoring || status == Status::Paused {
        return (StatusColor::Gray, "Наблюдение выключено".into());
    }
    match status {
        Status::Ok => (StatusColor::Green, "VPN включён".into()),
        // «down» — проверки начали падать, ждём подтверждения.
        Status::Down => (StatusColor::Yellow, "VPN пропадает…".into()),
        Status::Countdown => (
            StatusColor::Yellow,
            match left {
                Some(n) => format!("VPN пропал — закрою через {n} с"),
                None => "VPN пропал — скоро закрою агентов".into(),
            },
        ),
        Status::Closed => (StatusColor::Red, "Агенты закрыты, VPN нет".into()),
        Status::Paused | Status::Unknown => (StatusColor::Gray, "Проверяю VPN…".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_and_titles() {
        assert_eq!(status_view(false, Status::Ok, None).0, StatusColor::Gray);
        assert_eq!(status_view(true, Status::Paused, None).0, StatusColor::Gray);
        assert_eq!(
            status_view(true, Status::Ok, None),
            (StatusColor::Green, "VPN включён".into())
        );
        assert_eq!(
            status_view(true, Status::Countdown, Some(23)),
            (StatusColor::Yellow, "VPN пропал — закрою через 23 с".into())
        );
        assert_eq!(
            status_view(true, Status::Countdown, None).1,
            "VPN пропал — скоро закрою агентов"
        );
        assert_eq!(
            status_view(true, Status::Down, None),
            (StatusColor::Yellow, "VPN пропадает…".into())
        );
        assert_eq!(status_view(true, Status::Closed, None).0, StatusColor::Red);
        assert_eq!(
            status_view(true, Status::Unknown, None),
            (StatusColor::Gray, "Проверяю VPN…".into())
        );
    }
}
