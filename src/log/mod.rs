pub mod log_entry;
pub mod logger;

pub use log_entry::*;
pub use logger::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match std::env::var("RUST_LOG")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "error" => LogLevel::Error,
                "warn" => LogLevel::Warn,
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _ => LogLevel::Info,
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASMではコンパイル時の環境変数を使用
            let rust_log = option_env!("RUST_LOG").unwrap_or("info");
            match rust_log.to_lowercase().as_str() {
                "error" => LogLevel::Error,
                "warn" => LogLevel::Warn,
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _ => {
                    web_sys::console::log_1(
                        &format!("Unknown log level '{}', defaulting to Info", rust_log).into(),
                    );
                    LogLevel::Info
                }
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Error => "\x1b[31m", // 赤
            LogLevel::Warn => "\x1b[33m",  // 黄色
            LogLevel::Info => "\x1b[32m",  // 緑
            LogLevel::Debug => "\x1b[36m", // シアン
            LogLevel::Trace => "\x1b[37m", // 白
        }
    }
}
