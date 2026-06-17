// events.rs — 跨窗口事件名约定。
//
// 任意 Rust 命令可以 emit；前端 `listen('xxx-changed', ...)`
// 在所有窗口里订阅，本窗口数据自动刷新。
pub const PROMPTS_CHANGED: &str = "prompts-changed";
pub const FOLDERS_CHANGED: &str = "folders-changed";
pub const TAGS_CHANGED: &str = "tags-changed";
pub const SITES_CHANGED: &str = "sites-changed";
pub const SETTINGS_CHANGED: &str = "settings-changed";
pub const THEME_CHANGED: &str = "theme-changed";
pub const CLIPBOARD_CHANGED: &str = "clipboard-changed";
pub const SYNC_STATUS_CHANGED: &str = "sync-status-changed";

use tauri::{AppHandle, Emitter};

pub fn emit_prompts_changed(app: &AppHandle) {
    let _ = app.emit(PROMPTS_CHANGED, ());
}
pub fn emit_folders_changed(app: &AppHandle) {
    let _ = app.emit(FOLDERS_CHANGED, ());
}
pub fn emit_tags_changed(app: &AppHandle) {
    let _ = app.emit(TAGS_CHANGED, ());
}
pub fn emit_sites_changed(app: &AppHandle) {
    let _ = app.emit(SITES_CHANGED, ());
}
pub fn emit_settings_changed(app: &AppHandle, key: &str) {
    let _ = app.emit(SETTINGS_CHANGED, key);
}
pub fn emit_theme_changed(app: &AppHandle, theme: &str) {
    let _ = app.emit(THEME_CHANGED, theme);
}
pub fn emit_clipboard_changed(app: &AppHandle) {
    let _ = app.emit(CLIPBOARD_CHANGED, ());
}
pub fn emit_sync_status_changed<S: serde::Serialize + Clone>(app: &AppHandle, status: S) {
    let _ = app.emit(SYNC_STATUS_CHANGED, status);
}
