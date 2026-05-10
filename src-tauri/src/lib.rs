// lib.rs — Tauri 入口；保持薄。
mod error;
mod events;
mod logging;
mod models;
mod commands;
mod db;
mod platform;

use parking_lot::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutEvent, ShortcutState};

use crate::db::DbState;

/// 跟踪「按下全局快捷键的瞬间」的目标窗口。
/// - pid: 跨平台都用得上（macOS 用它 NSRunningApplication 激活）
/// - hwnd: 仅 Windows 用，比 PID 更准（多窗口应用）
/// 注入时优先用 hwnd，再 fallback 到 pid。
#[derive(Default, Clone, Copy)]
pub struct FrontmostTarget {
    pub pid: Option<i32>,
    pub hwnd: Option<isize>,
}
pub struct LastFrontmost(pub Mutex<FrontmostTarget>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("app_data_dir");
            let log_dir = app_data.join("logs");
            let guard = logging::init(&log_dir);
            app.manage(LoggingGuard(guard));
            tracing::info!("Prompt Hub starting up");

            // 隐藏 dock 图标但允许激活（必须先于窗口设置）
            crate::platform::init_app_chrome();

            let db_path = app_data.join("prompt_hub.sqlite");
            let state = DbState::open(&db_path).expect("init db");
            app.manage(state);
            app.manage(LastFrontmost(Mutex::new(FrontmostTarget::default())));

            // 列出所有从 tauri.conf.json 预声明的窗口（用于确认 conf 是否生效）
            let labels: Vec<String> = app
                .webview_windows()
                .keys()
                .cloned()
                .collect();
            tracing::info!(?labels, "webview windows at startup");

            if let Some(drawer) = app.get_webview_window("drawer") {
                crate::platform::apply_panel_style(&drawer);
                #[cfg(debug_assertions)]
                {
                    drawer.open_devtools();
                }
            } else {
                tracing::warn!("drawer window not found at startup");
            }

            let settings = crate::db::settings::get_all(
                &app.state::<DbState>().0.lock(),
            )
            .ok();
            if let Some(s) = settings {
                if let Some(shortcut) = s.hotkey {
                    register_global_hotkey(app.handle(), &shortcut);
                }
                // 启动时把持久化的 auto_start 偏好同步到系统层
                crate::commands::settings::apply_autostart(app.handle(), s.auto_start);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            logging::log_record,
            logging::log_dir,
            commands::prompts::prompts_list,
            commands::prompts::prompts_get,
            commands::prompts::prompts_create,
            commands::prompts::prompts_update,
            commands::prompts::prompts_delete,
            commands::prompts::prompts_toggle_favorite,
            commands::prompts::prompts_toggle_pin,
            commands::prompts::prompts_record_use,
            commands::folders::folders_list,
            commands::folders::folders_create,
            commands::folders::folders_rename,
            commands::folders::folders_delete,
            commands::folders::folders_reorder,
            commands::tags::tags_list,
            commands::tags::tags_create,
            commands::tags::tags_rename,
            commands::tags::tags_delete,
            commands::settings::settings_get_all,
            commands::settings::settings_set,
            commands::data::data_export_json,
            commands::data::data_import_json,
            commands::sites::sites_list,
            commands::sites::sites_create,
            commands::sites::sites_update,
            commands::sites::sites_delete,
            commands::sites::sites_reorder,
            commands::sites::sites_refresh_favicon,
            commands::sites::sites_open,
            commands::inject::inject_paste,
            commands::inject::inject_copy_only,
            commands::inject::permissions_check_accessibility,
            commands::inject::permissions_request_accessibility,
            commands::window::window_show_drawer,
            commands::window::window_hide_drawer,
            commands::window::window_set_pin,
            commands::window::window_open_preview,
            commands::window::window_open_editor,
            commands::window::window_open_settings,
            register_hotkey_cmd,
            unregister_hotkey_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct LoggingGuard(#[allow(dead_code)] tracing_appender::non_blocking::WorkerGuard);

#[tauri::command]
fn register_hotkey_cmd(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    register_global_hotkey(&app, &shortcut);
    Ok(())
}

#[tauri::command]
fn unregister_hotkey_cmd(app: tauri::AppHandle) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();
    tracing::info!("global hotkeys unregistered");
    Ok(())
}

fn register_global_hotkey(app: &tauri::AppHandle, shortcut: &str) {
    let _ = app.global_shortcut().unregister_all();
    let app_clone = app.clone();
    let our_pid = std::process::id() as i32;
    let res = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _sc, ev: ShortcutEvent| {
            if ev.state() != ShortcutState::Pressed { return; }

            let drawer = match app_clone.get_webview_window("drawer") {
                Some(w) => w,
                None => return,
            };
            let visible = drawer.is_visible().unwrap_or(false);

            tracing::info!(visible, "hotkey pressed");

            if visible {
                let _ = drawer.hide();
                tracing::info!("hotkey: drawer hidden");
                return;
            }

            // 1) 记录当前 frontmost（必须在我们激活之前 — 那时候我们的
            //    PID 还不是 frontmost）。
            let pid_now = crate::platform::frontmost_app_pid();
            let hwnd_now = crate::platform::frontmost_window_handle();
            match pid_now {
                Some(pid) if pid != our_pid => {
                    if let Some(state) = app_clone.try_state::<LastFrontmost>() {
                        *state.0.lock() = FrontmostTarget {
                            pid: Some(pid),
                            hwnd: hwnd_now,
                        };
                        tracing::info!(
                            target_pid = pid,
                            target_hwnd = hwnd_now.unwrap_or(0),
                            "saved last frontmost target"
                        );
                    }
                }
                Some(pid) => {
                    tracing::warn!(pid, "frontmost is self; keeping previous saved target");
                }
                None => {
                    tracing::warn!("no frontmost app detected");
                }
            }

            // 2) 把我们自己 NSApp 激活成 frontmost
            //    —— 不激活的话 macOS 不会把键盘事件路由进我们的窗口
            crate::platform::activate_self();

            // 3) 显示窗口并让它成为 key window
            let _ = drawer.show();
            crate::platform::make_key(&drawer);
            tracing::info!("hotkey: activated self + drawer shown + made key");
        });
    match res {
        Ok(_) => tracing::info!(shortcut, "global hotkey registered"),
        Err(e) => tracing::error!(shortcut, error = %e, "register hotkey failed"),
    }
}
