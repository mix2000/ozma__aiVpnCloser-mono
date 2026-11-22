//! Состояние ядра в виде, нужном значку и меню трея. Цвет и заголовок
//! считает ядро (`AppState.color`, `AppState.title`) — здесь только подпись.

use vpn_closer_core::{sessions_text, AppState, StatusColor};

/// Состояние для UI + то, что нужно значку.
#[derive(Debug, Clone)]
pub struct StateView {
    pub state: AppState,
    pub color: StatusColor,
    /// Строка для меню и подсказки значка: статус + число сессий агентов.
    pub tray_text: String,
}

impl StateView {
    pub fn from_state(state: AppState) -> StateView {
        // В AppState.agents — только сами сессии (корни деревьев).
        let sessions = state.agents.len();
        let tray_text = if state.monitoring && sessions > 0 {
            format!("{} · {}", state.title, sessions_text(sessions))
        } else {
            state.title.clone()
        };
        StateView {
            color: state.color,
            state,
            tray_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vpn_closer_core::{AgentInfo, TargetKind};

    fn agent(pid: u32) -> AgentInfo {
        AgentInfo {
            target: TargetKind::ClaudeCode,
            pid,
            name: "claude".into(),
            frozen: false,
            root: true,
            root_pid: pid,
        }
    }

    #[test]
    fn tray_text_counts_sessions() {
        let v = StateView::from_state(AppState {
            monitoring: true,
            agents: (1..=15).map(agent).collect(),
            color: StatusColor::Green,
            title: "VPN включён".into(),
            ..AppState::default()
        });
        assert_eq!(v.tray_text, "VPN включён · 15 сессий");
        assert_eq!(v.color, StatusColor::Green);
    }

    #[test]
    fn title_only_when_off_or_no_sessions() {
        let v = StateView::from_state(AppState {
            agents: vec![agent(1)],
            ..AppState::default()
        });
        assert_eq!(v.tray_text, "Наблюдение выключено");
        assert_eq!(v.color, StatusColor::Gray);
    }
}
