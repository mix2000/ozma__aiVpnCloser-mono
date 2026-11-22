//! Проверка `route`: трафик к каждому хосту агентов уходит в туннель.

use std::collections::HashMap;
use std::sync::Arc;

use super::resolver::{HostIps, Resolver};
use super::rules::route_verdict;
use crate::models::NetInterface;
use crate::platform::{DnsResolver, NetProbe};

/// После стольких неудачных резолвов подряд хост считается «упал».
pub const DNS_FAILS_TO_DOWN: u32 = 3;

/// Что вышло по хостам за один опрос.
#[derive(Debug, Default)]
struct Tally {
    good: Vec<String>,
    bad: Vec<String>,
    unknown: Vec<String>,
}

impl Tally {
    /// Любой плохой — нет; иначе любой неизвестный — неизвестно; иначе ок.
    fn outcome(self) -> (Option<bool>, String) {
        if !self.bad.is_empty() {
            (Some(false), self.bad.join("; "))
        } else if !self.unknown.is_empty() {
            (None, self.unknown.join("; "))
        } else {
            (Some(true), self.good.join("; "))
        }
    }
}

pub struct RouteChecker {
    pub(crate) resolver: Resolver,
    /// host → (подряд неудачных резолвов, номер последней учтённой попытки).
    dns_fails: HashMap<String, (u32, u64)>,
}

impl RouteChecker {
    pub fn new(dns: Arc<dyn DnsResolver>) -> Self {
        RouteChecker {
            resolver: Resolver::new(dns),
            dns_fails: HashMap::new(),
        }
    }

    pub fn check(
        &mut self,
        net: &dyn NetProbe,
        hosts: &[String],
        ifaces: &[NetInterface],
        tunnels: &[&NetInterface],
    ) -> (Option<bool>, String) {
        if hosts.is_empty() {
            return (Some(false), "список хостов пуст".into());
        }
        let mut t = Tally::default();
        for host in hosts {
            match self.resolver.lookup(host) {
                HostIps::Pending => t.unknown.push(format!("{host}: узнаю адрес")),
                HostIps::Failed(err, generation) => {
                    let line = format!("{host}: адрес не узнать ({err})");
                    if self.dns_failed(host, generation) >= DNS_FAILS_TO_DOWN {
                        t.bad.push(line);
                    } else {
                        t.unknown.push(line);
                    }
                }
                HostIps::Ready(ips) => {
                    self.dns_fails.remove(host);
                    let ip = ips[0];
                    match net.local_addr_for(ip) {
                        Ok(local) => {
                            let (ok, owner) = route_verdict(local, ifaces, tunnels);
                            let via = owner
                                .map(|o| format!("{} ({local})", o.name))
                                .unwrap_or_else(|| local.to_string());
                            if ok {
                                t.good.push(format!("{host} → {via}"));
                            } else {
                                t.bad.push(format!("{host} → {via}, не через VPN"));
                            }
                        }
                        Err(e) => t.bad.push(format!("{host}: нет маршрута ({e})")),
                    }
                }
            }
        }
        t.outcome()
    }

    /// Засчитать неудачную попытку (каждую — один раз) и вернуть число подряд.
    fn dns_failed(&mut self, host: &str, generation: u64) -> u32 {
        let e = self.dns_fails.entry(host.to_string()).or_insert((0, 0));
        if e.1 != generation {
            e.0 += 1;
            e.1 = generation;
        }
        e.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SystemVpn;
    use crate::services::checks::resolver::tests::InstantDns;
    use crate::services::checks::rules::tests::machine;
    use std::net::IpAddr;

    /// Сеть, где ОС выбирает заданный исходный адрес для любого хоста.
    struct Via(&'static str);
    impl NetProbe for Via {
        fn interfaces(&self) -> Vec<NetInterface> {
            machine()
        }
        fn system_vpns(&self) -> Vec<SystemVpn> {
            Vec::new()
        }
        fn local_addr_for(&self, _: IpAddr) -> std::io::Result<IpAddr> {
            Ok(self.0.parse().unwrap())
        }
    }

    fn wait(
        rc: &mut RouteChecker,
        net: &dyn NetProbe,
        hosts: &[String],
        tunnels: &[&NetInterface],
    ) -> (Option<bool>, String) {
        let ifaces = machine();
        let mut res = (None, String::new());
        for _ in 0..100 {
            res = rc.check(net, hosts, &ifaces, tunnels);
            if res.0.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        res
    }

    #[test]
    fn via_tunnel_or_physical() {
        let hosts = vec!["10.1.1.1".to_string(), "10.1.1.2".to_string()];
        let mut rc = RouteChecker::new(Arc::new(InstantDns));
        let res = wait(&mut rc, &Via("172.19.0.1"), &hosts, &[]);
        assert_eq!(res.0, Some(true), "{}", res.1);
        assert_eq!(
            res.1,
            "10.1.1.1 → utun233 (172.19.0.1); 10.1.1.2 → utun233 (172.19.0.1)"
        );
        let res = wait(&mut rc, &Via("192.168.1.20"), &hosts, &[]);
        assert_eq!(res.0, Some(false));
        assert!(res
            .1
            .starts_with("10.1.1.1 → en1 (192.168.1.20), не через VPN"));
        assert_eq!(
            rc.check(&Via("1.1.1.1"), &[], &machine(), &[]),
            (Some(false), "список хостов пуст".into())
        );
    }

    #[test]
    fn dns_failure_is_unknown_then_down() {
        let mut rc = RouteChecker::new(Arc::new(InstantDns));
        let hosts = vec!["nonexistent.invalid".to_string()];
        let ifaces = machine();
        // Подменяем результат резолва: три неудачные попытки подряд.
        let mut seen = Vec::new();
        for generation in 1..=3u64 {
            rc.resolver.set_failed(&hosts[0], "not found", generation);
            // Несколько опросов на одну попытку засчитываются один раз.
            seen.push(rc.check(&Via("1.1.1.1"), &hosts, &ifaces, &[]).0);
            seen.push(rc.check(&Via("1.1.1.1"), &hosts, &ifaces, &[]).0);
        }
        assert_eq!(seen, vec![None, None, None, None, Some(false), Some(false)]);
    }

    #[test]
    fn tally_priority() {
        let t = Tally {
            good: vec!["a".into()],
            bad: vec![],
            unknown: vec!["b".into()],
        };
        assert_eq!(t.outcome(), (None, "b".into()));
        let t = Tally {
            good: vec!["a".into()],
            bad: vec!["c".into()],
            unknown: vec!["b".into()],
        };
        assert_eq!(t.outcome(), (Some(false), "c".into()));
    }
}
