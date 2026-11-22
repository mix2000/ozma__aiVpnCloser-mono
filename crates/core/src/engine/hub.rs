//! Общее состояние движка между потоком монитора и вызовами app: настройки,
//! последнее состояние, журнал и подписчики. Рассылка изменений — здесь.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::models::{AppState, Config, Event, Notice};
use crate::services::monitor::TickOutput;
use crate::storage::EventLog;

pub type StateCb = Arc<dyn Fn(&AppState) + Send + Sync>;
pub type NoticeCb = Arc<dyn Fn(&Notice) + Send + Sync>;
pub type ConfigCb = Arc<dyn Fn(&Config) + Send + Sync>;

/// Номер подписки для `unsubscribe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub(super) u64);

/// Замок, переживающий панику держателя (данные простые, их можно читать).
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub struct Hub {
    pub config: Mutex<Config>,
    pub state: Mutex<AppState>,
    events: Mutex<EventLog>,
    subs: Mutex<Vec<(u64, StateCb)>>,
    notice_subs: Mutex<Vec<(u64, NoticeCb)>>,
    config_subs: Mutex<Vec<(u64, ConfigCb)>>,
    next_id: Mutex<u64>,
}

impl Hub {
    pub fn new(config: Config, events: EventLog) -> Hub {
        let state = AppState::initial(config.monitoring, config.dry_run);
        Hub {
            config: Mutex::new(config),
            state: Mutex::new(state),
            events: Mutex::new(events),
            subs: Mutex::new(Vec::new()),
            notice_subs: Mutex::new(Vec::new()),
            config_subs: Mutex::new(Vec::new()),
            next_id: Mutex::new(0),
        }
    }

    pub fn push_event(&self, ev: Event) {
        lock(&self.events).push(ev);
    }

    pub fn recent_events(&self, limit: usize) -> Vec<Event> {
        lock(&self.events).recent(limit)
    }

    /// Итог шага: события — в журнал, изменившееся состояние и уведомления —
    /// подписчикам.
    pub fn publish(&self, out: TickOutput) {
        for ev in out.events {
            self.push_event(ev);
        }
        let changed = {
            let mut st = lock(&self.state);
            if *st != out.state {
                *st = out.state.clone();
                true
            } else {
                false
            }
        };
        if changed {
            let subs: Vec<StateCb> = lock(&self.subs).iter().map(|(_, f)| f.clone()).collect();
            for f in subs {
                f(&out.state);
            }
        }
        if !out.notices.is_empty() {
            let subs: Vec<NoticeCb> = lock(&self.notice_subs)
                .iter()
                .map(|(_, f)| f.clone())
                .collect();
            for n in &out.notices {
                for f in &subs {
                    f(n);
                }
            }
        }
    }

    fn id(&self) -> u64 {
        let mut n = lock(&self.next_id);
        *n += 1;
        *n
    }

    pub fn subscribe(&self, f: StateCb) -> SubscriptionId {
        let id = self.id();
        lock(&self.subs).push((id, f));
        SubscriptionId(id)
    }

    pub fn subscribe_notices(&self, f: NoticeCb) -> SubscriptionId {
        let id = self.id();
        lock(&self.notice_subs).push((id, f));
        SubscriptionId(id)
    }

    pub fn subscribe_config(&self, f: ConfigCb) -> SubscriptionId {
        let id = self.id();
        lock(&self.config_subs).push((id, f));
        SubscriptionId(id)
    }

    /// Сохранённые настройки — подписчикам (автозапуск ОС и т.п.).
    pub fn publish_config(&self, cfg: &Config) {
        let subs: Vec<ConfigCb> = lock(&self.config_subs)
            .iter()
            .map(|(_, f)| f.clone())
            .collect();
        for f in subs {
            f(cfg);
        }
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        lock(&self.subs).retain(|(i, _)| *i != id.0);
        lock(&self.notice_subs).retain(|(i, _)| *i != id.0);
        lock(&self.config_subs).retain(|(i, _)| *i != id.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EventKind, Status};

    #[test]
    fn publish_only_changes() {
        let hub = Hub::new(Config::default(), EventLog::in_memory());
        let seen: Arc<Mutex<Vec<Status>>> = Arc::default();
        let notices: Arc<Mutex<usize>> = Arc::default();
        let s = seen.clone();
        let id = hub.subscribe(Arc::new(move |st: &AppState| {
            s.lock().unwrap().push(st.status)
        }));
        let n = notices.clone();
        hub.subscribe_notices(Arc::new(move |_: &Notice| *n.lock().unwrap() += 1));

        // То же состояние — подписчиков не дёргаем, событие в журнал всё равно.
        hub.publish(TickOutput {
            state: AppState::default(),
            events: vec![Event::new("2026-11-22T10:00:00Z", EventKind::Info, "x")],
            notices: vec![],
        });
        assert!(seen.lock().unwrap().is_empty());
        assert_eq!(hub.recent_events(5).len(), 1);

        let ok = AppState {
            status: Status::Ok,
            ..AppState::default()
        };
        hub.publish(TickOutput {
            state: ok.clone(),
            events: vec![],
            notices: vec![Notice {
                title: "t".into(),
                body: "b".into(),
            }],
        });
        assert_eq!(*seen.lock().unwrap(), [Status::Ok]);
        assert_eq!(*notices.lock().unwrap(), 1);

        hub.unsubscribe(id);
        hub.publish(TickOutput::default());
        assert_eq!(seen.lock().unwrap().len(), 1);

        let cfgs: Arc<Mutex<Vec<bool>>> = Arc::default();
        let c = cfgs.clone();
        hub.subscribe_config(Arc::new(move |cfg: &Config| {
            c.lock().unwrap().push(cfg.autostart)
        }));
        hub.publish_config(&Config {
            autostart: true,
            ..Config::default()
        });
        assert_eq!(*cfgs.lock().unwrap(), [true]);
    }
}
