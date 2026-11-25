//! Windows: адаптеры через GetAdaptersAddresses, RAS-подключения через
//! `rasdial` и телефонные книги, заморозка через NtSuspendProcess (ntdll).

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::windows::process::CommandExt;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_BUFFER_OVERFLOW, HANDLE, NO_ERROR,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
    IF_TYPE_PPP, IF_TYPE_PROP_VIRTUAL, IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH,
};
use windows_sys::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows_sys::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SUSPEND_RESUME, PROCESS_TERMINATE,
};

use super::parse::{is_tunnel_desc, parse_pbk, parse_rasdial, ras_connections};
use crate::models::{Action, IfAddr, NetInterface, SystemVpn};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// ---------------------------------------------------------------- интерфейсы

unsafe fn wide(p: *const u16) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *p.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
}

unsafe fn sock_ip(sa: *const SOCKADDR) -> Option<IpAddr> {
    if sa.is_null() {
        return None;
    }
    match (*sa).sa_family {
        AF_INET => {
            let sin = &*(sa as *const SOCKADDR_IN);
            Some(IpAddr::V4(Ipv4Addr::from(u32::from_be(
                sin.sin_addr.S_un.S_addr,
            ))))
        }
        AF_INET6 => {
            let sin6 = &*(sa as *const SOCKADDR_IN6);
            Some(IpAddr::V6(Ipv6Addr::from(sin6.sin6_addr.u.Byte)))
        }
        _ => None,
    }
}

/// Буфер со списком адаптеров (GetAdaptersAddresses, с ростом размера).
fn adapters_buffer() -> Option<Vec<u64>> {
    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;
    let mut size: u32 = 16 * 1024;
    for _ in 0..4 {
        let mut buf = vec![0u64; (size as usize).div_ceil(8)];
        // SAFETY: буфер выровнен по 8 и имеет размер size байт.
        let rc = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC as u32,
                flags,
                std::ptr::null(),
                buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH,
                &mut size,
            )
        };
        if rc == NO_ERROR {
            return Some(buf);
        }
        if rc != ERROR_BUFFER_OVERFLOW {
            return None;
        }
    }
    None
}

pub fn interfaces() -> Vec<NetInterface> {
    let Some(buf) = adapters_buffer() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // SAFETY: список заполнен системой внутри buf и живёт, пока жив buf.
    unsafe {
        let mut cur = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
        while !cur.is_null() {
            let a = &*cur;
            cur = a.Next;
            out.push(adapter(a));
        }
    }
    out
}

/// Один адаптер: имя, описание, состояние, туннель ли, адреса.
///
/// # Safety
/// `a` — элемент списка из [`adapters_buffer`], пока жив буфер.
unsafe fn adapter(a: &IP_ADAPTER_ADDRESSES_LH) -> NetInterface {
    let name = wide(a.FriendlyName);
    let desc = wide(a.Description);
    let up = a.OperStatus == IfOperStatusUp;
    let tunnel = a.IfType == IF_TYPE_TUNNEL
        || a.IfType == IF_TYPE_PPP
        || ((a.IfType == IF_TYPE_PROP_VIRTUAL || a.IfType == 6)
            && (is_tunnel_desc(&desc) || is_tunnel_desc(&name)));
    let tunnel = tunnel && !desc.to_lowercase().contains("loopback");
    let mut nets = Vec::new();
    let mut u = a.FirstUnicastAddress;
    while !u.is_null() {
        let ua = &*u;
        u = ua.Next;
        if let Some(ip) = sock_ip(ua.Address.lpSockaddr) {
            nets.push(IfAddr {
                ip,
                prefix: ua.OnLinkPrefixLength,
            });
        }
    }
    let mut iface = NetInterface::new(&name, up, tunnel, nets);
    iface.ptp = tunnel;
    if !desc.is_empty() {
        iface.desc = Some(desc);
    }
    iface
}

// ---------------------------------------------------------------- RAS

pub fn system_vpns() -> Vec<SystemVpn> {
    let connected: Vec<String> = std::process::Command::new("rasdial")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| parse_rasdial(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    let books: Vec<Vec<String>> = [
        std::env::var_os("APPDATA").map(std::path::PathBuf::from),
        std::env::var_os("PROGRAMDATA").map(std::path::PathBuf::from),
    ]
    .into_iter()
    .flatten()
    .filter_map(|base| {
        let p = base.join(r"Microsoft\Network\Connections\Pbk\rasphone.pbk");
        std::fs::read(&p).ok()
    })
    .map(|bytes| parse_pbk(&String::from_utf8_lossy(&bytes)))
    .collect();
    ras_connections(&books, &connected)
}

// ---------------------------------------------------------------- процессы

#[link(name = "ntdll")]
extern "system" {
    fn NtSuspendProcess(process: HANDLE) -> i32;
    fn NtResumeProcess(process: HANDLE) -> i32;
}

const STILL_ACTIVE: u32 = 259;

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: хэндл получен от OpenProcess и закрывается один раз.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn open(pid: u32, access: u32) -> io::Result<Handle> {
    // SAFETY: обычный системный вызов; null — ошибка.
    let h = unsafe { OpenProcess(access, 0, pid) };
    if h.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(Handle(h))
    }
}

fn nt(status: i32) -> io::Result<()> {
    if status >= 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "NTSTATUS 0x{:08X}",
            status as u32
        )))
    }
}

/// NtSuspendProcess / NtResumeProcess / TerminateProcess.
pub fn apply(action: Action, pid: u32) -> io::Result<()> {
    match action {
        Action::Freeze => {
            let h = open(pid, PROCESS_SUSPEND_RESUME)?;
            // SAFETY: валидный хэндл с правом SUSPEND_RESUME.
            nt(unsafe { NtSuspendProcess(h.0) })
        }
        Action::Resume => {
            let h = open(pid, PROCESS_SUSPEND_RESUME)?;
            // SAFETY: валидный хэндл с правом SUSPEND_RESUME.
            nt(unsafe { NtResumeProcess(h.0) })
        }
        Action::Terminate | Action::Kill => {
            let h = open(pid, PROCESS_TERMINATE)?;
            // SAFETY: валидный хэндл с правом TERMINATE.
            if unsafe { TerminateProcess(h.0, 1) } != 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

pub fn is_alive(pid: u32) -> bool {
    let h = match open(pid, PROCESS_QUERY_LIMITED_INFORMATION) {
        Ok(h) => h,
        // Нет доступа — значит, есть к чему (как EPERM на Unix).
        Err(e) => return e.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32),
    };
    let mut code = 0u32;
    // SAFETY: валидный хэндл, указатель на локальную переменную.
    let ok = unsafe { GetExitCodeProcess(h.0, &mut code) } != 0;
    ok && code == STILL_ACTIVE
}
