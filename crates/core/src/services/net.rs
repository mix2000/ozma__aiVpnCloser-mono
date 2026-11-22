//! Чистая логика над сетевыми интерфейсами: подсети, порядок показа.

use std::net::IpAddr;

use crate::models::NetInterface;

/// Лежат ли адреса в одной подсети с данным префиксом.
pub fn same_net(a: IpAddr, b: IpAddr, prefix: u8) -> bool {
    match (a, b) {
        (IpAddr::V4(a), IpAddr::V4(b)) => {
            let p = prefix.min(32) as u32;
            if p == 0 {
                return true;
            }
            let mask = u32::MAX << (32 - p);
            (u32::from(a) & mask) == (u32::from(b) & mask)
        }
        (IpAddr::V6(a), IpAddr::V6(b)) => {
            let p = prefix.min(128) as u32;
            if p == 0 {
                return true;
            }
            let mask = u128::MAX << (128 - p);
            (u128::from(a) & mask) == (u128::from(b) & mask)
        }
        _ => false,
    }
}

fn is_link_local(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V6(v6) => (v6.segments()[0] & 0xffc0) == 0xfe80,
        IpAddr::V4(_) => false,
    }
}

/// Убрать link-local IPv6, пересобрать строки адресов и упорядочить:
/// туннели, затем включённые, затем по имени.
pub fn normalize_interfaces(mut list: Vec<NetInterface>) -> Vec<NetInterface> {
    for i in &mut list {
        i.nets.retain(|n| !is_link_local(&n.ip));
        i.addrs = i.nets.iter().map(|n| n.ip.to_string()).collect();
    }
    list.sort_by(|a, b| {
        b.tunnel
            .cmp(&a.tunnel)
            .then(b.up.cmp(&a.up))
            .then(a.name.cmp(&b.name))
    });
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IfAddr;

    #[test]
    fn subnets() {
        let a: IpAddr = "172.19.0.1".parse().unwrap();
        let b: IpAddr = "172.19.0.2".parse().unwrap();
        let c: IpAddr = "172.19.0.5".parse().unwrap();
        assert!(same_net(a, b, 30));
        assert!(!same_net(a, c, 30));
        assert!(same_net(a, c, 24));
        assert!(same_net(a, c, 0));
        assert!(!same_net(a, "::1".parse().unwrap(), 0));
    }

    #[test]
    fn normalize_drops_link_local_and_sorts() {
        let net = |ip: &str| IfAddr {
            ip: ip.parse().unwrap(),
            prefix: 64,
        };
        let list = vec![
            NetInterface::new("en0", true, false, vec![net("192.168.1.2")]),
            NetInterface::new("utun0", true, true, vec![net("fe80::1")]),
            NetInterface::new("utun9", false, true, vec![]),
            NetInterface::new("utun233", true, true, vec![net("172.19.0.1")]),
        ];
        let out = normalize_interfaces(list);
        let names: Vec<&str> = out.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["utun0", "utun233", "utun9", "en0"]);
        assert!(out[0].addrs.is_empty() && out[0].nets.is_empty());
        assert_eq!(out[1].addrs, ["172.19.0.1"]);
    }
}
