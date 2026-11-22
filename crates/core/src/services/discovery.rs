//! Что есть на машине (команда `discover`): собирается из [`NetProbe`] и снимка процессов.

use crate::models::{Discovery, NetInterface, ProcInfo};
use crate::platform::NetProbe;
use crate::services::net::normalize_interfaces;
use crate::services::owners::OwnerResolver;
use crate::services::vpn_processes::vpn_processes;

/// Интерфейсы для показа и проверок: без link-local, туннели первыми.
pub fn interfaces(net: &dyn NetProbe) -> Vec<NetInterface> {
    normalize_interfaces(net.interfaces())
}

/// Всё сразу; у туннелей — программа-владелец.
pub fn discover(net: &dyn NetProbe, owners: &mut OwnerResolver, procs: &[ProcInfo]) -> Discovery {
    let mut interfaces = interfaces(net);
    let system_vpns = net.system_vpns();
    owners.resolve(&mut interfaces, procs, &system_vpns);
    Discovery {
        interfaces,
        processes: vpn_processes(procs),
        system_vpns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IfAddr, SystemVpn};

    struct FakeNet;
    impl NetProbe for FakeNet {
        fn interfaces(&self) -> Vec<NetInterface> {
            vec![
                NetInterface::new("en0", true, false, vec![]),
                NetInterface::new(
                    "utun233",
                    true,
                    true,
                    vec![IfAddr {
                        ip: "172.19.0.1".parse().unwrap(),
                        prefix: 30,
                    }],
                ),
            ]
        }
        fn local_addr_for(&self, ip: std::net::IpAddr) -> std::io::Result<std::net::IpAddr> {
            Ok(ip)
        }
        fn system_vpns(&self) -> Vec<SystemVpn> {
            vec![SystemVpn {
                id: "A".into(),
                name: "V2BOX".into(),
                connected: false,
            }]
        }
    }

    struct NoOwners;
    impl crate::platform::OwnerProbe for NoOwners {
        fn utun_units(&self, _: &[u32]) -> Vec<(u32, u32)> {
            Vec::new()
        }
        fn read_config(&self, _: &std::path::Path) -> Option<String> {
            None
        }
        fn service_interface(&self, _: &str) -> Option<String> {
            None
        }
    }

    #[test]
    fn composes_sources() {
        let procs = vec![ProcInfo {
            pid: 7,
            name: "xray".into(),
            ..Default::default()
        }];
        let mut owners = OwnerResolver::new(std::sync::Arc::new(NoOwners), 1);
        let d = discover(&FakeNet, &mut owners, &procs);
        assert_eq!(d.interfaces[0].name, "utun233");
        assert_eq!(d.processes[0].pid, 7);
        assert_eq!(d.system_vpns[0].name, "V2BOX");
    }
}
