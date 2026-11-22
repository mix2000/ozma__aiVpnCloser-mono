//! Конфиги VPN-клиентов: где лежит (по argv) и какой туннель описывает.
//!
//! Читаются ТОЛЬКО поля туннеля (имя интерфейса и его адрес). Остальное
//! содержимое (серверы, ключи) не разбирается, не хранится и не пишется в журнал.

use std::net::IpAddr;
use std::path::PathBuf;

use serde_json::Value;

use super::clients::ConfigKind;
use crate::services::config_validation::parse_subnet;
use crate::services::net::same_net;

/// Туннель из конфига: имя интерфейса и/или его подсети.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TunSpec {
    pub name: Option<String>,
    pub nets: Vec<(IpAddr, u8)>,
}

impl TunSpec {
    fn is_empty(&self) -> bool {
        self.name.is_none() && self.nets.is_empty()
    }

    /// Описывает ли конфиг этот интерфейс: по имени или по подсети.
    pub fn matches(&self, name: &str, ips: &[IpAddr]) -> bool {
        self.name.as_deref() == Some(name)
            || self
                .nets
                .iter()
                .any(|(net, p)| ips.iter().any(|ip| same_net(*ip, *net, *p)))
    }
}

/// Путь к конфигу по аргументам запуска. Относительный путь берётся от
/// рабочей папки, указанной в аргументах; без неё — не угадываем.
pub fn config_path(kind: ConfigKind, argv: &[String]) -> Option<PathBuf> {
    let (file_flags, dir_flags, default_name): (&[&str], &[&str], Option<&str>) = match kind {
        ConfigKind::SingBox => (
            &["-c", "--config"],
            &["-D", "--directory"],
            Some("config.json"),
        ),
        ConfigKind::Xray => (&["-c", "-config", "--config"], &[], None),
        ConfigKind::Clash => (&["-f"], &["-d"], Some("config.yaml")),
        ConfigKind::OpenVpn => (&["--config"], &["--cd"], None),
    };
    let file = flag_value(argv, file_flags);
    let dir = flag_value(argv, dir_flags).map(PathBuf::from);
    let path = match (file, &dir, default_name) {
        (Some(f), _, _) => PathBuf::from(f),
        (None, Some(d), Some(name)) => d.join(name),
        _ => return None,
    };
    if path.is_absolute() {
        Some(path)
    } else {
        dir.filter(|d| d.is_absolute()).map(|d| d.join(path))
    }
}

/// Значение флага: `-c path` или `-c=path`.
fn flag_value<'a>(argv: &'a [String], flags: &[&str]) -> Option<&'a str> {
    let mut it = argv.iter().skip(1);
    while let Some(a) = it.next() {
        for f in flags {
            if a == f {
                return it.next().map(String::as_str);
            }
            if let Some(v) = a.strip_prefix(f).and_then(|r| r.strip_prefix('=')) {
                return Some(v);
            }
        }
    }
    None
}

/// Туннель из текста конфига.
pub fn tun_spec(kind: ConfigKind, text: &str) -> Option<TunSpec> {
    let spec = match kind {
        ConfigKind::SingBox => sing_box(text),
        ConfigKind::Xray => xray(text),
        ConfigKind::Clash => clash(text),
        ConfigKind::OpenVpn => openvpn(text),
    }?;
    (!spec.is_empty()).then_some(spec)
}

