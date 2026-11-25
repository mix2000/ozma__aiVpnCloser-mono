//! `Engine` — фасад ядра для app: по методу на каждую команду контракта.
//!
//! Собирает сервисы (монитор, проверки, действия), хранилище и платформу.
//! Монитор крутится в своём потоке (`poller`) с интервалом `pollMs`;
//! изменения состояния рассылаются подписчикам (`hub`), app пушит их в UI
//! событием `state-changed`.

mod hub;
mod parts;
mod poller;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use hub::SubscriptionId;
pub use parts::PlatformParts;
#[cfg(feature = "test-util")]
pub use parts::ProcFactory;

use crate::error::{Error, Result};
use crate::models::{
    AgentInfo, AgentProc, AppState, Config, Discovery, Event, EventKind, Limits, Notice, SelfCtx,
};
use crate::platform::{paths, ProcSource};
use crate::services::agents;
use crate::services::config_changes::config_change_events;
use crate::services::config_validation::{validate, LIMITS};
use crate::services::discovery;
use crate::services::monitor::MonitorParts;
use crate::services::owners::OwnerResolver;
use crate::storage::{self, EventLog, CONFIG_FILE, EVENTS_FILE};
use hub::{lock, Hub};
use poller::Poller;

pub struct Engine {
    data_dir: PathBuf,
    platform: PlatformParts,
    hub: Arc<Hub>,
    poller: Mutex<Option<Poller>>,
    /// Снимки процессов для `discover` и `test_targets`.
    procs: Mutex<Box<dyn ProcSource>>,
    /// Владельцы туннелей для `discover` (кеш дорогих источников).
    owners: Mutex<OwnerResolver>,
    /// Чтение-изменение-запись настроек — по одному (переключатель в трее и окне).
    writes: Mutex<()>,
}

impl Engine {
    /// Открыть папку данных на настоящей ОС. Монитор не запускается.
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Engine> {
        Engine::with_platform(data_dir, PlatformParts::os())
    }

    /// Открыть папку данных: прочитать конфиг (битый — отложить в
    /// `config.json.bad` и начать с умолчаний) и журнал.
    pub fn with_platform(data_dir: impl Into<PathBuf>, platform: PlatformParts) -> Result<Engine> {
        let data_dir = data_dir.into();
        storage::ensure_dir(&data_dir)?;
        let mut events = EventLog::open(&data_dir.join(EVENTS_FILE));
        let config = load_or_set_aside(&data_dir, &mut events, &platform);
        Ok(Engine {
            procs: Mutex::new((platform.procs)()),
            owners: Mutex::new(platform.owner_resolver()),
            data_dir,
            platform,
            hub: Arc::new(Hub::new(config, events)),
            poller: Mutex::new(None),
            writes: Mutex::new(()),
        })
    }

