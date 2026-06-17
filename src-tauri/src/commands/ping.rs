// commands/ping.rs — M0 自检命令：从前端 invoke('ping') 看返回。
use crate::error::AppResult;

#[tauri::command]
pub fn ping() -> AppResult<String> {
    Ok(format!(
        "PromptCast backend alive · v{} · {}",
        env!("CARGO_PKG_VERSION"),
        chrono::Utc::now().to_rfc3339()
    ))
}