fn nets_of(v: Option<&Value>) -> Vec<(IpAddr, u8)> {
    let items: Vec<&str> = match v {
        Some(Value::String(s)) => vec![s.as_str()],
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    items
        .into_iter()
        .filter_map(parse_subnet)
        .map(|(ip, p)| (ip, p.unwrap_or(if ip.is_ipv4() { 32 } else { 128 })))
        .collect()
}

/// sing-box: `inbounds[].type == "tun"` → `interface_name`, `address` /
/// `inet4_address` / `inet6_address`.
pub fn sing_box(text: &str) -> Option<TunSpec> {
    let v: Value = serde_json::from_str(text).ok()?;
    let tun = v["inbounds"]
        .as_array()?
        .iter()
        .find(|i| i["type"] == "tun")?;
    let mut nets = nets_of(tun.get("address"));
    nets.extend(nets_of(tun.get("inet4_address")));
    nets.extend(nets_of(tun.get("inet6_address")));
    Some(TunSpec {
        name: tun["interface_name"].as_str().map(str::to_string),
        nets,
    })
}

/// xray: `inbounds[].protocol == "tun"` → `settings.name`.
pub fn xray(text: &str) -> Option<TunSpec> {
    let v: Value = serde_json::from_str(text).ok()?;
    let tun = v["inbounds"]
        .as_array()?
        .iter()
        .find(|i| i["protocol"] == "tun")?;
    let s = &tun["settings"];
    let name = s["name"].as_str().or(s["interfaceName"].as_str());
    Some(TunSpec {
        name: name.map(str::to_string),
        nets: nets_of(s.get("address")),
    })
}

/// clash/mihomo YAML: блок `tun:` верхнего уровня → `device`, `inet4-address`.
/// Разбор построчный, только этого блока.
pub fn clash(text: &str) -> Option<TunSpec> {
    let mut in_tun = false;
    let mut in_addr = false;
    let mut spec = TunSpec::default();
    let mut found = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let top = !line.starts_with(' ') && !line.starts_with('\t');
        if top {
            in_tun = trimmed == "tun:";
            found |= in_tun;
            in_addr = false;
            continue;
        }
        if !in_tun {
            continue;
        }
        let unquote = |s: &str| s.trim().trim_matches(['"', '\'']).to_string();
        if let Some(item) = trimmed.strip_prefix("- ") {
            if in_addr {
                spec.nets
                    .extend(nets_of(Some(&Value::String(unquote(item)))));
            }
            continue;
        }
        in_addr = false;
        if let Some((k, v)) = trimmed.split_once(':') {
            match k.trim() {
                "device" if !v.trim().is_empty() => spec.name = Some(unquote(v)),
                "inet4-address" | "inet6-address" => {
                    let v = v.trim().trim_start_matches('[').trim_end_matches(']');
                    if v.is_empty() {
                        in_addr = true;
                    } else {
                        for part in v.split(',') {
                            spec.nets
                                .extend(nets_of(Some(&Value::String(unquote(part)))));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    found.then_some(spec)
}

/// OpenVPN: строка `dev utun7` (конкретное имя; просто `dev tun` ничего не даёт).
pub fn openvpn(text: &str) -> Option<TunSpec> {
    let dev = text.lines().find_map(|l| {
        let mut w = l.split_whitespace();
        (w.next() == Some("dev")).then(|| w.next()).flatten()
    })?;
    let generic = ["tun", "tap", "null"].contains(&dev);
    Some(TunSpec {
        name: (!generic).then(|| dev.to_string()),
        nets: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Абсолютная папка на любой ОС (на Windows `/etc/…` — не абсолютный путь).
    fn base() -> PathBuf {
        std::env::temp_dir().join("vpn-closer-cfg")
    }

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    fn s(p: &std::path::Path) -> &str {
        p.to_str().expect("utf-8 путь")
    }

    #[test]
    fn paths_from_argv() {
        let b = base();
        let sb = b.join("sing-box");
        let conf = sb.join("config.json");
        assert_eq!(
            config_path(
                ConfigKind::SingBox,
                &argv(&["sing-box", "run", "-c", s(&conf)])
            ),
            Some(conf.clone())
        );
        assert_eq!(
            config_path(
                ConfigKind::SingBox,
                &argv(&["sing-box", "run", "-D", s(&sb), "-c", "conf.json"])
            ),
            Some(sb.join("conf.json"))
        );
        assert_eq!(
            config_path(
                ConfigKind::SingBox,
                &argv(&["sing-box", "run", "-D", s(&sb)])
            ),
            Some(sb.join("config.json"))
        );
        assert_eq!(
            config_path(
                ConfigKind::SingBox,
                &argv(&["sing-box", "run", "-c", "rel.json"])
            ),
            None
        );
        let xray = b.join("c.json");
        assert_eq!(
            config_path(
                ConfigKind::Xray,
                &argv(&["xray", "run", &format!("-config={}", s(&xray))])
            ),
            Some(xray.clone())
        );
        let mihomo = b.join("mihomo");
        assert_eq!(
            config_path(ConfigKind::Clash, &argv(&["mihomo", "-d", s(&mihomo)])),
            Some(mihomo.join("config.yaml"))
        );
        let ovpn = b.join("openvpn").join("client");
        assert_eq!(
            config_path(
                ConfigKind::OpenVpn,
                &argv(&["openvpn", "--config", "client.ovpn", "--cd", s(&ovpn)])
            ),
            Some(ovpn.join("client.ovpn"))
        );
        assert_eq!(config_path(ConfigKind::OpenVpn, &argv(&["openvpn"])), None);
    }

    #[test]
    fn sing_box_tun() {
        let text = r#"{"log":{},"inbounds":[{"type":"mixed","listen_port":2080},
            {"type":"tun","interface_name":"utun233","address":["172.19.0.1/30"]}],
            "outbounds":[{"type":"vless","server":"secret.example","uuid":"k"}]}"#;
        let s = sing_box(text).unwrap();
        assert_eq!(s.name.as_deref(), Some("utun233"));
        assert_eq!(s.nets, vec![("172.19.0.1".parse().unwrap(), 30)]);
        let old = r#"{"inbounds":[{"type":"tun","inet4_address":"172.19.0.1/30"}]}"#;
        let s = sing_box(old).unwrap();
        assert!(s.matches("utun9", &["172.19.0.2".parse().unwrap()]));
        assert!(!s.matches("utun9", &["10.0.0.1".parse().unwrap()]));
        assert!(sing_box(r#"{"inbounds":[{"type":"mixed"}]}"#).is_none());
        assert!(sing_box("не json").is_none());
    }

    #[test]
    fn xray_tun() {
        let text =
            r#"{"inbounds":[{"protocol":"socks"},{"protocol":"tun","settings":{"name":"xray0"}}]}"#;
        assert_eq!(xray(text).unwrap().name.as_deref(), Some("xray0"));
        assert!(xray(r#"{"inbounds":[{"protocol":"socks"}]}"#).is_none());
    }

    #[test]
    fn clash_tun() {
        let text =
            "mixed-port: 7890\nproxies:\n  - name: x\n    server: secret\ntun:\n  enable: true\n  \
                    device: utun199\n  inet4-address:\n    - 198.18.0.1/30\ndns:\n  enable: true\n";
        let s = clash(text).unwrap();
        assert_eq!(s.name.as_deref(), Some("utun199"));
        assert_eq!(s.nets, vec![("198.18.0.1".parse().unwrap(), 30)]);
        let inline = "tun:\n  inet4-address: [\"198.18.0.1/16\"]\n";
        assert_eq!(clash(inline).unwrap().nets.len(), 1);
        assert!(clash("dns:\n  enable: true\n").is_none());
    }

    #[test]
    fn openvpn_dev() {
        assert_eq!(
            openvpn("client\ndev utun7\nremote x 1194\n")
                .unwrap()
                .name
                .as_deref(),
            Some("utun7")
        );
        assert_eq!(openvpn("dev tun\n").unwrap().name, None);
        assert!(tun_spec(ConfigKind::OpenVpn, "dev tun\n").is_none());
        assert!(openvpn("remote x\n").is_none());
    }
}
