pub mod chat;
pub mod client;
pub mod config;
pub mod history;
pub mod tools;

/// 简单的 AI 事件日志（输出到 stderr，生产环境可替换为 tracing/log crate）
#[macro_export]
macro_rules! ai_log {
    ($($arg:tt)*) => {
        eprintln!("[AI] {}", format!($($arg)*));
    };
}
