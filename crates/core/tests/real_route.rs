//! Проверка маршрута на настоящей ОС, только чтением: loopback — не туннель,
//! а выбранный как «туннель» — подходит. Сети наружу не нужно.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use vpn_closer_core::testing::{
    interfaces, OsDns, OsNet, OsOwnerProbe, OwnerResolver, SystemChecks,
};

#[test]
fn route_on_loopback_is_not_tunnel() {
    let lo: IpAddr = "127.0.0.1".parse().unwrap();
    let ifaces = interfaces(&OsNet);
    let owners = OwnerResolver::new(Arc::new(OsOwnerProbe), std::process::id());
    let mut sc = SystemChecks::new(Arc::new(OsNet), owners, Arc::new(OsDns));
    let hosts = vec!["127.0.0.1".to_string()];
    let mut res = (None, String::new());
    for _ in 0..50 {
        res = sc.check_route(&hosts, &ifaces, &[]);
        if res.0.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(res.0, Some(false), "{}", res.1);
    assert!(res.1.contains("не через VPN"), "{}", res.1);
    let lo_iface = ifaces.iter().find(|i| i.has_ip(lo)).unwrap();
    let res = sc.check_route(&hosts, &ifaces, &[lo_iface]);
    assert_eq!(res.0, Some(true), "{}", res.1);
}
