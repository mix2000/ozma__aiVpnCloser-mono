//! Словарь VPN-клиентов: как показать программу и может ли она сама держать туннель.

use crate::models::{base_name, ProcInfo};

/// Что за процесс у клиента.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Сам поднимает туннель (sing-box, openvpn, tailscaled…).
    Tunnel,
    /// Фоновый помощник или окно клиента (ovpnagent, OpenVPN Connect).
    Helper,
    /// Туннель через системный VPN (Network Extension) — его видно в scutil.
    SystemVpn,
    /// Только прокси, туннель не создаёт (xray, ss-local…), если конфиг не скажет иного.
    Proxy,
}

/// Откуда читать конфиг клиента.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKind {
    SingBox,
    Xray,
    Clash,
    OpenVpn,
}

/// Известный клиент.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    /// Подпись в интерфейсе.
    pub label: &'static str,
    pub role: Role,
    pub config: Option<ConfigKind>,
}

const fn c(label: &'static str, role: Role, config: Option<ConfigKind>) -> Client {
    Client {
        label,
        role,
        config,
    }
}

/// Имя процесса (нижний регистр, без `.exe`) → клиент.
const EXACT: &[(&str, Client)] = &[
    (
        "sing-box",
        c("sing-box", Role::Tunnel, Some(ConfigKind::SingBox)),
    ),
    ("xray", c("xray", Role::Proxy, Some(ConfigKind::Xray))),
    ("v2ray", c("v2ray", Role::Proxy, None)),
    ("v2rayn", c("v2rayN", Role::Proxy, None)),
    ("v2raya", c("v2rayA", Role::Proxy, None)),
    ("clash", c("Clash", Role::Tunnel, Some(ConfigKind::Clash))),
    (
        "clash-meta",
        c("Clash", Role::Tunnel, Some(ConfigKind::Clash)),
    ),
    ("clash-verge", c("Clash Verge", Role::Helper, None)),
    ("mihomo", c("mihomo", Role::Tunnel, Some(ConfigKind::Clash))),
    (
        "verge-mihomo",
        c("mihomo", Role::Tunnel, Some(ConfigKind::Clash)),
    ),
    (
        "openvpn",
        c("OpenVPN", Role::Tunnel, Some(ConfigKind::OpenVpn)),
    ),
    ("openvpnserv", c("OpenVPN", Role::Helper, None)),
    ("openvpn-gui", c("OpenVPN", Role::Helper, None)),
    ("ovpnagent", c("OpenVPN Connect", Role::Helper, None)),
    ("ovpnhelper", c("OpenVPN Connect", Role::Helper, None)),
    ("openvpnconnect", c("OpenVPN Connect", Role::Helper, None)),
    ("openvpn connect", c("OpenVPN Connect", Role::Helper, None)),
    ("wireguard-go", c("WireGuard", Role::Tunnel, None)),
    ("boringtun", c("WireGuard", Role::Tunnel, None)),
    ("wireguard", c("WireGuard", Role::Helper, None)),
    ("tailscaled", c("Tailscale", Role::Tunnel, None)),
    ("tailscale", c("Tailscale", Role::Helper, None)),
    ("outline", c("Outline", Role::Helper, None)),
    ("outline-client", c("Outline", Role::Helper, None)),
    ("nekoray", c("NekoRay", Role::Helper, None)),
    ("nekobox", c("NekoBox", Role::Helper, None)),
    ("tun2socks", c("tun2socks", Role::Tunnel, None)),
    ("openconnect", c("OpenConnect", Role::Tunnel, None)),
    ("vpnagentd", c("Cisco AnyConnect", Role::Tunnel, None)),
    ("forticlient", c("FortiClient", Role::Helper, None)),
    ("mullvad-daemon", c("Mullvad", Role::Tunnel, None)),
    ("zerotier-one", c("ZeroTier", Role::Tunnel, None)),
    ("shadowsocks", c("Shadowsocks", Role::Proxy, None)),
    ("ss-local", c("Shadowsocks", Role::Proxy, None)),
    ("sslocal", c("Shadowsocks", Role::Proxy, None)),
    ("trojan", c("Trojan", Role::Proxy, None)),
    ("trojan-go", c("Trojan", Role::Proxy, None)),
    ("hysteria", c("Hysteria", Role::Proxy, None)),
    ("tuic", c("TUIC", Role::Proxy, None)),
    ("naive", c("NaiveProxy", Role::Proxy, None)),
];

/// Начало имени → клиент (`HappVPNWidgetExtension`, `hiddify-core`).
const PREFIX: &[(&str, Client)] = &[
    ("happ", c("Happ", Role::SystemVpn, None)),
    ("v2box", c("V2Box", Role::SystemVpn, None)),
    ("hiddify", c("Hiddify", Role::Tunnel, None)),
    ("amnezia", c("AmneziaVPN", Role::Tunnel, None)),
];

/// Клиент по имени процесса или исполняемого файла.
pub fn client_of(p: &ProcInfo) -> Option<Client> {
    let mut names = vec![base_name(&p.name)];
    if let Some(e) = &p.exe {
        names.push(base_name(e));
    }
    names.iter().find_map(|n| {
        EXACT
            .iter()
            .find(|(k, _)| k == n)
            .or_else(|| PREFIX.iter().find(|(k, _)| n.starts_with(k)))
            .map(|(_, c)| *c)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str, exe: Option<&str>) -> ProcInfo {
        ProcInfo {
            pid: 1,
            name: name.into(),
            exe: exe.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn known_clients() {
        let sb = client_of(&p("sing-box", None)).unwrap();
        assert_eq!((sb.label, sb.role), ("sing-box", Role::Tunnel));
        assert_eq!(sb.config, Some(ConfigKind::SingBox));
        assert_eq!(client_of(&p("xray", None)).unwrap().role, Role::Proxy);
        assert_eq!(
            client_of(&p("ovpnagent", None)).unwrap().label,
            "OpenVPN Connect"
        );
        let happ = client_of(&p("HappVPNWidgetExtension", None)).unwrap();
        assert_eq!(happ.role, Role::SystemVpn);
        assert_eq!(
            client_of(&p("x", Some(r"C:\Program Files\OpenVPN\bin\openvpn.exe")))
                .unwrap()
                .label,
            "OpenVPN"
        );
        assert!(client_of(&p("zsh", None)).is_none());
    }
}
