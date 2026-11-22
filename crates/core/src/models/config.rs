//! Настройки: что считать «VPN включён», что делать, когда он пропал.
//! JSON в camelCase ровно по контракту core ↔ app. Проверка — в
//! `services::config_validation`, чтение и запись — в `storage::config_store`.

use serde::{Deserialize, Serialize};

/// Хосты агентов по умолчанию для проверки `route`.
pub const DEFAULT_HOSTS: &[&str] = &[
    "api.anthropic.com",
    "claude.ai",
    "api.openai.com",
    "chatgpt.com",
    "api.x.ai",
    "grok.com",
];

/// Пункт, за которым следим.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WatchItem {
    /// Туннельный интерфейс. `addr` — IPv4 (или `IPv4/префикс`); совпадение по
    /// адресу важнее имени, потому что `utunN` меняется после перезапуска VPN.
    Interface {
        #[serde(default)]
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        addr: Option<String>,
    },
    /// Процесс VPN-клиента по имени.
    Process { name: String },
    /// Системный VPN-сервис (macOS scutil / Windows RAS).
    SystemVpn {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
    },
    /// Трафик к хостам агентов идёт через туннель.
    Route,
}

impl WatchItem {
    /// Название пункта — одно для окна (`CheckResult.label`) и журнала.
    pub fn label(&self) -> String {
        match self {
            WatchItem::Interface { name, addr } => match (name.is_empty(), addr) {
                (false, Some(a)) => format!("Интерфейс {name} ({a})"),
                (false, None) => format!("Интерфейс {name}"),
                (true, Some(a)) => format!("Интерфейс с адресом {a}"),
                (true, None) => "Интерфейс".into(),
            },
            WatchItem::Process { name } => format!("Программа {name}"),
            WatchItem::SystemVpn { id, name } => {
                if name.is_empty() {
                    format!("Системный VPN {id}")
                } else {
                    format!("Системный VPN «{name}»")
                }
            }
            WatchItem::Route => "Трафик к агентам идёт через VPN".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Нужны все пункты.
    #[default]
    All,
    /// Достаточно одного.
    Any,
}

/// Какие агенты закрывать.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Targets {
    pub claude_code: bool,
    pub claude_desktop: bool,
    pub codex_cli: bool,
    pub codex_desktop: bool,
    pub grok: bool,
}

impl Default for Targets {
    fn default() -> Self {
        Targets {
            claude_code: true,
            claude_desktop: true,
            codex_cli: true,
            codex_desktop: true,
            grok: true,
        }
    }
}

impl Targets {
    pub fn any(&self) -> bool {
        self.claude_code || self.claude_desktop || self.codex_cli || self.codex_desktop || self.grok
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub watch: Vec<WatchItem>,
    pub mode: Mode,
    pub hosts: Vec<String>,
    pub targets: Targets,
    pub freeze_immediately: bool,
    pub close_after_sec: u32,
    pub kill_grace_sec: u32,
    pub block_while_down: bool,
    pub poll_ms: u32,
    pub confirm_failures: u32,
    pub dry_run: bool,
    pub notify: bool,
    pub autostart: bool,
    pub monitoring: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            watch: Vec::new(),
            mode: Mode::All,
            hosts: DEFAULT_HOSTS.iter().map(|s| s.to_string()).collect(),
            targets: Targets::default(),
            freeze_immediately: true,
            close_after_sec: 30,
            kill_grace_sec: 5,
            block_while_down: true,
            poll_ms: 1000,
            confirm_failures: 2,
            dry_run: true,
            notify: true,
            autostart: false,
            monitoring: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = Config::default();
        assert!(!c.monitoring);
        assert!(c.dry_run);
        assert!(c.watch.is_empty());
        assert_eq!(c.mode, Mode::All);
        assert_eq!(c.poll_ms, 1000);
        assert_eq!(c.confirm_failures, 2);
        assert_eq!(c.close_after_sec, 30);
        assert_eq!(c.kill_grace_sec, 5);
        assert!(c.freeze_immediately && c.block_while_down);
        assert_eq!(c.hosts.len(), 6);
    }

    #[test]
    fn json_contract_camel_case() {
        let c = Config {
            watch: vec![
                WatchItem::Interface {
                    name: "utun233".into(),
                    addr: Some("172.19.0.1".into()),
                },
                WatchItem::Process {
                    name: "xray".into(),
                },
                WatchItem::SystemVpn {
                    id: "X-1".into(),
                    name: "V2BOX".into(),
                },
                WatchItem::Route,
            ],
            ..Config::default()
        };
        let v = serde_json::to_value(&c).unwrap();
        for key in [
            "watch",
            "mode",
            "hosts",
            "targets",
            "freezeImmediately",
            "closeAfterSec",
            "killGraceSec",
            "blockWhileDown",
            "pollMs",
            "confirmFailures",
            "dryRun",
            "notify",
            "autostart",
            "monitoring",
        ] {
            assert!(v.get(key).is_some(), "нет ключа {key}");
        }
        assert_eq!(v["mode"], "all");
        assert_eq!(v["targets"]["claudeCode"], true);
        assert_eq!(v["targets"]["codexDesktop"], true);
        assert_eq!(v["watch"][0]["kind"], "interface");
        assert_eq!(v["watch"][0]["addr"], "172.19.0.1");
        assert_eq!(v["watch"][2]["kind"], "system_vpn");
        assert_eq!(v["watch"][3], serde_json::json!({"kind": "route"}));
        let back: Config = serde_json::from_value(v).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn partial_json_gets_defaults() {
        let c: Config = serde_json::from_str(r#"{"monitoring":true,"mode":"any"}"#).unwrap();
        assert!(c.monitoring);
        assert_eq!(c.mode, Mode::Any);
        assert_eq!(c.poll_ms, 1000);
        let item: WatchItem =
            serde_json::from_str(r#"{"kind":"interface","name":"utun4"}"#).unwrap();
        assert_eq!(
            item,
            WatchItem::Interface {
                name: "utun4".into(),
                addr: None
            }
        );
    }

    #[test]
    fn labels() {
        assert_eq!(WatchItem::Route.label(), "Трафик к агентам идёт через VPN");
        let p = WatchItem::Process {
            name: "xray".into(),
        };
        assert_eq!(p.label(), "Программа xray");
        let i = WatchItem::Interface {
            name: String::new(),
            addr: Some("10.0.0.1".into()),
        };
        assert_eq!(i.label(), "Интерфейс с адресом 10.0.0.1");
        let v = WatchItem::SystemVpn {
            id: "X".into(),
            name: String::new(),
        };
        assert_eq!(v.label(), "Системный VPN X");
    }
}
