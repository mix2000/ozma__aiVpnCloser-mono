//! Подставная платформа для `Engine`: машина из ТЗ (utun233 от sing-box, en1),
//! процессы claude и sing-box, действия — только в журнал, без сигналов.
#![allow(dead_code)]

use std::io;
use std::net::IpAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use vpn_closer_core::models::{Action, IfAddr, NetInterface, ProcInfo, SystemVpn};
use vpn_closer_core::testing::{
    DnsDone, DnsResolver, NetProbe, OwnerProbe, PlatformParts, ProcSource, ProcessControl,
    SystemClock,
};

pub const SELF_PID: u32 = 999_999;

/// Конфиг sing-box на подставной машине: абсолютный путь на любой ОС.
pub fn sing_box_config() -> std::path::PathBuf {
    std::env::temp_dir()
        .join("vpn-closer-cfg")
        .join("sing-box")
        .join("config.json")
}

/// Флажок «трафик идёт через туннель» и журнал действий.
#[derive(Clone, Default)]
pub struct Machine {
    pub via_tunnel: Arc<Mutex<bool>>,
    pub actions: Arc<Mutex<Vec<(Action, u32)>>>,
}

impl Machine {
    pub fn new() -> Self {
        let m = Machine::default();
        *m.via_tunnel.lock().unwrap() = true;
        m
    }

    pub fn set_via_tunnel(&self, on: bool) {
        *self.via_tunnel.lock().unwrap() = on;
    }

    pub fn did(&self, action: Action) -> bool {
        self.actions
            .lock()
            .unwrap()
            .iter()
            .any(|(a, _)| *a == action)
    }

    pub fn parts(&self) -> PlatformParts {
        PlatformParts {
            net: Arc::new(FakeNet(self.via_tunnel.clone())),
            dns: Arc::new(InstantDns),
            owners: Arc::new(FakeOwners),
            control: Arc::new(RecControl(self.actions.clone())),
            procs: Arc::new(|| Box::new(MachineProcs)),
            clock: Arc::new(SystemClock::default()),
            self_pid: SELF_PID,
        }
    }
}

struct FakeNet(Arc<Mutex<bool>>);
impl NetProbe for FakeNet {
    fn interfaces(&self) -> Vec<NetInterface> {
        let net = |ip: &str, prefix| IfAddr {
            ip: ip.parse().unwrap(),
            prefix,
        };
        let mut tun = NetInterface::new("utun233", true, true, vec![net("172.19.0.1", 30)]);
        tun.ptp = true;
        vec![
            tun,
            NetInterface::new("en1", true, false, vec![net("192.168.1.20", 24)]),
        ]
    }
    fn system_vpns(&self) -> Vec<SystemVpn> {
        Vec::new()
    }
    fn local_addr_for(&self, _: IpAddr) -> io::Result<IpAddr> {
        let via = if *self.0.lock().unwrap() {
            "172.19.0.1"
        } else {
            "192.168.1.20"
        };
        Ok(via.parse().unwrap())
    }
}

struct InstantDns;
impl DnsResolver for InstantDns {
    fn resolve_in_background(&self, host: String, done: DnsDone) {
        done(
            host.parse::<IpAddr>()
                .map(|ip| vec![ip])
                .map_err(io::Error::other),
        );
    }
}

/// Конфиг sing-box читается и говорит про utun233.
struct FakeOwners;
impl OwnerProbe for FakeOwners {
    fn utun_units(&self, _: &[u32]) -> Vec<(u32, u32)> {
        Vec::new()
    }
    fn read_config(&self, path: &Path) -> Option<String> {
        (path == sing_box_config())
            .then(|| r#"{"inbounds":[{"type":"tun","interface_name":"utun233"}]}"#.to_string())
    }
    fn service_interface(&self, _: &str) -> Option<String> {
        None
    }
}

/// Действия — только в журнал.
struct RecControl(Arc<Mutex<Vec<(Action, u32)>>>);
impl ProcessControl for RecControl {
    fn apply(&self, action: Action, pid: u32) -> io::Result<()> {
        self.0.lock().unwrap().push((action, pid));
        Ok(())
    }
    fn is_alive(&self, _: u32) -> bool {
        true
    }
}

pub struct MachineProcs;
impl ProcSource for MachineProcs {
    fn snapshot(&mut self) -> Vec<ProcInfo> {
        let p = |pid, ppid, name: &str, cmd: &str| ProcInfo {
            pid,
            ppid: Some(ppid),
            uid: Some("501".into()),
            name: name.into(),
            exe: None,
            cmd: cmd.split_whitespace().map(str::to_string).collect(),
            start_time: 1,
        };
        vec![
            p(
                75686,
                1,
                "sing-box",
                &format!("sing-box run -c {}", sing_box_config().display()),
            ),
            p(4242, 1, "claude", "claude"),
            p(4243, 4242, "node", "node /x/mcp.js"),
        ]
    }
}

/// Ждать условия до 5 с.
pub fn wait_for(mut f: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}
