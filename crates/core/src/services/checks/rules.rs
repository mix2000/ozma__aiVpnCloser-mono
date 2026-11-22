//! Чистые проверки пунктов по готовым данным (интерфейсы, процессы, системные VPN).

use std::net::IpAddr;

use crate::models::{CheckResult, Mode, NetInterface, ProcInfo, SystemVpn, Verdict};
use crate::services::config_validation::parse_subnet;
use crate::services::net::same_net;
use crate::services::vpn_processes::process_matches;

/// Итог по всем пунктам с учётом правила all/any.
pub fn combine(mode: Mode, checks: &[CheckResult]) -> Verdict {
    if checks.is_empty() {
        return Verdict::Unknown;
    }
    let any_true = checks.iter().any(|c| c.ok == Some(true));
    let any_false = checks.iter().any(|c| c.ok == Some(false));
    let any_none = checks.iter().any(|c| c.ok.is_none());
    match mode {
        Mode::All if any_false => Verdict::Down,
        Mode::All if any_none => Verdict::Unknown,
        Mode::All => Verdict::Up,
        Mode::Any if any_true => Verdict::Up,
        Mode::Any if any_none => Verdict::Unknown,
        Mode::Any => Verdict::Down,
    }
}

/// Найти интерфейс для пункта `interface`: по адресу/подсети (главное), иначе по имени.
pub fn find_interface<'a>(
    name: &str,
    addr: Option<&str>,
    ifaces: &'a [NetInterface],
) -> Option<&'a NetInterface> {
    if let Some(a) = addr {
        let (ip, prefix) = parse_subnet(a)?;
        return ifaces.iter().find(|i| {
            i.nets.iter().any(|n| match prefix {
                Some(p) => same_net(n.ip, ip, p),
                None => n.ip == ip || same_net(n.ip, ip, n.prefix),
            })
        });
    }
    ifaces.iter().find(|i| i.name == name)
}

pub fn check_interface(name: &str, addr: Option<&str>, ifaces: &[NetInterface]) -> (bool, String) {
    match find_interface(name, addr, ifaces) {
        Some(i) if i.up && !i.nets.is_empty() => {
            (true, format!("{} включён ({})", i.name, i.addrs.join(", ")))
        }
        Some(i) if !i.up => (false, format!("{} выключен", i.name)),
        Some(i) => (false, format!("{} без адреса", i.name)),
        None => {
            let other = ifaces.iter().find(|i| !name.is_empty() && i.name == name);
            match (addr, other) {
                (Some(a), Some(o)) => (
                    false,
                    format!(
                        "адреса {a} нет; у {} сейчас {}",
                        o.name,
                        if o.addrs.is_empty() {
                            "нет адресов".to_string()
                        } else {
                            o.addrs.join(", ")
                        }
                    ),
                ),
                (Some(a), None) => (false, format!("интерфейса с адресом {a} нет")),
                (None, _) => (false, format!("интерфейса {name} нет")),
            }
        }
    }
}

pub fn check_process(name: &str, procs: &[ProcInfo]) -> (bool, String) {
    let found: Vec<u32> = procs
        .iter()
        .filter(|p| process_matches(p, name))
        .map(|p| p.pid)
        .collect();
    if found.is_empty() {
        (false, format!("{name} не запущен"))
    } else {
        (true, format!("{name} запущен (pid {})", found[0]))
    }
}

pub fn check_system_vpn(id: &str, name: &str, vpns: &[SystemVpn]) -> (bool, String) {
    let v = vpns.iter().find(|v| {
        (!id.is_empty() && v.id.eq_ignore_ascii_case(id))
            || (!name.is_empty() && v.name.eq_ignore_ascii_case(name))
    });
    let label = if name.is_empty() { id } else { name };
    match v {
        Some(v) if v.connected => (true, format!("«{}» подключён", v.name)),
        Some(v) => (false, format!("«{}» не подключён", v.name)),
        None => (false, format!("«{label}» не найден")),
    }
}

