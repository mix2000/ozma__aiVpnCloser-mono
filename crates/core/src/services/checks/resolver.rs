//! Резолв хостов агентов в фоне с кешем на 60 с: медленный DNS не задерживает опрос.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::platform::DnsResolver;

const RESOLVE_TTL: Duration = Duration::from_secs(60);
/// Старые адреса ещё годятся, пока новый резолв не удался.
const RESOLVE_STALE: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Default)]
struct Resolved {
    ips: Vec<IpAddr>,
    ips_at: Option<Instant>,
    err: Option<String>,
    attempt_at: Option<Instant>,
    inflight: bool,
    /// Номер попытки, по которой посчитаны ошибки (чтобы одна неудача не
    /// засчитывалась на каждом опросе).
    generation: u64,
}

pub enum HostIps {
    Ready(Vec<IpAddr>),
    /// Ошибка резолва; `u64` — номер попытки.
    Failed(String, u64),
    Pending,
}

/// Кеш резолва поверх [`DnsResolver`] (сам резолв — в фоне, его делает платформа).
#[derive(Clone)]
pub struct Resolver {
    dns: Arc<dyn DnsResolver>,
    cache: Arc<Mutex<HashMap<String, Resolved>>>,
}

impl Resolver {
    pub fn new(dns: Arc<dyn DnsResolver>) -> Self {
        Resolver {
            dns,
            cache: Arc::default(),
        }
    }

    pub fn lookup(&self, host: &str) -> HostIps {
        let start = {
            let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            let entry = cache.entry(host.to_string()).or_default();
            let fresh = entry.attempt_at.is_some_and(|t| t.elapsed() < RESOLVE_TTL);
            let start = !fresh && !entry.inflight;
            if start {
                entry.inflight = true;
                entry.attempt_at = Some(Instant::now());
            }
            start
        };
        // Вне замка: подставной резолвер отвечает сразу, в этом же потоке.
        if start {
            self.spawn_resolve(host.to_string());
        }
        let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        current(&cache[host])
    }

    fn spawn_resolve(&self, host: String) {
        let cache = Arc::clone(&self.cache);
        let key = host.clone();
        self.dns.resolve_in_background(
            host,
            Box::new(move |res| {
                let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
                store(c.entry(key).or_default(), res);
            }),
        );
    }

    /// Подставить неудачную попытку резолва (тесты).
    #[cfg(test)]
    pub(crate) fn set_failed(&self, host: &str, err: &str, generation: u64) {
        let mut c = self.cache.lock().unwrap();
        c.insert(
            host.to_string(),
            Resolved {
                err: Some(err.into()),
                attempt_at: Some(Instant::now()),
                generation,
                ..Default::default()
            },
        );
    }
}

/// Записать ответ резолва.
fn store(e: &mut Resolved, res: std::io::Result<Vec<IpAddr>>) {
    e.inflight = false;
    e.generation += 1;
    match res {
        Ok(mut ips) if !ips.is_empty() => {
            ips.sort_by_key(|ip| !ip.is_ipv4());
            ips.dedup();
            e.ips = ips;
            e.ips_at = Some(Instant::now());
            e.err = None;
        }
        Ok(_) => e.err = Some("адрес не найден".into()),
        Err(err) => e.err = Some(err.to_string()),
    }
}

/// Что отдать по записи кеша сейчас.
fn current(entry: &Resolved) -> HostIps {
    let usable = entry
        .ips_at
        .is_some_and(|t| t.elapsed() < RESOLVE_STALE && !entry.ips.is_empty());
    if entry.err.is_none() && usable {
        return HostIps::Ready(entry.ips.clone());
    }
    match &entry.err {
        Some(e) => HostIps::Failed(e.clone(), entry.generation),
        None if usable => HostIps::Ready(entry.ips.clone()),
        None => HostIps::Pending,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::platform::DnsDone;

    /// Резолвер, который отвечает сразу: IP-литерал — им же, остальное — ошибка.
    pub(crate) struct InstantDns;
    impl DnsResolver for InstantDns {
        fn resolve_in_background(&self, host: String, done: DnsDone) {
            done(
                host.parse::<IpAddr>()
                    .map(|ip| vec![ip])
                    .map_err(|_| std::io::Error::other("not found")),
            );
        }
    }

    /// Не отвечает никогда.
    struct SilentDns;
    impl DnsResolver for SilentDns {
        fn resolve_in_background(&self, _: String, _: DnsDone) {}
    }

    #[test]
    fn ready_failed_pending() {
        let r = Resolver::new(Arc::new(InstantDns));
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        assert!(matches!(r.lookup("127.0.0.1"), HostIps::Ready(v) if v == vec![ip]));
        assert!(matches!(r.lookup("nope.invalid"), HostIps::Failed(e, 1) if e == "not found"));
        let r = Resolver::new(Arc::new(SilentDns));
        assert!(matches!(r.lookup("x"), HostIps::Pending));
    }

    #[test]
    fn empty_answer_and_v4_first() {
        let mut e = Resolved::default();
        store(&mut e, Ok(vec![]));
        assert_eq!(e.err.as_deref(), Some("адрес не найден"));
        let v6: IpAddr = "::1".parse().unwrap();
        let v4: IpAddr = "10.0.0.1".parse().unwrap();
        store(&mut e, Ok(vec![v6, v4, v4]));
        assert_eq!(e.ips, vec![v4, v6]);
        assert!(matches!(current(&e), HostIps::Ready(_)));
    }
}
