// src/log/logger.rs
use crate::log::{LogEntry, LogLevel};

pub struct Log;

impl Log {
    /// ログシステムを初期化
    pub fn init() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut builder = env_logger::Builder::from_default_env();

            // RUST_LOGが設定されていない場合のみデフォルトレベルを設定
            if std::env::var("RUST_LOG").is_err() {
                // 自分のアプリのみログを有効化
                builder
                    .filter_level(log::LevelFilter::Off) // 全体はOFFに
                    .filter_module("amus", log::LevelFilter::Info); // 自分のアプリのみINFO
            }

            // env_loggerのフォーマットをシンプルにする
            builder.format(|buf, record| {
                use std::io::Write;
                // メッセージのみを出力（自作フォーマットが既に含まれているため）
                writeln!(buf, "{}", record.args())
            });

            builder.init();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASMではコンパイル時の環境変数を使ってログレベルを設定
            let log_level = match option_env!("RUST_LOG")
                .unwrap_or("info")
                .to_lowercase()
                .as_str()
            {
                "error" => log::Level::Error,
                "warn" => log::Level::Warn,
                "info" => log::Level::Info,
                "debug" => log::Level::Debug,
                "trace" => log::Level::Trace,
                _ => log::Level::Info,
            };

            console_log::init_with_level(log_level).expect("Failed to initialize logger");
        }
    }

    /// エラーログ
    pub fn error(entry: LogEntry) {
        Self::log(LogLevel::Error, entry);
    }

    /// 警告ログ
    pub fn warn(entry: LogEntry) {
        Self::log(LogLevel::Warn, entry);
    }

    /// 情報ログ
    pub fn info(entry: LogEntry) {
        Self::log(LogLevel::Info, entry);
    }

    /// デバッグログ
    pub fn debug(entry: LogEntry) {
        Self::log(LogLevel::Debug, entry);
    }

    /// トレースログ
    pub fn trace(entry: LogEntry) {
        Self::log(LogLevel::Trace, entry);
    }

    fn log(level: LogLevel, entry: LogEntry) {
        let current_level = LogLevel::from_env();
        if level as u8 > current_level as u8 {
            return; // ログレベルが低い場合は出力しない
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let formatted = entry.format_native(level);
            match level {
                LogLevel::Error => log::error!("{}", formatted),
                LogLevel::Warn => log::warn!("{}", formatted),
                LogLevel::Info => log::info!("{}", formatted),
                LogLevel::Debug => log::debug!("{}", formatted),
                LogLevel::Trace => log::trace!("{}", formatted),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let (message, data) = entry.format_web();

            match level {
                LogLevel::Error => {
                    if let Some(data) = data {
                        let data_js = Self::value_to_js_value(data);
                        web_sys::console::error_2(&message.into(), &data_js);
                    } else {
                        web_sys::console::error_1(&message.into());
                    }
                }
                LogLevel::Warn => {
                    if let Some(data) = data {
                        let data_js = Self::value_to_js_value(data);
                        web_sys::console::warn_2(&message.into(), &data_js);
                    } else {
                        web_sys::console::warn_1(&message.into());
                    }
                }
                LogLevel::Info => {
                    if let Some(data) = data {
                        let data_js = Self::value_to_js_value(data);
                        web_sys::console::info_2(&message.into(), &data_js);
                    } else {
                        web_sys::console::info_1(&message.into());
                    }
                }
                LogLevel::Debug | LogLevel::Trace => {
                    if let Some(data) = data {
                        let data_js = Self::value_to_js_value(data);
                        web_sys::console::log_2(&message.into(), &data_js);
                    } else {
                        web_sys::console::log_1(&message.into());
                    }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn value_to_js_value(value: serde_json::Value) -> wasm_bindgen::JsValue {
        use wasm_bindgen::JsValue;

        match value {
            serde_json::Value::Null => JsValue::NULL,
            serde_json::Value::Bool(b) => JsValue::from_bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    JsValue::from_f64(i as f64)
                } else if let Some(f) = n.as_f64() {
                    JsValue::from_f64(f)
                } else {
                    JsValue::from_str(&n.to_string())
                }
            }
            serde_json::Value::String(s) => JsValue::from_str(&s),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                // 複雑なオブジェクトは文字列として表示
                JsValue::from_str(&serde_json::to_string_pretty(&value).unwrap_or_default())
            }
        }
    }
}

/// ログ作成マクロ
#[macro_export]
macro_rules! log_info {
    ($tag:expr, $message:expr) => {
        crate::log::Log::info(
            crate::log::LogEntry::new($tag, $message).with_location(file!(), line!()),
        );
    };
    ($tag:expr, $message:expr, $data:expr) => {
        crate::log::Log::info(
            crate::log::LogEntry::new($tag, $message)
                .with_data($data)
                .with_location(file!(), line!()),
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($tag:expr, $message:expr) => {
        crate::log::Log::error(
            crate::log::LogEntry::new($tag, $message).with_location(file!(), line!()),
        );
    };
    ($tag:expr, $message:expr, $data:expr) => {
        crate::log::Log::error(
            crate::log::LogEntry::new($tag, $message)
                .with_data($data)
                .with_location(file!(), line!()),
        );
    };
}

#[macro_export]
macro_rules! log_warn {
    ($tag:expr, $message:expr) => {
        crate::log::Log::warn(
            crate::log::LogEntry::new($tag, $message).with_location(file!(), line!()),
        );
    };
    ($tag:expr, $message:expr, $data:expr) => {
        crate::log::Log::warn(
            crate::log::LogEntry::new($tag, $message)
                .with_data($data)
                .with_location(file!(), line!()),
        );
    };
}

#[macro_export]
macro_rules! log_debug {
    ($tag:expr, $message:expr) => {
        crate::log::Log::debug(
            crate::log::LogEntry::new($tag, $message).with_location(file!(), line!()),
        );
    };
    ($tag:expr, $message:expr, $data:expr) => {
        crate::log::Log::debug(
            crate::log::LogEntry::new($tag, $message)
                .with_data($data)
                .with_location(file!(), line!()),
        );
    };
}