/// Интерфейс, которому принадлежит адрес, и считается ли он туннелем.
/// `tunnels` — допустимые интерфейсы (выбранные пункты `interface`); пусто —
/// подходит любой туннельный.
pub fn route_verdict<'a>(
    local: IpAddr,
    ifaces: &'a [NetInterface],
    tunnels: &[&NetInterface],
) -> (bool, Option<&'a NetInterface>) {
    let owner = ifaces.iter().find(|i| i.has_ip(local));
    let ok = match owner {
        None => false,
        Some(o) if tunnels.is_empty() => o.tunnel,
        Some(o) => tunnels.iter().any(|t| t.name == o.name),
    };
    (ok, owner)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::models::{IfAddr, WatchItem};

    pub(crate) fn iface(name: &str, ip: &str, prefix: u8, tunnel: bool) -> NetInterface {
        NetInterface::new(
            name,
            true,
            tunnel,
            vec![IfAddr {
                ip: ip.parse().unwrap(),
                prefix,
            }],
        )
    }

    /// Машина из ТЗ: sing-box TUN utun233 и корпоративный utun4.
    pub(crate) fn machine() -> Vec<NetInterface> {
        vec![
            iface("lo0", "127.0.0.1", 8, false),
            iface("en1", "192.168.1.20", 24, false),
            iface("utun233", "172.19.0.1", 30, true),
            iface("utun4", "10.8.0.6", 32, true),
        ]
    }

    fn cr(ok: Option<bool>) -> CheckResult {
        CheckResult::new(WatchItem::Route, ok, String::new())
    }

    #[test]
    fn combine_rules() {
        use Verdict::*;
        let t = cr(Some(true));
        let f = cr(Some(false));
        let n = cr(None);
        assert_eq!(combine(Mode::All, &[t.clone(), t.clone()]), Up);
        assert_eq!(combine(Mode::All, &[t.clone(), f.clone()]), Down);
        assert_eq!(combine(Mode::All, &[t.clone(), n.clone()]), Unknown);
        assert_eq!(combine(Mode::All, &[n.clone(), f.clone()]), Down);
        assert_eq!(combine(Mode::Any, &[f.clone(), t.clone()]), Up);
        assert_eq!(combine(Mode::Any, &[f.clone(), n.clone()]), Unknown);
        assert_eq!(combine(Mode::Any, &[f.clone(), f.clone()]), Down);
        assert_eq!(combine(Mode::All, &[]), Unknown);
    }

    #[test]
    fn interface_by_address_beats_name() {
        let mut m = machine();
        // После перезапуска VPN имя сменилось, адрес тот же.
        m[2].name = "utun7".into();
        let (ok, d) = check_interface("utun233", Some("172.19.0.1"), &m);
        assert!(ok, "{d}");
        assert!(d.contains("utun7"));
        // Адрес другой из той же /30 — тоже он.
        let (ok, _) = check_interface("utun233", Some("172.19.0.2"), &m);
        assert!(ok);
        // Адреса нет, а интерфейс с таким именем есть с другим адресом — нет.
        let (ok, d) = check_interface("utun4", Some("10.9.9.9"), &m);
        assert!(!ok);
        assert!(d.contains("utun4"), "{d}");
        // Без адреса — по имени.
        assert!(check_interface("utun4", None, &m).0);
        assert!(!check_interface("utun99", None, &m).0);
        // Выключенный.
        m[3].up = false;
        assert!(!check_interface("utun4", None, &m).0);
    }

    #[test]
    fn route_verdicts() {
        let m = machine();
        let tun: IpAddr = "172.19.0.1".parse().unwrap();
        let phys: IpAddr = "192.168.1.20".parse().unwrap();
        assert!(route_verdict(tun, &m, &[]).0);
        assert!(!route_verdict(phys, &m, &[]).0);
        // Выбран конкретный туннель utun4 — utun233 не подходит.
        assert!(!route_verdict(tun, &m, &[&m[3]]).0);
        assert!(route_verdict(tun, &m, &[&m[2]]).0);
        assert!(!route_verdict("8.8.8.8".parse().unwrap(), &m, &[]).0);
    }

    #[test]
    fn process_and_system_vpn() {
        let procs = vec![ProcInfo {
            pid: 65030,
            name: "xray".into(),
            ..Default::default()
        }];
        assert!(check_process("xray", &procs).0);
        assert!(!check_process("sing-box", &procs).0);
        let vpns = vec![SystemVpn {
            id: "ID1".into(),
            name: "Happ Plus".into(),
            connected: false,
        }];
        assert!(!check_system_vpn("ID1", "Happ Plus", &vpns).0);
        let mut v2 = vpns.clone();
        v2[0].connected = true;
        assert!(check_system_vpn("", "happ plus", &v2).0);
        assert!(check_system_vpn("id1", "", &v2).0);
        assert!(!check_system_vpn("X", "Y", &v2).0);
    }
}
