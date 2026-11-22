//! Настоящие проверки: берут данные из [`NetProbe`] и снимка процессов.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::route::RouteChecker;
use super::rules::{check_interface, check_process, check_system_vpn, find_interface};
use super::CheckSource;
use crate::models::{CheckResult, Config, NetInterface, ProcInfo, SystemVpn, WatchItem};
use crate::platform::{DnsResolver, NetProbe};
use crate::services::discovery;
use crate::services::owners::OwnerResolver;

/// Сколько держать список системных VPN (scutil/rasdial — внешние команды).
const SYSTEM_VPN_TTL: Duration = Duration::from_secs(2);

pub struct SystemChecks {
    net: Arc<dyn NetProbe>,
    route: RouteChecker,
    owners: OwnerResolver,
    vpn_cache: Option<(Instant, Vec<SystemVpn>)>,
}

/// Что понадобится для пунктов этого конфига.
struct Needs {
    ifaces: bool,
    owners: bool,
    vpns: bool,
}

impl Needs {
    fn of(cfg: &Config) -> Needs {
        let has = |f: fn(&WatchItem) -> bool| cfg.watch.iter().any(f);
        let owners = has(|w| matches!(w, WatchItem::Interface { .. }));
        Needs {
            ifaces: owners || has(|w| matches!(w, WatchItem::Route)),
            owners,
            vpns: owners || has(|w| matches!(w, WatchItem::SystemVpn { .. })),
        }
    }
}

impl SystemChecks {
    /// Проверки поверх сети, источников владельцев туннелей и резолвера.
    pub fn new(net: Arc<dyn NetProbe>, owners: OwnerResolver, dns: Arc<dyn DnsResolver>) -> Self {
        SystemChecks {
            net,
            route: RouteChecker::new(dns),
            owners,
            vpn_cache: None,
        }
    }

    fn system_vpns(&mut self) -> Vec<SystemVpn> {
        if let Some((at, list)) = &self.vpn_cache {
            if at.elapsed() < SYSTEM_VPN_TTL {
                return list.clone();
            }
        }
        let list = self.net.system_vpns();
        self.vpn_cache = Some((Instant::now(), list.clone()));
        list
    }

    /// Проверка `route` по списку хостов.
    pub fn check_route(
        &mut self,
        hosts: &[String],
        ifaces: &[NetInterface],
        tunnels: &[&NetInterface],
    ) -> (Option<bool>, String) {
        self.route.check(self.net.as_ref(), hosts, ifaces, tunnels)
    }

    /// Интерфейсы (с владельцами туннелей) и системные VPN — только если нужны.
    fn gather(&mut self, needs: &Needs, procs: &[ProcInfo]) -> (Vec<NetInterface>, Vec<SystemVpn>) {
        let mut ifaces = if needs.ifaces {
            discovery::interfaces(self.net.as_ref())
        } else {
            Vec::new()
        };
        let vpns = if needs.vpns {
            self.system_vpns()
        } else {
            Vec::new()
        };
        if needs.owners {
            self.owners.resolve(&mut ifaces, procs, &vpns);
        }
        (ifaces, vpns)
    }

    /// Проверка одного пункта.
    fn check_item(&mut self, item: &WatchItem, cfg: &Config, ctx: &Ctx) -> CheckResult {
        let (ok, detail, owner) = match item {
            WatchItem::Interface { name, addr } => {
                let (ok, d) = check_interface(name, addr.as_deref(), ctx.ifaces);
                let owner =
                    find_interface(name, addr.as_deref(), ctx.ifaces).and_then(|i| i.owner.clone());
                (Some(ok), d, owner)
            }
            WatchItem::Process { name } => {
                let (ok, d) = check_process(name, ctx.procs);
                (Some(ok), d, None)
            }
            WatchItem::SystemVpn { id, name } => {
                let (ok, d) = check_system_vpn(id, name, ctx.vpns);
                (Some(ok), d, None)
            }
            WatchItem::Route => {
                let (ok, d) = self.check_route(&cfg.hosts, ctx.ifaces, &ctx.tunnels);
                (ok, d, None)
            }
        };
        CheckResult::new(item.clone(), ok, detail).with_owner(owner)
    }
}

