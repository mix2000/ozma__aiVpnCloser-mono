//! Что найдено на машине (`discover`): интерфейсы, процессы VPN, системные VPN.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// Адрес интерфейса с длиной префикса.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IfAddr {
    pub ip: IpAddr,
    pub prefix: u8,
}

/// Насколько уверенно определена программа-владелец туннеля.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerConfidence {
    /// Точно: сокет utun_control, конфиг клиента, системный VPN, адаптер.
    Exact,
    /// Вероятно: единственный подходящий запущенный клиент.
    Likely,
}

/// Программа, которой принадлежит туннельный интерфейс.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IfOwner {
    /// Что показать: «sing-box», «OpenVPN», имя системного VPN.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub confidence: OwnerConfidence,
}

impl IfOwner {
    pub fn exact(name: impl Into<String>, pid: Option<u32>) -> IfOwner {
        IfOwner {
            name: name.into(),
            pid,
            confidence: OwnerConfidence::Exact,
        }
    }

    pub fn likely(name: impl Into<String>, pid: Option<u32>) -> IfOwner {
        IfOwner {
            name: name.into(),
            pid,
            confidence: OwnerConfidence::Likely,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetInterface {
    pub name: String,
    /// Адреса строками (`172.19.0.1`), без link-local IPv6.
    pub addrs: Vec<String>,
    pub up: bool,
    pub tunnel: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    /// Программа-владелец туннеля, если удалось определить.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<IfOwner>,
    /// Точка-точка (IFF_POINTOPOINT; на Windows — туннельный адаптер).
    #[serde(skip)]
    pub ptp: bool,
    /// Адреса с длиной префикса (для сравнения подсетей); в JSON не идут.
    #[serde(skip)]
    pub nets: Vec<IfAddr>,
}

impl NetInterface {
    pub fn new(name: &str, up: bool, tunnel: bool, nets: Vec<IfAddr>) -> NetInterface {
        NetInterface {
            name: name.to_string(),
            addrs: nets.iter().map(|n| n.ip.to_string()).collect(),
            up,
            tunnel,
            desc: None,
            owner: None,
            ptp: false,
            nets,
        }
    }

    pub fn has_ip(&self, ip: IpAddr) -> bool {
        self.nets.iter().any(|n| n.ip == ip)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnProcess {
    pub name: String,
    pub pid: u32,
    pub known_vpn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemVpn {
    pub id: String,
    pub name: String,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discovery {
    pub interfaces: Vec<NetInterface>,
    pub processes: Vec<VpnProcess>,
    pub system_vpns: Vec<SystemVpn>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape() {
        let mut i = NetInterface::new(
            "utun4",
            true,
            true,
            vec![IfAddr {
                ip: "10.8.0.6".parse().unwrap(),
                prefix: 32,
            }],
        );
        let v = serde_json::to_value(&i).unwrap();
        assert_eq!(v["addrs"][0], "10.8.0.6");
        assert!(v.get("desc").is_none() && v.get("nets").is_none());
        i.desc = Some("WireGuard".into());
        assert_eq!(serde_json::to_value(&i).unwrap()["desc"], "WireGuard");
        assert!(v.get("owner").is_none() && v.get("ptp").is_none());
        i.owner = Some(IfOwner::likely("OpenVPN", None));
        let v = serde_json::to_value(&i).unwrap();
        assert_eq!(
            v["owner"],
            serde_json::json!({"name": "OpenVPN", "confidence": "likely"})
        );
        i.owner = Some(IfOwner::exact("sing-box", Some(7)));
        assert_eq!(serde_json::to_value(&i).unwrap()["owner"]["pid"], 7);
        let d = serde_json::to_value(Discovery {
            processes: vec![VpnProcess {
                name: "xray".into(),
                pid: 1,
                known_vpn: true,
            }],
            ..Discovery::default()
        })
        .unwrap();
        assert_eq!(d["processes"][0]["knownVpn"], true);
        assert!(d.get("systemVpns").is_some());
    }
}
