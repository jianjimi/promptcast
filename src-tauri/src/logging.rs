// logging.rs — 全局日志：app_data_dir/logs/app.log（按天滚）+ stderr。
//
// 前端通过 IPC `log_record` 把 JS 侧日志合并到同一文件，便于一站式排查。
use std::path::Path;

use tauri::Manager;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

/// 返回 guard：必须保留到进程结束，否则 worker 线程提前退出。
pub fn init(log_dir: &Path) -> WorkerGuard {
    std::fs::create_dir_all(log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,prompt_manager_lib=debug"));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true);

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stderr_layer)
        .init();

    tracing::info!("logging initialized at {}", log_dir.display());
    guard
}

/// 把前端 JS 日志接进来，与后端日志混在同一文件。
#[derive(Debug, serde::Deserialize)]
pub struct FrontendLog {
    pub level: String,    // "trace" | "debug" | "info" | "warn" | "error"
    pub source: String,   // "drawer" | "preview" | "editor" | "settings" | ...
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[tauri::command]
pub fn log_record(entry: FrontendLog) {
    let level = match entry.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    let msg = match entry.data {
        Some(d) => format!("[FE/{}] {} | {}", entry.source, entry.message, d),
        None => format!("[FE/{}] {}", entry.source, entry.message),
    };
    match level {
        Level::TRACE => tracing::trace!("{msg}"),
        Level::DEBUG => tracing::debug!("{msg}"),
        Level::INFO => tracing::info!("{msg}"),
        Level::WARN => tracing::warn!("{msg}"),
        Level::ERROR => tracing::error!("{msg}"),
    }
}

#[tauri::command]
pub fn log_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("logs");
    Ok(dir.to_string_lossy().to_string())
}