/// Данные одного опроса.
struct Ctx<'a> {
    ifaces: &'a [NetInterface],
    vpns: &'a [SystemVpn],
    procs: &'a [ProcInfo],
    /// Выбранные туннели: маршрут должен идти через один из них.
    tunnels: Vec<&'a NetInterface>,
}

impl CheckSource for SystemChecks {
    fn check(&mut self, cfg: &Config, procs: &[ProcInfo]) -> Vec<CheckResult> {
        let (ifaces, vpns) = self.gather(&Needs::of(cfg), procs);
        let tunnels = cfg
            .watch
            .iter()
            .filter_map(|w| match w {
                WatchItem::Interface { name, addr } => {
                    find_interface(name, addr.as_deref(), &ifaces)
                }
                _ => None,
            })
            .collect();
        let ctx = Ctx {
            ifaces: &ifaces,
            vpns: &vpns,
            procs,
            tunnels,
        };
        cfg.watch
            .iter()
            .map(|item| self.check_item(item, cfg, &ctx))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::OwnerProbe;
    use crate::services::checks::resolver::tests::InstantDns;
    use crate::services::checks::rules::tests::machine;
    use std::net::IpAddr;

    struct FakeNet;
    impl NetProbe for FakeNet {
        fn interfaces(&self) -> Vec<NetInterface> {
            machine()
        }
        fn system_vpns(&self) -> Vec<SystemVpn> {
            vec![SystemVpn {
                id: "H".into(),
                name: "Happ Plus".into(),
                connected: true,
            }]
        }
        fn local_addr_for(&self, _: IpAddr) -> std::io::Result<IpAddr> {
            Ok("172.19.0.1".parse().unwrap())
        }
    }

    /// Владельцы: системный VPN «Happ Plus» держит utun4.
    struct HappOnUtun4;
    impl OwnerProbe for HappOnUtun4 {
        fn utun_units(&self, _: &[u32]) -> Vec<(u32, u32)> {
            Vec::new()
        }
        fn read_config(&self, _: &std::path::Path) -> Option<String> {
            None
        }
        fn service_interface(&self, id: &str) -> Option<String> {
            (id == "H").then(|| "utun4".to_string())
        }
    }

    #[test]
    fn all_kinds_on_fake_machine() {
        let owners = OwnerResolver::new(Arc::new(HappOnUtun4), 1);
        let mut sc = SystemChecks::new(Arc::new(FakeNet), owners, Arc::new(InstantDns));
        let cfg = Config {
            watch: vec![
                WatchItem::Interface {
                    name: "utun4".into(),
                    addr: None,
                },
                WatchItem::Process {
                    name: "xray".into(),
                },
                WatchItem::SystemVpn {
                    id: String::new(),
                    name: "Happ Plus".into(),
                },
                WatchItem::Route,
            ],
            hosts: vec!["10.0.0.1".into()],
            ..Config::default()
        };
        let procs = vec![ProcInfo {
            pid: 9,
            name: "xray".into(),
            ..Default::default()
        }];
        let mut res = Vec::new();
        for _ in 0..100 {
            res = sc.check(&cfg, &procs);
            if res[3].ok.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let oks: Vec<Option<bool>> = res.iter().map(|c| c.ok).collect();
        // Маршрут идёт через utun233, а выбран только utun4 — не через VPN.
        assert_eq!(oks, [Some(true), Some(true), Some(true), Some(false)]);
        assert!(res[3].detail.contains("не через VPN"), "{}", res[3].detail);
        let owner = res[0].owner.as_ref().expect("владелец utun4");
        assert_eq!(owner.name, "Happ Plus");
        assert!(res[1].owner.is_none() && res[3].owner.is_none());
    }
}
