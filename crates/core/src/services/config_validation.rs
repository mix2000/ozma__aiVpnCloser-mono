//! Проверка и нормализация настроек перед сохранением.

use std::net::IpAddr;

use crate::error::{Error, Result};
use crate::models::{Config, Limits, Range, WatchItem};

/// Пределы числовых настроек — единственный источник (UI получает их командой).
pub const LIMITS: Limits = Limits {
    poll_ms: Range {
        min: 200,
        max: 60_000,
    },
    confirm_failures: Range { min: 1, max: 20 },
    close_after_sec: Range { min: 0, max: 600 },
    kill_grace_sec: Range { min: 0, max: 60 },
};

/// Проверить и привести к нормальному виду (обрезать пробелы, убрать
/// дубликаты, нормализовать хосты). Ошибка — текст по-русски для UI.
pub fn validate(mut cfg: Config) -> Result<Config> {
    check_limits(&cfg)?;
    cfg.hosts = normalize_hosts(&cfg.hosts)?;
    let mut watch: Vec<WatchItem> = Vec::new();
    for item in cfg.watch {
        let item = normalize_item(item)?;
        if !watch.contains(&item) {
            watch.push(item);
        }
    }
    if watch.contains(&WatchItem::Route) && cfg.hosts.is_empty() {
        return Err(Error::invalid(
            "Для проверки «трафик к агентам идёт через VPN» нужен хотя бы один хост",
        ));
    }
    cfg.watch = watch;
    Ok(cfg)
}

fn normalize_hosts(raw: &[String]) -> Result<Vec<String>> {
    let mut hosts: Vec<String> = Vec::new();
    for h in raw {
        let h = normalize_host(h)?;
        if !h.is_empty() && !hosts.contains(&h) {
            hosts.push(h);
        }
    }
    Ok(hosts)
}

fn check_limits(cfg: &Config) -> Result<()> {
    let l = LIMITS;
    if !l.poll_ms.contains(cfg.poll_ms) {
        return Err(Error::invalid(format!(
            "Интервал опроса должен быть от {} до {} мс",
            l.poll_ms.min, l.poll_ms.max
        )));
    }
    if !l.confirm_failures.contains(cfg.confirm_failures) {
        return Err(Error::invalid(format!(
            "Число подтверждений должно быть от {} до {}",
            l.confirm_failures.min, l.confirm_failures.max
        )));
    }
    if !l.close_after_sec.contains(cfg.close_after_sec) {
        return Err(Error::invalid(format!(
            "«Закрыть через» — не больше {} секунд",
            l.close_after_sec.max
        )));
    }
    if !l.kill_grace_sec.contains(cfg.kill_grace_sec) {
        return Err(Error::invalid(format!(
            "Ожидание перед принудительным закрытием — не больше {} секунд",
            l.kill_grace_sec.max
        )));
    }
    Ok(())
}

fn normalize_host(raw: &str) -> Result<String> {
    let mut h = raw.trim().to_ascii_lowercase();
    for scheme in ["https://", "http://"] {
        if let Some(rest) = h.strip_prefix(scheme) {
            h = rest.to_string();
        }
    }
    if let Some((head, _)) = h.split_once('/') {
        h = head.to_string();
    }
    let h = h.trim_end_matches('.').to_string();
    if h.is_empty() {
        return Ok(h);
    }
    let ok = h.len() <= 253
        && h.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == ':')
        && !h.starts_with('-')
        && !h.starts_with('.');
    if !ok {
        return Err(Error::invalid(format!(
            "Неверный адрес хоста: «{}»",
            raw.trim()
        )));
    }
    Ok(h)
}

fn normalize_item(item: WatchItem) -> Result<WatchItem> {
    Ok(match item {
        WatchItem::Interface { name, addr } => {
            let name = name.trim().to_string();
            let addr = addr.map(|a| a.trim().to_string()).filter(|a| !a.is_empty());
            if let Some(a) = &addr {
                if parse_subnet(a).is_none() {
                    return Err(Error::invalid(format!(
                        "Неверный адрес интерфейса: «{a}» (нужно вроде 172.19.0.1 или 172.19.0.0/30)"
                    )));
                }
            }
            if name.is_empty() && addr.is_none() {
                return Err(Error::invalid("У интерфейса должно быть имя или адрес"));
            }
            WatchItem::Interface { name, addr }
        }
        WatchItem::Process { name } => {
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(Error::invalid("Укажите имя процесса VPN"));
            }
            WatchItem::Process { name }
        }
        WatchItem::SystemVpn { id, name } => {
            let id = id.trim().to_string();
            let name = name.trim().to_string();
            if id.is_empty() && name.is_empty() {
                return Err(Error::invalid(
                    "У системного VPN должно быть имя или идентификатор",
                ));
            }
            WatchItem::SystemVpn { id, name }
        }
        WatchItem::Route => WatchItem::Route,
    })
}

/// `"172.19.0.1"` → (адрес, None); `"172.19.0.0/30"` → (адрес, Some(30)).
pub fn parse_subnet(s: &str) -> Option<(IpAddr, Option<u8>)> {
    let (ip, prefix) = match s.split_once('/') {
        Some((ip, p)) => (ip, Some(p.trim().parse::<u8>().ok()?)),
        None => (s, None),
    };
    let ip: IpAddr = ip.trim().parse().ok()?;
    let max = if ip.is_ipv4() { 32 } else { 128 };
    if prefix.is_some_and(|p| p > max) {
        return None;
    }
    Some((ip, prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(c: Config) -> String {
        validate(c).unwrap_err().to_string()
    }

    #[test]
    fn validate_errors_in_russian() {
        assert!(err(Config {
            poll_ms: 10,
            ..Config::default()
        })
        .contains("Интервал опроса"));
        assert!(err(Config {
            close_after_sec: 601,
            ..Config::default()
        })
        .contains("Закрыть через"));
        assert!(validate(Config {
            confirm_failures: 0,
            ..Config::default()
        })
        .is_err());
        assert!(err(Config {
            watch: vec![WatchItem::Route],
            hosts: vec![],
            ..Config::default()
        })
        .contains("хост"));
        assert!(err(Config {
            watch: vec![WatchItem::Interface {
                name: "utun1".into(),
                addr: Some("300.1.1.1".into()),
            }],
            ..Config::default()
        })
        .contains("адрес"));
        assert!(validate(Config {
            watch: vec![WatchItem::Process { name: " ".into() }],
            ..Config::default()
        })
        .is_err());
        assert!(err(Config {
            hosts: vec!["bad host".into()],
            ..Config::default()
        })
        .contains("хоста"));
    }

    #[test]
    fn validate_normalizes() {
        let c = validate(Config {
            hosts: vec![
                " HTTPS://Claude.ai/path ".into(),
                "claude.ai".into(),
                "".into(),
            ],
            watch: vec![
                WatchItem::Route,
                WatchItem::Route,
                WatchItem::Process {
                    name: " xray ".into(),
                },
            ],
            ..Config::default()
        })
        .unwrap();
        assert_eq!(c.hosts, vec!["claude.ai"]);
        assert_eq!(c.watch.len(), 2);
        assert_eq!(
            c.watch[1],
            WatchItem::Process {
                name: "xray".into()
            }
        );
    }

    #[test]
    fn subnet_parse() {
        assert!(parse_subnet("172.19.0.1").is_some());
        assert_eq!(parse_subnet("172.19.0.0/30").unwrap().1, Some(30));
        assert!(parse_subnet("172.19.0.0/33").is_none());
        assert!(parse_subnet("nope").is_none());
    }
}
