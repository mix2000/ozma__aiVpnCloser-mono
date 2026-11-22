//! Настоящие реализации trait'ов `platform` поверх функций конкретной ОС.

use std::io;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::Path;

use super::{
    os_apply, os_interfaces, os_is_alive, os_process_args, os_service_interface, os_system_vpns,
    os_utun_units, DnsDone, DnsResolver, NetProbe, OwnerProbe, ProcessControl,
};
use crate::models::{Action, NetInterface, SystemVpn};

/// Сколько читать из конфига, не больше.
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

/// Настоящая сеть ОС.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsNet;

impl NetProbe for OsNet {
    fn interfaces(&self) -> Vec<NetInterface> {
        os_interfaces()
    }

    fn system_vpns(&self) -> Vec<SystemVpn> {
        os_system_vpns()
    }

    fn local_addr_for(&self, ip: IpAddr) -> io::Result<IpAddr> {
        local_addr_for(ip)
    }
}

/// UDP `connect` ничего не отправляет — только выбирает маршрут.
pub fn local_addr_for(ip: IpAddr) -> io::Result<IpAddr> {
    let bind: SocketAddr = if ip.is_ipv4() {
        "0.0.0.0:0".parse().expect("valid")
    } else {
        "[::]:0".parse().expect("valid")
    };
    let sock = UdpSocket::bind(bind)?;
    sock.connect(SocketAddr::new(ip, 443))?;
    Ok(sock.local_addr()?.ip())
}

/// Системный резолвер в отдельном потоке: медленный DNS не задерживает опрос.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsDns;

impl DnsResolver for OsDns {
    fn resolve_in_background(&self, host: String, done: DnsDone) {
        std::thread::spawn(move || done(resolve(&host)));
    }
}

/// IP-адрес — сразу, имя — через системный резолвер.
fn resolve(host: &str) -> io::Result<Vec<IpAddr>> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(vec![ip]);
    }
    (host, 443)
        .to_socket_addrs()
        .map(|it| it.map(|sa| sa.ip()).collect())
}

/// Настоящие сигналы ОС.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsProcessControl;

impl ProcessControl for OsProcessControl {
    fn apply(&self, action: Action, pid: u32) -> io::Result<()> {
        os_apply(action, pid)
    }

    fn is_alive(&self, pid: u32) -> bool {
        pid != 0 && pid <= i32::MAX as u32 && os_is_alive(pid)
    }
}

/// Настоящие источники владельцев туннелей.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsOwnerProbe;

impl OwnerProbe for OsOwnerProbe {
    fn utun_units(&self, pids: &[u32]) -> Vec<(u32, u32)> {
        os_utun_units(pids)
    }

    fn read_config(&self, path: &Path) -> Option<String> {
        let mut text = String::new();
        std::fs::File::open(path)
            .ok()?
            .take(MAX_CONFIG_BYTES)
            .read_to_string(&mut text)
            .ok()?;
        Some(text)
    }

    fn service_interface(&self, id: &str) -> Option<String> {
        os_service_interface(id)
    }

    fn process_args(&self, pid: u32) -> Option<Vec<String>> {
        os_process_args(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn os_interfaces_have_loopback() {
        let lo: IpAddr = "127.0.0.1".parse().unwrap();
        assert!(OsNet.interfaces().iter().any(|i| i.has_ip(lo)));
    }

    #[test]
    fn loopback_route_source() {
        let lo: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(OsNet.local_addr_for(lo).unwrap(), lo);
    }

    #[test]
    fn dns_ip_literal_in_background() {
        let (tx, rx) = mpsc::channel();
        OsDns.resolve_in_background("127.0.0.1".into(), Box::new(move |r| tx.send(r).unwrap()));
        let ips = rx.recv().unwrap().unwrap();
        assert_eq!(ips, vec!["127.0.0.1".parse::<IpAddr>().unwrap()]);
    }

    #[test]
    fn config_reading_is_bounded_and_safe() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.json");
        std::fs::write(&p, "{}").unwrap();
        assert_eq!(OsOwnerProbe.read_config(&p).as_deref(), Some("{}"));
        assert!(OsOwnerProbe.read_config(&dir.path().join("нет")).is_none());
        assert!(OsOwnerProbe.utun_units(&[]).is_empty());
    }

    #[test]
    fn is_alive_rejects_bad_pids() {
        assert!(!OsProcessControl.is_alive(0));
        assert!(!OsProcessControl.is_alive(u32::MAX));
        assert!(OsProcessControl.is_alive(std::process::id()));
    }
}
