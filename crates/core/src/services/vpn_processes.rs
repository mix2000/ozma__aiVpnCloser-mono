//! Процессы VPN-клиентов: какие имена похожи на VPN, список для выбора в UI.

use std::collections::HashMap;

use crate::models::{base_name, ProcInfo, VpnProcess};

/// Известные VPN-клиенты (имя процесса в нижнем регистре, без `.exe`).
pub const KNOWN_VPN_PROCESSES: &[&str] = &[
    "xray",
    "sing-box",
    "v2ray",
    "v2rayn",
    "v2raya",
    "openvpn",
    "openvpn-gui",
    "openvpnconnect",
    "openvpnserv",
    "ovpnagent",
    "wireguard",
    "wireguard-go",
    "wg-quick",
    "boringtun",
    "clash",
    "clash-verge",
    "clash-meta",
    "verge-mihomo",
    "mihomo",
    "hiddify",
    "hiddifycli",
    "hiddify-core",
    "nekoray",
    "nekobox",
    "amnezia",
    "amneziavpn",
    "amneziawg",
    "outline",
    "outline-client",
    "tailscaled",
    "tailscale",
    "happ",
    "v2box",
    "shadowsocks",
    "ss-local",
    "sslocal",
    "trojan",
    "trojan-go",
    "tun2socks",
    "hysteria",
    "tuic",
    "naive",
    "openconnect",
    "vpnagentd",
    "forticlient",
    "mullvad-daemon",
    "zerotier-one",
];

/// Начала имён семейств VPN-клиентов (`HappVPNWidgetExtension`, `clash-verge-service`).
const VPN_NAME_PREFIXES: &[&str] = &[
    "xray",
    "sing-box",
    "v2ray",
    "clash",
    "mihomo",
    "hiddify",
    "nekoray",
    "amnezia",
    "wireguard",
    "openvpn",
    "outline",
    "tailscale",
    "happ",
    "v2box",
];

/// Похоже ли имя процесса на VPN-клиент.
pub fn is_known_vpn(name: &str) -> bool {
    let n = base_name(name);
    KNOWN_VPN_PROCESSES.contains(&n.as_str())
        || n.contains("vpn")
        || VPN_NAME_PREFIXES.iter().any(|p| n.starts_with(p))
}

/// Процессы для выбора в UI: по одному на имя, похожие на VPN — первыми.
pub fn vpn_processes(procs: &[ProcInfo]) -> Vec<VpnProcess> {
    let mut seen = HashMap::<String, VpnProcess>::new();
    for p in procs {
        if p.name.is_empty() {
            continue;
        }
        let key = p.name.to_lowercase();
        seen.entry(key).or_insert_with(|| VpnProcess {
            name: p.name.clone(),
            pid: p.pid,
            known_vpn: is_known_vpn(&p.name),
        });
    }
    let mut list: Vec<VpnProcess> = seen.into_values().collect();
    list.sort_by(|a, b| {
        b.known_vpn
            .cmp(&a.known_vpn)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    list
}

/// Совпадает ли процесс с именем из пункта `process` (без учёта регистра и `.exe`).
pub fn process_matches(p: &ProcInfo, wanted: &str) -> bool {
    let w = base_name(wanted.trim());
    if w.is_empty() {
        return false;
    }
    base_name(&p.name) == w || p.exe.as_deref().is_some_and(|e| base_name(e) == w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vpn_names() {
        for n in [
            "xray",
            "sing-box",
            "Happ",
            "V2Box",
            "openvpn.exe",
            "HappVPNWidgetExtension",
        ] {
            assert!(is_known_vpn(n), "{n}");
        }
        for n in ["zsh", "Finder", "claude"] {
            assert!(!is_known_vpn(n), "{n}");
        }
    }

    #[test]
    fn process_match() {
        let p = ProcInfo {
            pid: 1,
            name: "xray.exe".into(),
            ..Default::default()
        };
        assert!(process_matches(&p, "XRAY"));
        assert!(process_matches(&p, "xray.exe"));
        assert!(!process_matches(&p, "x"));
    }

    #[test]
    fn list_unique_vpn_first() {
        let p = |pid, name: &str| ProcInfo {
            pid,
            name: name.into(),
            ..Default::default()
        };
        let list = vpn_processes(&[
            p(1, "zsh"),
            p(2, "xray"),
            p(3, "Xray"),
            p(4, ""),
            p(5, "bash"),
        ]);
        let names: Vec<(&str, bool)> = list
            .iter()
            .map(|v| (v.name.as_str(), v.known_vpn))
            .collect();
        assert_eq!(names.len(), 3);
        assert!(names[0].1 && names[0].0.eq_ignore_ascii_case("xray"));
        assert_eq!(&names[1..], [("bash", false), ("zsh", false)]);
    }
}
