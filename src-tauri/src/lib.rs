// lib.rs — Tauri 入口；保持薄。
mod error;
mod events;
mod logging;
mod models;
mod commands;
mod db;
mod platform;

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutEvent, ShortcutState};

use crate::db::DbState;

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
            // 1) 日志（需先于其他模块以便 init 期间也能记录）
            let app_data = app
                .path()
                .app_data_dir()
                .expect("app_data_dir");
            let log_dir = app_data.join("logs");
            let guard = logging::init(&log_dir);
            // 把 guard 放进 state，进程结束才被 drop
            app.manage(LoggingGuard(guard));
            tracing::info!("Prompt Hub starting up");

            // 2) 数据库
            let db_path = app_data.join("prompt_hub.sqlite");
            let state = DbState::open(&db_path).expect("init db");
            app.manage(state);

            // 3) 抽屉窗口加平台特性（不抢焦点）
            if let Some(drawer) = app.get_webview_window("drawer") {
                crate::platform::apply_panel_style(&drawer);
            } else {
                tracing::warn!("drawer window not found at startup");
            }

            // 4) 注册全局快捷键（若用户已设置）
            let settings = crate::db::settings::get_all(
                &app.state::<DbState>().0.lock(),
            )
            .ok();
            if let Some(s) = settings {
                if let Some(shortcut) = s.hotkey {
                    register_global_hotkey(app.handle(), &shortcut);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            // logging
            logging::log_record,
            logging::log_dir,
            // prompts
            commands::prompts::prompts_list,
            commands::prompts::prompts_get,
            commands::prompts::prompts_create,
            commands::prompts::prompts_update,
            commands::prompts::prompts_delete,
            commands::prompts::prompts_toggle_favorite,
            commands::prompts::prompts_toggle_pin,
            commands::prompts::prompts_record_use,
            // folders / tags / settings / data
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
            // sites
            commands::sites::sites_list,
            commands::sites::sites_create,
            commands::sites::sites_update,
            commands::sites::sites_delete,
            commands::sites::sites_reorder,
            commands::sites::sites_refresh_favicon,
            commands::sites::sites_open,
            // inject + window + perms
            commands::inject::inject_paste,
            commands::inject::inject_copy_only,
            commands::inject::permissions_check_accessibility,
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

/// 持有日志 guard 直到进程结束。
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
    let res = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _sc, ev: ShortcutEvent| {
            if ev.state() == ShortcutState::Pressed {
                if let Some(w) = app_clone.get_webview_window("drawer") {
                    let visible = w.is_visible().unwrap_or(false);
                    if visible {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        // NSPanel 不会抢焦点；这里不调 set_focus 是有意的
                        // (set_focus 会激活应用，破坏注入流程)
                    }
                }
            }
        });
    match res {
        Ok(_) => tracing::info!(shortcut, "global hotkey registered"),
        Err(e) => tracing::error!(shortcut, error = %e, "register hotkey failed"),
    }
}
