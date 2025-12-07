use std::collections::HashMap;

use chrono::{Local, Utc};
use serde::Serialize;

use crate::log::LogLevel;

#[allow(dead_code)]
pub struct LogEntry {
    pub tag: String,
    pub message: String,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl LogEntry {
    pub fn new(tag: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            message: message.into(),
            data: None,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            file: None,
            line: None,
        }
    }
}

#[allow(dead_code)]
impl LogEntry {
    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        match serde_json::to_value(data) {
            Ok(value) => {
                let mut data_map = HashMap::new();
                if let serde_json::Value::Object(obj) = value {
                    for (k, v) in obj {
                        data_map.insert(k, v);
                    }
                } else {
                    data_map.insert("data".to_string(), value);
                }
                self.data = Some(data_map);
            }
            Err(_) => {
                eprint!("Failed to serialize data for LogEntry");
            }
        }
        self
    }

    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn format_native(&self, level: LogLevel) -> String {
        let reset = "\x1b[0m";
        let level_color = level.color_code();
        let tag_color = "\x1b[35m"; // マゼンタ
        let timestamp_color = "\x1b[90m"; // グレー

        let mut output = format!(
            "{}{} {}{}{}{} {}{} {}",
            timestamp_color,
            self.timestamp,
            level_color,
            level.as_str(),
            reset,
            tag_color,
            self.tag,
            reset,
            self.message,
        );

        if let Some(ref data) = self.data {
            if !data.is_empty() {
                output.push_str(&format!(
                    "\n Data:{}",
                    serde_json::to_string_pretty(data).unwrap_or_default()
                ));
            }
        }

        if let (Some(ref file), Some(line)) = (&self.file, self.line) {
            output.push_str(&format!("\n Location: {}:{}", file, line));
        }

        output
    }

    #[cfg(target_arch = "wasm32")]
    pub fn format_web(&self) -> (String, Option<serde_json::Value>) {
        let message = format!("[{}] {}: {}", self.tag, self.timestamp, self.message);
        (
            message,
            self.data.as_ref().map(|d| {
                serde_json::Value::Object(d.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }),
        )
    }
}
