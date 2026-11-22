//! Проверка на машине разработчика ТОЛЬКО ЧТЕНИЕМ (без сигналов и без изменений
//! сети). Ожидания — про конкретную машину из ТЗ, поэтому #[ignore]:
//! cargo test -p vpn-closer-core --test live_machine -- --ignored --nocapture
#![cfg(target_os = "macos")]

use std::time::Duration;

use std::sync::Arc;

use vpn_closer_core::testing::{
    discover, interfaces, snapshot, OsDns, OsNet, OsOwnerProbe, OwnerResolver, SystemChecks,
};

fn owners() -> OwnerResolver {
    OwnerResolver::new(Arc::new(OsOwnerProbe), std::process::id())
}

#[test]
#[ignore = "зависит от VPN на машине разработчика"]
fn discover_finds_vpn_of_dev_machine() {
    let d = discover(&OsNet, &mut owners(), &snapshot());
    let utun = d
        .interfaces
        .iter()
        .find(|i| i.addrs.iter().any(|a| a == "172.19.0.1"))
        .expect("интерфейс 172.19.0.1");
    println!("tun: {} {:?}", utun.name, utun.addrs);
    assert!(utun.tunnel && utun.up);
    assert!(d.processes.iter().any(|p| p.name == "xray" && p.known_vpn));
    for name in ["V2BOX", "Happ Plus"] {
        assert!(
            d.system_vpns.iter().any(|v| v.name == name),
            "нет системного VPN {name}"
        );
    }

    let ifaces = interfaces(&OsNet);
    let mut sc = SystemChecks::new(Arc::new(OsNet), owners(), Arc::new(OsDns));
    let hosts = vec!["api.anthropic.com".to_string()];
    let mut res = (None, String::new());
    for _ in 0..100 {
        res = sc.check_route(&hosts, &ifaces, &[]);
        if res.0.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    println!("route: {:?} {}", res.0, res.1);
    assert_eq!(res.0, Some(true), "{}", res.1);
    assert!(res.1.contains(&utun.name));
}
