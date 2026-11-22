//! Журнал событий: `events.jsonl`, хранятся последние [`MAX_EVENTS`] записей.

use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::atomic::write_atomic;
use crate::models::Event;

pub const MAX_EVENTS: usize = 500;

/// Журнал в памяти + файл. Файл дописывается построчно; когда в нём набирается
/// вдвое больше лимита, он атомарно переписывается последними записями.
pub struct EventLog {
    path: Option<PathBuf>,
    items: VecDeque<Event>,
    lines_in_file: usize,
}

impl EventLog {
    /// Журнал только в памяти (для тестов).
    #[cfg(test)]
    pub fn in_memory() -> EventLog {
        EventLog {
            path: None,
            items: VecDeque::new(),
            lines_in_file: 0,
        }
    }

    /// Открыть файл; битые строки пропускаются.
    pub fn open(path: &Path) -> EventLog {
        let mut items = VecDeque::new();
        let mut lines = 0;
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                lines += 1;
                if let Ok(ev) = serde_json::from_str::<Event>(line) {
                    items.push_back(ev);
                    if items.len() > MAX_EVENTS {
                        items.pop_front();
                    }
                }
            }
        }
        EventLog {
            path: Some(path.to_path_buf()),
            items,
            lines_in_file: lines,
        }
    }

    pub fn push(&mut self, ev: Event) {
        self.items.push_back(ev.clone());
        while self.items.len() > MAX_EVENTS {
            self.items.pop_front();
        }
        let Some(path) = self.path.clone() else {
            return;
        };
        if self.lines_in_file + 1 > MAX_EVENTS * 2 {
            self.rewrite(&path);
            return;
        }
        let Ok(line) = serde_json::to_string(&ev) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let res = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut f| writeln!(f, "{line}"));
        if res.is_ok() {
            self.lines_in_file += 1;
        }
    }

    fn rewrite(&mut self, path: &Path) {
        let mut text = String::new();
        for ev in &self.items {
            if let Ok(line) = serde_json::to_string(ev) {
                text.push_str(&line);
                text.push('\n');
            }
        }
        if write_atomic(path, text.as_bytes()).is_ok() {
            self.lines_in_file = self.items.len();
        }
    }

    /// Последние `limit` событий, новые первыми.
    pub fn recent(&self, limit: usize) -> Vec<Event> {
        self.items.iter().rev().take(limit).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EventKind;

    #[test]
    fn rotation_keeps_last_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        let mut log = EventLog::open(&path);
        for i in 0..(MAX_EVENTS * 2 + 30) {
            log.push(Event::new(
                "2026-11-22T10:00:00Z",
                EventKind::Info,
                format!("e{i}"),
            ));
        }
        let lines = std::fs::read_to_string(&path).unwrap().lines().count();
        assert!(lines <= MAX_EVENTS * 2, "{lines}");
        let recent = log.recent(3);
        assert_eq!(recent[0].message, format!("e{}", MAX_EVENTS * 2 + 29));
        let reopened = EventLog::open(&path);
        assert_eq!(reopened.recent(MAX_EVENTS + 10).len(), MAX_EVENTS);
        assert_eq!(reopened.recent(1)[0].message, recent[0].message);
    }

    #[test]
    fn bad_lines_skipped_and_kind_serialized() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        std::fs::write(&path, "garbage\n").unwrap();
        let mut log = EventLog::open(&path);
        log.push(Event::new("2026-11-22T10:00:00Z", EventKind::VpnDown, "x"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"kind\":\"vpn_down\""));
        assert_eq!(EventLog::open(&path).recent(10).len(), 1);
    }
}
