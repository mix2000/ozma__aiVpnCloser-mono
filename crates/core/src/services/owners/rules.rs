//! Чистое сопоставление «интерфейс → программа» по собранным свидетельствам,
//! по убыванию надёжности: utun_control → конфиг → системный VPN → описание
//! адаптера → эвристика «единственный подходящий клиент».

use std::net::IpAddr;

use super::clients::Role;
use super::configs::TunSpec;
use crate::models::{IfOwner, NetInterface, OwnerConfidence};

/// Запущенный известный клиент.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    pub label: String,
    pub pid: u32,
    pub role: Role,
}

/// Всё, что удалось узнать о владельцах.
#[derive(Debug, Clone, Default)]
pub struct Evidence {
    /// Сокет utun_control: (подпись, pid, номер utun).
    pub utun: Vec<(String, u32, u32)>,
    /// Туннель из конфига клиента: (подпись, pid, что в конфиге).
    pub configs: Vec<(String, u32, TunSpec)>,
    /// Подключённый системный VPN: (имя сервиса, интерфейс).
    pub services: Vec<(String, String)>,
    /// Запущенные известные клиенты.
    pub running: Vec<Running>,
}

/// Владелец по описанию адаптера Windows (или имени интерфейса).
pub fn adapter_owner(name: &str, desc: Option<&str>) -> Option<IfOwner> {
    let text = format!("{} {}", name, desc.unwrap_or("")).to_lowercase();
    const EXACT: &[(&str, &str)] = &[
        ("wireguard", "WireGuard"),
        ("amneziawg", "AmneziaVPN"),
        ("amnezia", "AmneziaVPN"),
        ("sing-box", "sing-box"),
        ("happ", "Happ"),
        ("outline", "Outline"),
        ("tailscale", "Tailscale"),
        ("zerotier", "ZeroTier"),
        ("openvpn data channel", "OpenVPN"),
        ("hiddify", "Hiddify"),
        ("mullvad", "Mullvad"),
    ];
    const LIKELY: &[(&str, &str)] = &[("tap-windows", "OpenVPN"), ("tap-win", "OpenVPN")];
    if let Some((_, l)) = EXACT.iter().find(|(k, _)| text.contains(k)) {
        return Some(IfOwner::exact(*l, None));
    }
    LIKELY
        .iter()
        .find(|(k, _)| text.contains(k))
        .map(|(_, l)| IfOwner::likely(*l, None))
}

fn ips(i: &NetInterface) -> Vec<IpAddr> {
    i.nets.iter().map(|n| n.ip).collect()
}

/// Точный владелец интерфейса по свидетельствам (правила 1–4).
fn exact_owner(i: &NetInterface, ev: &Evidence) -> Option<IfOwner> {
    if let Some((label, pid, _)) = ev
        .utun
        .iter()
        .find(|(_, _, n)| i.name == format!("utun{n}"))
    {
        return Some(IfOwner::exact(label.clone(), Some(*pid)));
    }
    let addrs = ips(i);
    if let Some((label, pid, _)) = ev
        .configs
        .iter()
        .find(|(_, _, spec)| spec.matches(&i.name, &addrs))
    {
        return Some(IfOwner::exact(label.clone(), Some(*pid)));
    }
    if let Some((svc, _)) = ev.services.iter().find(|(_, ifn)| *ifn == i.name) {
        return Some(IfOwner::exact(svc.clone(), None));
    }
    adapter_owner(&i.name, i.desc.as_deref()).filter(|_| i.tunnel)
}

/// Проставить владельцев туннельным интерфейсам.
pub fn assign(ifaces: &mut [NetInterface], ev: &Evidence) {
    for i in ifaces.iter_mut().filter(|i| i.tunnel && i.owner.is_none()) {
        i.owner = exact_owner(i, ev);
    }
    guess_owner(ifaces, ev);
}

