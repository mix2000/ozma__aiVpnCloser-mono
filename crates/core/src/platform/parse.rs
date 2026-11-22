//! Чистые разборщики того, что отдаёт ОС (вывод команд, файлы, маски, имена
//! адаптеров). Собраны здесь, чтобы проверяться тестами на любой ОС.

use crate::models::SystemVpn;

// ---------------------------------------------------------------- интерфейсы

/// Туннельный интерфейс по имени (macOS/Unix).
#[cfg_attr(not(unix), allow(dead_code))]
pub fn is_tunnel_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [
        "utun",
        "ipsec",
        "ppp",
        "tun",
        "tap",
        "wg",
        "zt",
        "tailscale",
    ]
    .iter()
    .any(|p| n.starts_with(p))
}

/// Туннельный адаптер Windows по описанию/имени.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn is_tunnel_desc(text: &str) -> bool {
    let t = text.to_lowercase();
    [
        "wireguard",
        "wintun",
        "tap-windows",
        "tap-win",
        "openvpn",
        "vpn",
        "tun",
        "tailscale",
        "zerotier",
        "sing-box",
        "clash",
        "mihomo",
        "hiddify",
        "amnezia",
        "outline",
    ]
    .iter()
    .any(|k| t.contains(k))
}

/// Длина префикса по маске (сколько единиц подряд сначала).
#[cfg_attr(not(unix), allow(dead_code))]
pub fn prefix_len(mask: &[u8]) -> u8 {
    let mut n = 0u8;
    for b in mask {
        if *b == 0xff {
            n += 8;
        } else {
            n += b.leading_ones() as u8;
            break;
        }
    }
    n
}

// ---------------------------------------------------------------- системные VPN

/// Разобрать вывод `scutil --nc list` (macOS).
///
/// ```text
/// * (Disconnected)   14329CBD-... VPN (hossin.asaadi.V2Box) "V2BOX"   [VPN:hossin.asaadi.V2Box]
/// ```
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_scutil_nc_list(text: &str) -> Vec<SystemVpn> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(open) = line.find('(') else { continue };
        if !line[..open].trim().chars().all(|c| c == '*') {
            continue;
        }
        let rest = &line[open + 1..];
        let Some(close) = rest.find(')') else {
            continue;
        };
        let status = rest[..close].trim();
        let after = rest[close + 1..].trim_start();
        let id = after.split_whitespace().next().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        let name = match after.find('"') {
            Some(q1) => {
                let tail = &after[q1 + 1..];
                tail.find('"').map(|q2| tail[..q2].to_string())
            }
            None => None,
        }
        .unwrap_or_else(|| id.clone());
        out.push(SystemVpn {
            id,
            name,
            connected: status.eq_ignore_ascii_case("connected"),
        });
    }
    out
}

/// Разобрать вывод `rasdial` без аргументов (Windows): имена активных
/// подключений. Первая строка — заголовок («Connected to» / «Подключены к»),
/// последняя — «Command completed successfully.»; если строк меньше трёх —
/// подключений нет («No connections»).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn parse_rasdial(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 3 {
        return Vec::new();
    }
    lines[1..lines.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Имена подключений из телефонной книги RAS (`rasphone.pbk`): секции `[Имя]`.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn parse_pbk(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix('[').and_then(|r| r.strip_suffix(']')))
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .collect()
}

/// Список RAS-подключений Windows: настроенные (из телефонных книг, по
/// порядку, без дублей) + активные, которых нет в книгах.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn ras_connections(books: &[Vec<String>], connected: &[String]) -> Vec<SystemVpn> {
    let mut names: Vec<String> = Vec::new();
    for n in books.iter().flatten().chain(connected) {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    names
        .into_iter()
        .map(|n| SystemVpn {
            connected: connected.iter().any(|c| c.eq_ignore_ascii_case(&n)),
            id: n.clone(),
            name: n,
        })
        .collect()
}

// ---------------------------------------------------------------- владельцы туннелей

/// Сокеты `com.apple.net.utun_control` из `lsof -nP -F pn -p …`: (pid, номер
/// utun). Unit N у ядра соответствует интерфейсу utun{N-1}.
///
/// ```text
/// p812
/// n[ctl com.apple.net.utun_control id 9 unit 234]
/// ```
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_lsof_utun(text: &str) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let mut pid: Option<u32> = None;
    for line in text.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.trim().parse().ok();
            continue;
        }
        let Some(name) = line.strip_prefix('n') else {
            continue;
        };
        if !name.contains("com.apple.net.utun_control") {
            continue;
        }
        let unit = name
            .split_whitespace()
            .skip_while(|w| *w != "unit")
            .nth(1)
            .and_then(|u| u.trim_end_matches(']').parse::<u32>().ok());
        if let (Some(p), Some(u)) = (pid, unit) {
            if u >= 1 && !out.contains(&(p, u - 1)) {
                out.push((p, u - 1));
            }
        }
    }
    out
}

/// Имя интерфейса из `scutil --nc status <id>` подключённого сервиса
/// (`InterfaceName : utun5`).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_scutil_nc_status_iface(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == "InterfaceName" && !v.trim().is_empty()).then(|| v.trim().to_string())
    })
}

