//! Unix (macOS и прочие): интерфейсы через getifaddrs, сигналы через kill.

use std::collections::BTreeMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::parse::{is_tunnel_name, prefix_len};
use crate::models::{Action, IfAddr, NetInterface};

// ---------------------------------------------------------------- интерфейсы

pub fn interfaces() -> Vec<NetInterface> {
    let mut map: BTreeMap<String, NetInterface> = BTreeMap::new();
    let mut head: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: getifaddrs заполняет связный список, который мы читаем и
    // освобождаем freeifaddrs; указатели не переживают этот блок.
    unsafe {
        if libc::getifaddrs(&mut head) != 0 {
            return Vec::new();
        }
        let mut cur = head;
        while !cur.is_null() {
            let ifa = &*cur;
            cur = ifa.ifa_next;
            if ifa.ifa_name.is_null() {
                continue;
            }
            let name = std::ffi::CStr::from_ptr(ifa.ifa_name)
                .to_string_lossy()
                .into_owned();
            let flags = ifa.ifa_flags as libc::c_int;
            let up = flags & libc::IFF_UP != 0 && flags & libc::IFF_RUNNING != 0;
            let entry = map
                .entry(name.clone())
                .or_insert_with(|| NetInterface::new(&name, up, is_tunnel_name(&name), Vec::new()));
            entry.up |= up;
            entry.ptp |= flags & libc::IFF_POINTOPOINT != 0;
            if let Some(addr) = sockaddr_ip(ifa.ifa_addr) {
                let prefix = netmask_prefix(ifa.ifa_netmask, &addr);
                entry.nets.push(IfAddr { ip: addr, prefix });
            }
        }
        libc::freeifaddrs(head);
    }
    map.into_values().collect()
}

unsafe fn sockaddr_ip(sa: *const libc::sockaddr) -> Option<IpAddr> {
    if sa.is_null() {
        return None;
    }
    match (*sa).sa_family as libc::c_int {
        libc::AF_INET => {
            let sin = &*(sa as *const libc::sockaddr_in);
            Some(IpAddr::V4(Ipv4Addr::from(u32::from_be(
                sin.sin_addr.s_addr,
            ))))
        }
        libc::AF_INET6 => {
            let sin6 = &*(sa as *const libc::sockaddr_in6);
            Some(IpAddr::V6(Ipv6Addr::from(sin6.sin6_addr.s6_addr)))
        }
        _ => None,
    }
}

unsafe fn netmask_prefix(sa: *const libc::sockaddr, addr: &IpAddr) -> u8 {
    let full = if addr.is_ipv4() { 32 } else { 128 };
    if sa.is_null() {
        return full;
    }
    // На macOS у маски бывает sa_family = 0, поэтому смотрим на семейство адреса.
    match addr {
        IpAddr::V4(_) => {
            let sin = &*(sa as *const libc::sockaddr_in);
            prefix_len(&sin.sin_addr.s_addr.to_ne_bytes())
        }
        IpAddr::V6(_) => {
            let sin6 = &*(sa as *const libc::sockaddr_in6);
            prefix_len(&sin6.sin6_addr.s6_addr)
        }
    }
}

// ---------------------------------------------------------------- сигналы

fn sig(pid: u32, s: libc::c_int) -> io::Result<()> {
    // SAFETY: обычный системный вызов; pid проверен вызывающим (> 1, ≤ i32::MAX).
    let rc = unsafe { libc::kill(pid as libc::pid_t, s) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// SIGSTOP / SIGCONT / SIGCONT+SIGTERM / SIGKILL.
pub fn apply(action: Action, pid: u32) -> io::Result<()> {
    match action {
        Action::Freeze => sig(pid, libc::SIGSTOP),
        Action::Resume => sig(pid, libc::SIGCONT),
        Action::Terminate => {
            // Замороженный процесс не обработает TERM, пока не продолжится.
            let _ = sig(pid, libc::SIGCONT);
            sig(pid, libc::SIGTERM)
        }
        Action::Kill => sig(pid, libc::SIGKILL),
    }
}

pub fn is_alive(pid: u32) -> bool {
    // SAFETY: сигнал 0 ничего не посылает, только проверяет существование.
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    rc == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}