    /// Папка данных: `$AI_VPN_CLOSER_DATA_DIR` или `dirs::config_dir()/ozma-ai-vpn-closer`.
    pub fn with_default_dir() -> Result<Engine> {
        let dir = paths::default_data_dir().ok_or(Error::NoDataDir)?;
        Engine::new(dir)
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Запустить монитор на платформе движка.
    pub fn start(&self) {
        self.start_with(self.platform.monitor_parts(&self.data_dir));
    }

    /// Запустить монитор с заданными зависимостями (тесты). Повторный вызов
    /// ничего не делает.
    pub fn start_with(&self, parts: MonitorParts) {
        let mut slot = lock(&self.poller);
        if slot.is_some() {
            return;
        }
        let cfg = self.get_config();
        *slot = Some(Poller::spawn(Arc::clone(&self.hub), cfg, parts));
    }

    /// Остановить монитор (замороженные агенты будут разморожены). Вызывать
    /// при выходе из программы; также вызывается из `Drop`.
    pub fn shutdown(&self) {
        let poller = lock(&self.poller).take();
        if let Some(p) = poller {
            p.stop();
        }
    }

    pub fn get_state(&self) -> AppState {
        lock(&self.hub.state).clone()
    }

    pub fn get_config(&self) -> Config {
        lock(&self.hub.config).clone()
    }

    /// Пределы числовых настроек (для полей в UI).
    pub fn limits(&self) -> Limits {
        LIMITS
    }

    /// Найденное на машине (интерфейсы, процессы, системные VPN).
    pub fn discover(&self) -> Discovery {
        let procs = lock(&self.procs).snapshot();
        discovery::discover(self.platform.net.as_ref(), &mut lock(&self.owners), &procs)
    }

    /// Проверить, сохранить, применить.
    pub fn save_config(&self, cfg: Config) -> Result<Config> {
        let _w = lock(&self.writes);
        self.save_locked(cfg)
    }

    /// Включить/выключить наблюдение (сохраняется в конфиг).
    pub fn set_monitoring(&self, on: bool) -> Result<AppState> {
        let _w = lock(&self.writes);
        let mut cfg = self.get_config();
        cfg.monitoring = on;
        self.save_locked(cfg)?;
        Ok(self.get_state())
    }

    /// Переключить наблюдение атомарно (меню трея): прочитать и записать под одним замком.
    pub fn toggle_monitoring(&self) -> Result<AppState> {
        let _w = lock(&self.writes);
        let mut cfg = self.get_config();
        cfg.monitoring = !cfg.monitoring;
        self.save_locked(cfg)?;
        Ok(self.get_state())
    }

    fn save_locked(&self, cfg: Config) -> Result<Config> {
        let cfg = validate(cfg)?;
        storage::save_config(&cfg, &self.data_dir.join(CONFIG_FILE))?;
        let old = std::mem::replace(&mut *lock(&self.hub.config), cfg.clone());
        let at = self.platform.clock.stamp();
        for (kind, text) in config_change_events(&old, &cfg) {
            self.hub.push_event(Event::new(at.clone(), kind, text));
        }
        self.apply_to_monitor(cfg.clone());
        self.hub.publish_config(&cfg);
        Ok(cfg)
    }

    /// Отправить конфиг монитору и дождаться шага с ним; без монитора —
    /// поправить флаги в состоянии.
    fn apply_to_monitor(&self, cfg: Config) {
        let handle = lock(&self.poller).as_ref().map(Poller::handle);
        match handle {
            Some(h) => h.apply(cfg),
            None => {
                let mut st = lock(&self.hub.state);
                st.monitoring = cfg.monitoring;
                st.dry_run = cfg.dry_run;
            }
        }
    }

    /// Какие процессы были бы затронуты сейчас (все, включая потомков), без действий.
    pub fn test_targets(&self) -> Vec<AgentInfo> {
        let snap = lock(&self.procs).snapshot();
        let me = SelfCtx::for_pid(self.platform.self_pid, &snap);
        let found = agents::collect(&snap, &self.get_config().targets, &me);
        with_frozen(found, &self.get_state())
    }

    /// Последние события, новые первыми.
    pub fn recent_events(&self, limit: usize) -> Vec<Event> {
        self.hub.recent_events(limit)
    }

    /// Подписка на изменения состояния (вызывается из потока монитора).
    pub fn subscribe(&self, f: impl Fn(&AppState) + Send + Sync + 'static) -> SubscriptionId {
        self.hub.subscribe(Arc::new(f))
    }

    /// Подписка на уведомления ОС (уже отфильтрованы по `notify`).
    pub fn subscribe_notices(&self, f: impl Fn(&Notice) + Send + Sync + 'static) -> SubscriptionId {
        self.hub.subscribe_notices(Arc::new(f))
    }

    /// Подписка на сохранённые настройки.
    pub fn subscribe_config(&self, f: impl Fn(&Config) + Send + Sync + 'static) -> SubscriptionId {
        self.hub.subscribe_config(Arc::new(f))
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        self.hub.unsubscribe(id);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Прочитать и проверить конфиг; битый — отложить в `config.json.bad`,
/// записать ошибку в журнал и начать с умолчаний.
fn load_or_set_aside(data_dir: &Path, events: &mut EventLog, platform: &PlatformParts) -> Config {
    let cfg_path = data_dir.join(CONFIG_FILE);
    match storage::load_config(&cfg_path).and_then(validate) {
        Ok(c) => c,
        Err(err) => {
            let aside = match storage::set_aside(&cfg_path) {
                Ok(bad) => format!("Файл отложен в {}", bad.display()),
                Err(e) => format!("Отложить файл не удалось ({e})"),
            };
            events.push(Event::new(
                platform.clock.stamp(),
                EventKind::Error,
                format!("{err}. {aside}, настройки сброшены"),
            ));
            Config::default()
        }
    }
}

/// Цели для `test_targets`: заморожен ли — из текущего состояния монитора.
fn with_frozen(found: Vec<AgentProc>, state: &AppState) -> Vec<AgentInfo> {
    let frozen: Vec<u32> = state
        .agents
        .iter()
        .filter(|a| a.frozen)
        .map(|a| a.pid)
        .collect();
    found
        .into_iter()
        .map(|a| AgentInfo {
            target: a.target,
            pid: a.pid,
            frozen: frozen.contains(&a.pid),
            root: a.is_root(),
            root_pid: a.root,
            name: a.name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TargetKind;

    #[test]
    fn test_targets_mark_frozen_roots() {
        let a = |pid, depth, root| AgentProc {
            target: TargetKind::Grok,
            pid,
            name: "grok".into(),
            start_time: 1,
            root,
            depth,
        };
        let state = AppState {
            agents: vec![AgentInfo {
                target: TargetKind::Grok,
                pid: 10,
                name: "grok".into(),
                frozen: true,
                root: true,
                root_pid: 10,
            }],
            ..AppState::default()
        };
        let out = with_frozen(vec![a(10, 0, 10), a(11, 1, 10)], &state);
        assert!(out[0].frozen && out[0].root && out[0].root_pid == 10);
        assert!(!out[1].frozen && !out[1].root && out[1].root_pid == 10);
    }
}
