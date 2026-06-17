// commands/auth.rs — 账户（同步用）：注册 / 登录 / 登出 / 状态。
// access token 存内存(SyncRuntime)；refresh token 存系统钥匙串；email 存 settings 原始 key
// 便于重启后 UI 显示。登录成功即开启同步并触发一拍。
use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::db::{settings, sync_state, DbState};
use crate::error::{AppError, AppResult};
use crate::sync::{self, client::Client, SyncRuntime};

const USER_EMAIL_KEY: &str = "sync_user_email";

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthStatus {
    pub logged_in: bool,
    pub email: Option<String>,
}

fn client_for(db: &DbState) -> Client {
    let conn = db.0.lock();
    Client::new(&sync::server_url(&conn))
}

fn on_authed(
    app: &AppHandle,
    db: &DbState,
    rt: &SyncRuntime,
    user_id: &str,
    email: &str,
    access: String,
    refresh: String,
) -> AppResult<()> {
    sync::store_refresh(&refresh);
    *rt.access.lock() = Some(access);
    *rt.email.lock() = Some(email.to_string());
    {
        let conn = db.0.lock();
        sync_state::set_user(&conn, Some(user_id))?;
        sync_state::set_enabled(&conn, true)?;
        settings::set_raw(&conn, USER_EMAIL_KEY, email)?;
    }
    rt.wake.store(true, Ordering::Relaxed);
    sync::emit_status(app, "idle", None);
    Ok(())
}

#[tauri::command]
pub fn auth_register(
    app: AppHandle,
    db: State<'_, DbState>,
    rt: State<'_, SyncRuntime>,
    email: String,
    password: String,
) -> AppResult<AuthStatus> {
    let r = client_for(&db)
        .register(&email, &password)
        .map_err(|e| AppError::Internal(format!("注册失败: {e}")))?;
    on_authed(&app, &db, &rt, &r.user_id, &email, r.access, r.refresh)?;
    tracing::info!("auth: registered + logged in");
    Ok(AuthStatus {
        logged_in: true,
        email: Some(email),
    })
}

#[tauri::command]
pub fn auth_login(
    app: AppHandle,
    db: State<'_, DbState>,
    rt: State<'_, SyncRuntime>,
    email: String,
    password: String,
) -> AppResult<AuthStatus> {
    let r = client_for(&db)
        .login(&email, &password)
        .map_err(|e| AppError::Internal(format!("登录失败: {e}")))?;
    on_authed(&app, &db, &rt, &r.user_id, &email, r.access, r.refresh)?;
    tracing::info!("auth: logged in");
    Ok(AuthStatus {
        logged_in: true,
        email: Some(email),
    })
}

#[tauri::command]
pub fn auth_logout(
    app: AppHandle,
    db: State<'_, DbState>,
    rt: State<'_, SyncRuntime>,
) -> AppResult<()> {
    // best-effort 撤销服务端 refresh，再清本地会话。
    if let Some(refresh) = sync::load_refresh() {
        let _ = client_for(&db).logout(&refresh);
    }
    sync::clear_refresh();
    *rt.access.lock() = None;
    *rt.email.lock() = None;
    {
        let conn = db.0.lock();
        let _ = sync_state::set_user(&conn, None);
        let _ = sync_state::set_enabled(&conn, false);
        let _ = settings::set_raw(&conn, USER_EMAIL_KEY, "");
    }
    sync::emit_status(&app, "logged_out", None);
    tracing::info!("auth: logged out");
    Ok(())
}

#[tauri::command]
pub fn auth_status(rt: State<'_, SyncRuntime>) -> AppResult<AuthStatus> {
    let email = rt.email.lock().clone();
    let logged_in = email.is_some() || sync::load_refresh().is_some();
    Ok(AuthStatus { logged_in, email })
}