/// Аргументы процесса из буфера `sysctl KERN_PROCARGS2` (macOS): `argc` (i32),
/// путь к файлу, NUL-выравнивание, затем `argc` строк через NUL.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_procargs2(buf: &[u8]) -> Vec<String> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let argc = i32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]).max(0) as usize;
    let rest = &buf[4..];
    // Пропустить путь к исполняемому файлу и NUL после него.
    let Some(end) = rest.iter().position(|b| *b == 0) else {
        return Vec::new();
    };
    let mut parts = rest[end..].split(|b| *b == 0).skip_while(|s| s.is_empty());
    let mut out = Vec::with_capacity(argc);
    for _ in 0..argc {
        match parts.next() {
            Some(a) => out.push(String::from_utf8_lossy(a).into_owned()),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCUTIL: &str = r#"Available network connection services in the current set (*=enabled):
* (Disconnected)   14329CBD-347C-4F43-89BE-72532029B1A0 VPN (hossin.asaadi.V2Box) "V2BOX"                          [VPN:hossin.asaadi.V2Box]
* (Connected)      F7A1CC0B-C703-454D-968E-3F0C2888BCAC VPN (su.ffg.happ.plus) "Happ Plus"                      [VPN:su.ffg.happ.plus]
  (Disconnected)   AAAA-BBBB PPP (L2TP) "Office L2TP"     [PPP/L2TP]
"#;

    #[test]
    fn scutil_parsed() {
        let v = parse_scutil_nc_list(SCUTIL);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].id, "14329CBD-347C-4F43-89BE-72532029B1A0");
        assert_eq!(v[0].name, "V2BOX");
        assert!(!v[0].connected);
        assert_eq!(v[1].name, "Happ Plus");
        assert!(v[1].connected);
        assert_eq!(v[2].name, "Office L2TP");
        assert!(parse_scutil_nc_list("").is_empty());
        assert!(parse_scutil_nc_list(
            "Available network connection services in the current set (*=enabled):\n"
        )
        .is_empty());
    }

    #[test]
    fn rasdial_parsed() {
        assert_eq!(
            parse_rasdial("Connected to\r\nMy VPN\r\nCommand completed successfully.\r\n"),
            vec!["My VPN"]
        );
        assert_eq!(
            parse_rasdial("Подключены к\nРабота\nДом\nКоманда успешно выполнена.\n"),
            vec!["Работа", "Дом"]
        );
        assert!(parse_rasdial("No connections\r\nCommand completed successfully.\r\n").is_empty());
        assert!(parse_rasdial("").is_empty());
    }

    #[test]
    fn pbk_parsed() {
        let text = "[Office]\r\nMEDIA=rastapi\r\n\r\n[Home VPN]\r\nType=2\r\n";
        assert_eq!(parse_pbk(text), vec!["Office", "Home VPN"]);
    }

    #[test]
    fn ras_connections_merged() {
        let books = vec![
            vec!["Office".to_string(), "Home".to_string()],
            vec!["Office".to_string()],
        ];
        let connected = vec!["home".to_string(), "Adhoc".to_string()];
        let v = ras_connections(&books, &connected);
        let names: Vec<(&str, bool)> = v.iter().map(|s| (s.name.as_str(), s.connected)).collect();
        assert_eq!(
            names,
            vec![
                ("Office", false),
                ("Home", true),
                ("home", true),
                ("Adhoc", true)
            ]
        );
        assert_eq!(v[0].id, "Office");
    }

    #[test]
    fn tunnel_names() {
        for n in ["utun233", "utun4", "ipsec0", "ppp0", "tun0", "wg0"] {
            assert!(is_tunnel_name(n), "{n}");
        }
        for n in ["en0", "en1", "lo0", "bridge0", "awdl0"] {
            assert!(!is_tunnel_name(n), "{n}");
        }
        assert!(is_tunnel_desc("WireGuard Tunnel"));
        assert!(is_tunnel_desc("TAP-Windows Adapter V9"));
        assert!(!is_tunnel_desc("Intel(R) Wi-Fi 6 AX201"));
    }

    #[test]
    fn masks() {
        assert_eq!(prefix_len(&[255, 255, 255, 252]), 30);
        assert_eq!(prefix_len(&[255, 255, 255, 255]), 32);
        assert_eq!(prefix_len(&[255, 255, 0, 0]), 16);
        assert_eq!(prefix_len(&[0, 0, 0, 0]), 0);
    }

    #[test]
    fn lsof_utun_units() {
        let text = "p812\nfcwd\nn/\nn[ctl com.apple.net.utun_control id 9 unit 234]\n\
                    p900\nn[ctl com.apple.netsrc id 7 unit 16]\n\
                    p901\nn[ctl com.apple.net.utun_control id 9 unit 5]\n";
        assert_eq!(parse_lsof_utun(text), vec![(812, 233), (901, 4)]);
        assert!(parse_lsof_utun("").is_empty());
    }

    #[test]
    fn scutil_status_iface() {
        let text = "Connected\nExtended Status <dictionary> {\n  IPv4 : <dictionary> {\n    \
                    InterfaceName : utun7\n  }\n}\n";
        assert_eq!(parse_scutil_nc_status_iface(text).as_deref(), Some("utun7"));
        assert_eq!(parse_scutil_nc_status_iface("Disconnected\n"), None);
    }

    #[test]
    fn procargs2() {
        let mut buf = 3i32.to_ne_bytes().to_vec();
        buf.extend_from_slice(
            b"/opt/homebrew/bin/sing-box\0\0\0sing-box\0run\0-c /x\0KEY=secret\0",
        );
        assert_eq!(parse_procargs2(&buf), ["sing-box", "run", "-c /x"]);
        assert!(parse_procargs2(&[1, 0]).is_empty());
    }
}
