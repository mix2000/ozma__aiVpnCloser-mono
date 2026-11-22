//! Программа-владелец туннельного интерфейса: «utun233 (sing-box)».
//!
//! - `clients` — словарь известных VPN-клиентов;
//! - `configs` — путь к конфигу по argv и поля туннеля из него;
//! - `rules` — чистое сопоставление по свидетельствам.
//!
//! [`OwnerResolver`] собирает свидетельства через `platform::OwnerProbe`
//! (lsof, чтение конфигов, scutil) и кеширует дорогие на [`EVIDENCE_TTL`].

mod clients;
mod configs;
mod rules;

use std::sync::Arc;
use std::time::{Duration, Instant};

use clients::{client_of, Client};
use configs::{config_path, tun_spec};
use rules::{assign, Evidence, Running};

use crate::models::{NetInterface, ProcInfo, SelfCtx, SystemVpn};
use crate::platform::OwnerProbe;

/// Как долго верить lsof, конфигам и scutil (не дёргать их на каждом опросе).
pub const EVIDENCE_TTL: Duration = Duration::from_secs(20);

/// Дорогая часть свидетельств и для каких процессов она собрана.
struct Cached {
    at: Instant,
    key: Vec<(u32, u64)>,
    services_key: Vec<String>,
    evidence: Evidence,
}

pub struct OwnerResolver {
    probe: Arc<dyn OwnerProbe>,
    /// Свой pid: lsof без sudo видит только процессы своего пользователя.
    self_pid: u32,
    ttl: Duration,
    cache: Option<Cached>,
}

impl OwnerResolver {
    pub fn new(probe: Arc<dyn OwnerProbe>, self_pid: u32) -> Self {
        Self::with_ttl(probe, self_pid, EVIDENCE_TTL)
    }

    pub fn with_ttl(probe: Arc<dyn OwnerProbe>, self_pid: u32, ttl: Duration) -> Self {
        OwnerResolver {
            probe,
            self_pid,
            ttl,
            cache: None,
        }
    }

    /// Проставить владельцев туннелям `ifaces`.
    pub fn resolve(&mut self, ifaces: &mut [NetInterface], procs: &[ProcInfo], vpns: &[SystemVpn]) {
        if !ifaces.iter().any(|i| i.tunnel) {
            return;
        }
        let running: Vec<(Running, &ProcInfo, Client)> = procs
            .iter()
            .filter_map(|p| {
                let c = client_of(p)?;
                let r = Running {
                    label: c.label.to_string(),
                    pid: p.pid,
                    role: c.role,
                };
                Some((r, p, c))
            })
            .collect();
        let key: Vec<(u32, u64)> = running
            .iter()
            .map(|(_, p, _)| (p.pid, p.start_time))
            .collect();
        let services_key: Vec<String> = vpns
            .iter()
            .filter(|v| v.connected)
            .map(|v| v.id.clone())
            .collect();
        let fresh = self.cache.as_ref().is_some_and(|c| {
            c.at.elapsed() < self.ttl && c.key == key && c.services_key == services_key
        });
        if !fresh {
            let me = SelfCtx::for_pid(self.self_pid, procs);
            let evidence = self.collect(&running, vpns, &me);
            self.cache = Some(Cached {
                at: Instant::now(),
                key,
                services_key,
                evidence,
            });
        }
        let mut evidence = self
            .cache
            .as_ref()
            .map(|c| c.evidence.clone())
            .unwrap_or_default();
        evidence.running = running.into_iter().map(|(r, _, _)| r).collect();
        assign(ifaces, &evidence);
    }

