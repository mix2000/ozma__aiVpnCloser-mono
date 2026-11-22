//! Проверка на живой машине ТОЛЬКО ЧТЕНИЕМ: что нашёл discover, куда идёт
//! трафик к хостам агентов, сколько агентов было бы затронуто. Никаких
//! сигналов, никаких изменений сети.
//!
//! cargo run -p vpn-closer-core --example inspect

use std::time::Duration;

use std::sync::Arc;

use vpn_closer_core::models::{SelfCtx, Targets, DEFAULT_HOSTS};
use vpn_closer_core::testing::{
    collect, discover, interfaces, snapshot, summary, OsDns, OsNet, OsOwnerProbe, OwnerResolver,
    SystemChecks,
};

fn owners() -> OwnerResolver {
    OwnerResolver::new(Arc::new(OsOwnerProbe), std::process::id())
}

fn main() {
    let snap = snapshot();
    let d = discover(&OsNet, &mut owners(), &snap);

    println!("== Интерфейсы (туннельные и включённые)");
    for i in d
        .interfaces
        .iter()
        .filter(|i| i.tunnel || (i.up && !i.addrs.is_empty()))
    {
        let owner = i
            .owner
            .as_ref()
            .map(|o| format!(" — {} ({:?})", o.name, o.confidence))
            .unwrap_or_default();
        println!(
            "  {:<12} up={:<5} tunnel={:<5} {}{owner}",
            i.name,
            i.up,
            i.tunnel,
            i.addrs.join(", ")
        );
    }

    println!("== Процессы, похожие на VPN");
    for p in d.processes.iter().filter(|p| p.known_vpn) {
        println!("  {} (pid {})", p.name, p.pid);
    }

    println!("== Системные VPN");
    for v in &d.system_vpns {
        println!("  «{}» id={} connected={}", v.name, v.id, v.connected);
    }

    println!("== Маршрут к хостам агентов");
    let ifaces = interfaces(&OsNet);
    let mut sc = SystemChecks::new(Arc::new(OsNet), owners(), Arc::new(OsDns));
    for host in DEFAULT_HOSTS {
        let hosts = vec![host.to_string()];
        let mut res = (None, String::new());
        for _ in 0..100 {
            res = sc.check_route(&hosts, &ifaces, &[]);
            if res.0.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        println!("  {:?}: {}", res.0, res.1);
    }

    println!("== Агенты (без действий)");
    let me = SelfCtx::for_pid(std::process::id(), &snap);
    let list = collect(&snap, &Targets::default(), &me);
    println!(
        "  {} ; всего процессов в деревьях: {}",
        summary(&list),
        list.len()
    );
}
