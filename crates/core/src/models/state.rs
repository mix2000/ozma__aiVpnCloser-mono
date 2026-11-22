//! Состояние для UI (`get_state`, событие `state-changed`).

use serde::{Deserialize, Serialize};

use super::agent::TargetKind;
use super::check::CheckResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// VPN есть.
    Ok,
    /// Проверки начали падать, ждём подтверждения (`confirmFailures`).
    Down,
    /// VPN пропал, идёт отсчёт до закрытия (агенты заморожены, если включено).
    Countdown,
    /// Агенты закрыты, VPN всё ещё нет.
    Closed,
    /// Наблюдение выключено.
    Paused,
    /// Не выбрано, за чем следить, или проверки пока без ответа.
    Unknown,
}

/// Цвет статуса — значок в трее и шапка окна.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusColor {
    Green,
    Yellow,
    Red,
    Gray,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub target: TargetKind,
    pub pid: u32,
    pub name: String,
    pub frozen: bool,
    /// true — сам агент (сессия), false — его потомок (MCP-сервер, команда).
    pub root: bool,
    /// pid корня дерева (для корня — свой pid).
    pub root_pid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub monitoring: bool,
    pub dry_run: bool,
    pub status: Status,
    pub countdown_left_sec: Option<u32>,
    pub checks: Vec<CheckResult>,
    pub agents: Vec<AgentInfo>,
    pub last_change_at: Option<String>,
    /// Цвет статуса (считает ядро — одинаково для трея и окна).
    pub color: StatusColor,
    /// Большая строка статуса: «VPN включён», «VPN пропал — закрою через 23 с».
    pub title: String,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            monitoring: false,
            dry_run: true,
            status: Status::Paused,
            countdown_left_sec: None,
            checks: Vec::new(),
            agents: Vec::new(),
            last_change_at: None,
            color: StatusColor::Gray,
            title: "Наблюдение выключено".into(),
        }
    }
}

impl AppState {
    /// Начальное состояние по настройкам (до первого шага монитора).
    pub fn initial(monitoring: bool, dry_run: bool) -> AppState {
        AppState {
            monitoring,
            dry_run,
            ..AppState::default()
        }
    }
}

/// Уведомление ОС (показывает app, если в настройках `notify`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape() {
        let s = AppState {
            agents: vec![AgentInfo {
                target: TargetKind::ClaudeCode,
                pid: 5,
                name: "claude".into(),
                frozen: true,
                root: true,
                root_pid: 5,
            }],
            countdown_left_sec: Some(3),
            ..AppState::default()
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["status"], "paused");
        assert_eq!(v["dryRun"], true);
        assert_eq!(v["countdownLeftSec"], 3);
        assert_eq!(v["lastChangeAt"], serde_json::Value::Null);
        assert_eq!(v["agents"][0]["target"], "claudeCode");
        assert_eq!(v["agents"][0]["frozen"], true);
        assert_eq!(v["agents"][0]["root"], true);
        assert_eq!(v["agents"][0]["rootPid"], 5);
        assert_eq!(v["color"], "gray");
        assert_eq!(v["title"], "Наблюдение выключено");
    }
}