/// Эвристика: единственный туннель без владельца (точка-точка, включён, с
/// IPv4) и единственный подходящий клиент, у которого туннель ещё не найден.
/// Клиенты, которые сами держат туннель, важнее помощников; прокси и
/// системные VPN не в счёт (у последних есть своё точное правило).
fn guess_owner(ifaces: &mut [NetInterface], ev: &Evidence) {
    let candidates: Vec<usize> = ifaces
        .iter()
        .enumerate()
        .filter(|(_, i)| {
            i.tunnel && i.up && i.ptp && i.owner.is_none() && i.nets.iter().any(|n| n.ip.is_ipv4())
        })
        .map(|(n, _)| n)
        .collect();
    let [only] = candidates[..] else {
        return;
    };
    let owned: Vec<&str> = ifaces
        .iter()
        .filter_map(|i| i.owner.as_ref())
        .filter(|o| o.confidence == OwnerConfidence::Exact)
        .map(|o| o.name.as_str())
        .collect();
    let pick = |role: Role| -> Vec<&Running> {
        let mut out: Vec<&Running> = Vec::new();
        for r in ev.running.iter().filter(|r| r.role == role) {
            if !owned.contains(&r.label.as_str()) && !out.iter().any(|o| o.label == r.label) {
                out.push(r);
            }
        }
        out
    };
    let tunnels = pick(Role::Tunnel);
    let chosen = match (tunnels.as_slice(), pick(Role::Helper).as_slice()) {
        ([one], _) => Some((*one).clone()),
        ([], [one]) => Some((*one).clone()),
        _ => None,
    };
    if let Some(r) = chosen {
        ifaces[only].owner = Some(IfOwner::likely(r.label, Some(r.pid)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IfAddr;

    fn tun(name: &str, ip: Option<&str>) -> NetInterface {
        let nets = ip
            .map(|ip| {
                vec![IfAddr {
                    ip: ip.parse().unwrap(),
                    prefix: 30,
                }]
            })
            .unwrap_or_default();
        let mut i = NetInterface::new(name, true, true, nets);
        i.ptp = true;
        i
    }

    fn run(label: &str, pid: u32, role: Role) -> Running {
        Running {
            label: label.into(),
            pid,
            role,
        }
    }

    /// Машина из задачи: sing-box (root, конфиг читается), корпоративный
    /// openvpn (root), помощники OpenVPN Connect, xray-прокси, виджет Happ.
    fn dev_machine() -> (Vec<NetInterface>, Evidence) {
        let ifaces = vec![
            tun("utun0", None),
            tun("utun233", Some("172.19.0.1")),
            tun("utun4", Some("10.8.0.6")),
        ];
        let ev = Evidence {
            configs: vec![(
                "sing-box".into(),
                75686,
                TunSpec {
                    name: Some("utun233".into()),
                    nets: vec![("172.19.0.1".parse().unwrap(), 30)],
                },
            )],
            running: vec![
                run("OpenVPN Connect", 341, Role::Helper),
                run("OpenVPN Connect", 343, Role::Helper),
                run("Happ", 33029, Role::SystemVpn),
                run("xray", 65030, Role::Proxy),
                run("sing-box", 75686, Role::Tunnel),
                run("OpenVPN", 92233, Role::Tunnel),
            ],
            ..Evidence::default()
        };
        (ifaces, ev)
    }

    #[test]
    fn dev_machine_owners() {
        let (mut ifaces, ev) = dev_machine();
        assign(&mut ifaces, &ev);
        assert_eq!(ifaces[0].owner, None, "без адреса — без подписи");
        assert_eq!(
            ifaces[1].owner,
            Some(IfOwner::exact("sing-box", Some(75686)))
        );
        assert_eq!(
            ifaces[2].owner,
            Some(IfOwner::likely("OpenVPN", Some(92233)))
        );
    }

    #[test]
    fn helper_when_no_tunnel_client() {
        let (mut ifaces, mut ev) = dev_machine();
        ev.running.retain(|r| r.label != "OpenVPN");
        assign(&mut ifaces, &ev);
        assert_eq!(
            ifaces[2].owner,
            Some(IfOwner::likely("OpenVPN Connect", Some(341)))
        );
    }

    #[test]
    fn ambiguous_means_no_label() {
        let (mut ifaces, mut ev) = dev_machine();
        ev.running.push(run("WireGuard", 5, Role::Tunnel));
        assign(&mut ifaces, &ev);
        assert_eq!(ifaces[2].owner, None);
        // Два туннеля без владельца — тоже не угадываем.
        let (mut ifaces, mut ev) = dev_machine();
        ev.configs.clear();
        assign(&mut ifaces, &ev);
        assert!(ifaces[1].owner.is_none() && ifaces[2].owner.is_none());
    }

    #[test]
    fn utun_control_service_and_adapter() {
        let mut ifaces = vec![
            tun("utun9", Some("10.0.0.2")),
            tun("utun5", Some("10.1.0.2")),
        ];
        let ev = Evidence {
            utun: vec![("sing-box".into(), 812, 9)],
            services: vec![("Happ Plus".into(), "utun5".into())],
            ..Evidence::default()
        };
        assign(&mut ifaces, &ev);
        assert_eq!(ifaces[0].owner, Some(IfOwner::exact("sing-box", Some(812))));
        assert_eq!(ifaces[1].owner, Some(IfOwner::exact("Happ Plus", None)));

        assert_eq!(
            adapter_owner("Ethernet 3", Some("WireGuard Tunnel")),
            Some(IfOwner::exact("WireGuard", None))
        );
        assert_eq!(
            adapter_owner("Ethernet 2", Some("TAP-Windows Adapter V9")),
            Some(IfOwner::likely("OpenVPN", None))
        );
        assert_eq!(adapter_owner("Wi-Fi", Some("Intel(R) Wi-Fi 6")), None);
    }
}