    /// Дорогие свидетельства: utun_control своих процессов, конфиги, scutil.
    fn collect(
        &self,
        running: &[(Running, &ProcInfo, Client)],
        vpns: &[SystemVpn],
        me: &SelfCtx,
    ) -> Evidence {
        let mine: Vec<u32> = running
            .iter()
            .filter(|(_, p, _)| me.uid.is_some() && p.uid == me.uid)
            .map(|(_, p, _)| p.pid)
            .collect();
        let utun = self
            .probe
            .utun_units(&mine)
            .into_iter()
            .filter_map(|(pid, n)| {
                let (r, _, _) = running.iter().find(|(r, _, _)| r.pid == pid)?;
                Some((r.label.clone(), pid, n))
            })
            .collect();
        let configs = running
            .iter()
            .filter_map(|(r, p, c)| {
                let kind = c.config?;
                let argv = if p.cmd.is_empty() {
                    self.probe.process_args(p.pid)?
                } else {
                    p.cmd.clone()
                };
                let path = config_path(kind, &argv)?;
                let text = self.probe.read_config(&path)?;
                Some((r.label.clone(), r.pid, tun_spec(kind, &text)?))
            })
            .collect();
        let services = vpns
            .iter()
            .filter(|v| v.connected)
            .filter_map(|v| Some((v.name.clone(), self.probe.service_interface(&v.id)?)))
            .collect();
        Evidence {
            utun,
            configs,
            services,
            running: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IfAddr, IfOwner};
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Подставные источники: считает обращения к дорогим.
    #[derive(Default)]
    struct Probe {
        calls: Arc<AtomicUsize>,
    }
    impl OwnerProbe for Probe {
        fn utun_units(&self, pids: &[u32]) -> Vec<(u32, u32)> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            pids.iter()
                .filter(|p| **p == 500)
                .map(|p| (*p, 7))
                .collect()
        }
        fn read_config(&self, path: &Path) -> Option<String> {
            (path == sing_box_config()).then(|| {
                r#"{"inbounds":[{"type":"tun","interface_name":"utun233","address":["172.19.0.1/30"]}]}"#
                    .to_string()
            })
        }
        fn service_interface(&self, id: &str) -> Option<String> {
            (id == "H").then(|| "utun5".to_string())
        }
    }

    /// Абсолютный путь на любой ОС (на Windows `/etc/…` не абсолютный).
    fn sing_box_config() -> std::path::PathBuf {
        std::env::temp_dir()
            .join("vpn-closer-cfg")
            .join("sing-box")
            .join("config.json")
    }

    fn tun(name: &str, ip: &str) -> NetInterface {
        let mut i = NetInterface::new(
            name,
            true,
            true,
            vec![IfAddr {
                ip: ip.parse().unwrap(),
                prefix: 30,
            }],
        );
        i.ptp = true;
        i
    }

    fn proc_(pid: u32, uid: &str, name: &str, cmd: &str) -> ProcInfo {
        ProcInfo {
            pid,
            uid: Some(uid.into()),
            name: name.into(),
            cmd: cmd.split_whitespace().map(str::to_string).collect(),
            start_time: pid as u64,
            ..Default::default()
        }
    }

    #[test]
    fn all_sources_and_cache() {
        let calls = Arc::new(AtomicUsize::new(0));
        let me = 4000;
        let mut r = OwnerResolver::new(
            Arc::new(Probe {
                calls: calls.clone(),
            }),
            me,
        );
        let procs = vec![
            proc_(me, "501", "ai-vpn-closer", "ai-vpn-closer"),
            proc_(
                75686,
                "0",
                "sing-box",
                &format!("sing-box run -c {}", sing_box_config().display()),
            ),
            proc_(500, "501", "wireguard-go", "wireguard-go utun"),
            proc_(92233, "0", "openvpn", "openvpn --config client.ovpn"),
        ];
        let vpns = vec![SystemVpn {
            id: "H".into(),
            name: "Happ Plus".into(),
            connected: true,
        }];
        let mut ifaces = vec![
            tun("utun233", "172.19.0.1"),
            tun("utun7", "10.7.0.2"),
            tun("utun5", "10.5.0.2"),
            tun("utun4", "10.8.0.6"),
        ];
        r.resolve(&mut ifaces, &procs, &vpns);
        let owners: Vec<Option<IfOwner>> = ifaces.iter().map(|i| i.owner.clone()).collect();
        assert_eq!(
            owners,
            vec![
                Some(IfOwner::exact("sing-box", Some(75686))),
                Some(IfOwner::exact("WireGuard", Some(500))),
                Some(IfOwner::exact("Happ Plus", None)),
                Some(IfOwner::likely("OpenVPN", Some(92233))),
            ]
        );
        // Повторно в пределах TTL — lsof не дёргаем.
        let mut again: Vec<NetInterface> = ifaces
            .iter()
            .map(|i| NetInterface {
                owner: None,
                ..i.clone()
            })
            .collect();
        r.resolve(&mut again, &procs, &vpns);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(again[0].owner, ifaces[0].owner);
        // Сменился набор процессов — собираем заново.
        r.resolve(&mut again, &procs[..3], &vpns);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn no_tunnels_no_work() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut r = OwnerResolver::with_ttl(
            Arc::new(Probe {
                calls: calls.clone(),
            }),
            1,
            Duration::ZERO,
        );
        let mut ifaces = vec![NetInterface::new("en0", true, false, vec![])];
        r.resolve(&mut ifaces, &[], &[]);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(ifaces[0].owner.is_none());
    }
}
