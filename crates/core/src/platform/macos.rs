//! macOS: системные VPN из `scutil --nc list` / `--nc status`, сокеты
//! utun_control у своих процессов через `lsof`.

use super::parse::{
    parse_lsof_utun, parse_procargs2, parse_scutil_nc_list, parse_scutil_nc_status_iface,
};
use crate::models::SystemVpn;

pub fn system_vpns() -> Vec<SystemVpn> {
    match std::process::Command::new("/usr/sbin/scutil")
        .args(["--nc", "list"])
        .output()
    {
        Ok(out) => parse_scutil_nc_list(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => Vec::new(),
    }
}

/// Интерфейс подключённого сервиса: `scutil --nc status <id>`.
pub fn service_interface(id: &str) -> Option<String> {
    let out = std::process::Command::new("/usr/sbin/scutil")
        .args(["--nc", "status", id])
        .output()
        .ok()?;
    parse_scutil_nc_status_iface(&String::from_utf8_lossy(&out.stdout))
}

/// Kernel control utun_control у процессов (`lsof -nP -F pn -p …`). Без sudo
/// lsof видит только процессы своего пользователя — их и спрашиваем.
pub fn utun_units(pids: &[u32]) -> Vec<(u32, u32)> {
    if pids.is_empty() {
        return Vec::new();
    }
    let list: Vec<String> = pids.iter().map(u32::to_string).collect();
    match std::process::Command::new("/usr/sbin/lsof")
        .args(["-nP", "-F", "pn", "-p", &list.join(",")])
        .output()
    {
        Ok(out) => parse_lsof_utun(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => Vec::new(),
    }
}

/// Аргументы запуска процесса. Свой процесс — `sysctl KERN_PROCARGS2`
/// (переменные окружения после аргументов не читаются); чужой (root) ядро не
/// отдаёт — тогда строка из `/bin/ps -o args=` (ps на macOS это может), по
/// словам: пробелы внутри путей теряются, для путей конфигов этого хватает.
pub fn process_args(pid: u32) -> Option<Vec<String>> {
    procargs(pid).or_else(|| ps_args(pid))
}

fn ps_args(pid: u32) -> Option<Vec<String>> {
    let out = std::process::Command::new("/bin/ps")
        .args(["-o", "args=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let args: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    (!args.is_empty()).then_some(args)
}

fn procargs(pid: u32) -> Option<Vec<String>> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as libc::c_int];
    let mut size: libc::size_t = 0;
    // SAFETY: первый вызов только узнаёт размер; указатели на локальные переменные.
    let rc = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc != 0 || size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size];
    // SAFETY: буфер размера size, ядро пишет не больше size и возвращает фактический размер.
    let rc = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc != 0 {
        return None;
    }
    buf.truncate(size);
    let args = parse_procargs2(&buf);
    (!args.is_empty()).then_some(args)
}
