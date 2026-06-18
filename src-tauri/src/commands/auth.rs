// commands/auth.rs — 账户（同步用）：注册 / 登录 / 登出 / 状态。
// access token 存内存(SyncRuntime)；refresh token 存系统钥匙串；email 存 settings 原始 key
// 便于重启后 UI 显示。登录成功即开启同步并触发一拍。
use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::db::{settings, sync_repo, sync_state, DbState};
use crate::error::{AppError, AppResult};
use crate::sync::{
    self,
    client::{Client, SyncError},
    SyncRuntime,
};

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
        // 账户切换：清掉上一个账户的本地可同步数据 + 重置游标，新账户从头拉自己的，
        // 避免上一个账户的本地行泄漏给新账户（review #5）。首登/同账户不动。
        let prev = sync_state::get(&conn).ok().and_then(|s| s.user_id);
        if prev.as_deref().is_some_and(|p| p != user_id) {
            sync_repo::wipe_syncable(&conn)?;
            sync_state::set_cursor(&conn, 0)?;
            tracing::info!("account switch: wiped local syncable data + reset pull cursor");
        }
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
        // 停同步、清会话；但保留 sync_state.user_id 作为「上一个账户」标记，供下次登录
        // 判定是否换号（换号才 wipe 本地数据，见 on_authed）。本地数据照常离线可用。
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

fn map_sync_err(e: SyncError) -> AppError {
    AppError::Internal(e.to_string())
}

/// 用 keyring 的 refresh 换新的 access（写回内存）。失败=会话过期。
fn refresh_access(client: &Client, rt: &SyncRuntime) -> AppResult<String> {
    let refresh = sync::load_refresh().ok_or_else(|| AppError::Internal("未登录".into()))?;
    let t = client
        .refresh(&refresh)
        .map_err(|e| AppError::Internal(format!("会话过期，请重新登录: {e}")))?;
    sync::store_refresh(&t.refresh);
    *rt.access.lock() = Some(t.access.clone());
    Ok(t.access)
}

/// 取一个内存 access，没有就用 keyring refresh 换。
fn access_or_refresh(client: &Client, rt: &SyncRuntime) -> AppResult<String> {
    // 先把锁释放再进 None 分支 —— refresh_access 会再次锁 rt.access；parking_lot 不可重入，
    // 若把 lock() 放在 match 头部，guard 会活到整个 match 结束 → 自死锁（重启后首次必现）。
    let cached = rt.access.lock().clone();
    match cached {
        Some(a) => Ok(a),
        None => refresh_access(client, rt),
    }
}

#[tauri::command]
pub fn auth_change_password(
    app: AppHandle,
    db: State<'_, DbState>,
    rt: State<'_, SyncRuntime>,
    old_password: String,
    new_password: String,
) -> AppResult<()> {
    let client = client_for(&db);
    let access = access_or_refresh(&client, &rt)?;
    // 401 可能是 token 过期、也可能是旧密码不对；过期就刷新重试一次，旧密码错则二次也会 401 报错。
    match client.change_password(&access, &old_password, &new_password) {
        Ok(()) => {}
        Err(SyncError::Unauthorized) => {
            let a = refresh_access(&client, &rt)?;
            client
                .change_password(&a, &old_password, &new_password)
                .map_err(map_sync_err)?;
        }
        Err(e) => return Err(map_sync_err(e)),
    }
    // 改密后服务端吊销了所有 refresh（含本机）→ 用新密码重登拿新令牌，保持本机在线。
    let email = rt.email.lock().clone().or_else(|| {
        let conn = db.0.lock();
        crate::db::settings::get_raw(&conn, USER_EMAIL_KEY)
            .ok()
            .flatten()
            .filter(|e| !e.is_empty())
    });
    let relogged = match email {
        Some(email) => match client.login(&email, &new_password) {
            Ok(r) => {
                sync::store_refresh(&r.refresh);
                *rt.access.lock() = Some(r.access);
                true
            }
            Err(_) => false,
        },
        None => false,
    };
    if !relogged {
        // keyring 里的 refresh 已被服务端吊销 → 主动清会话并提示重登，避免下次刷新时静默掉线。
        sync::clear_refresh();
        *rt.access.lock() = None;
        sync::emit_status(
            &app,
            "logged_out",
            Some("密码已修改，请用新密码重新登录".into()),
        );
        tracing::warn!("auth: password changed but re-login failed; session cleared");
    } else {
        tracing::info!("auth: password changed");
    }
    Ok(())
}

#[tauri::command]
pub fn auth_delete_account(
    app: AppHandle,
    db: State<'_, DbState>,
    rt: State<'_, SyncRuntime>,
    password: String,
) -> AppResult<()> {
    let client = client_for(&db);
    let access = access_or_refresh(&client, &rt)?;
    match client.delete_account(&access, &password) {
        Ok(()) => {}
        Err(SyncError::Unauthorized) => {
            let a = refresh_access(&client, &rt)?;
            client.delete_account(&a, &password).map_err(map_sync_err)?;
        }
        Err(e) => return Err(map_sync_err(e)),
    }
    // 账户已删 → 清会话 + 清本地可同步数据（这些是已删账户的数据）+ 复位游标/用户。
    sync::clear_refresh();
    *rt.access.lock() = None;
    *rt.email.lock() = None;
    {
        let conn = db.0.lock();
        let _ = sync_repo::wipe_syncable(&conn);
        let _ = sync_state::set_cursor(&conn, 0);
        let _ = sync_state::set_user(&conn, None);
        let _ = sync_state::set_enabled(&conn, false);
        let _ = settings::set_raw(&conn, USER_EMAIL_KEY, "");
    }
    crate::events::emit_prompts_changed(&app);
    crate::events::emit_folders_changed(&app);
    crate::events::emit_tags_changed(&app);
    crate::events::emit_sites_changed(&app);
    sync::emit_status(&app, "logged_out", None);
    tracing::info!("auth: account deleted + local wiped");
    Ok(())
}

/// 找回密码：返回 dev token（本地开发回显，便于测试）；生产为空。
#[tauri::command]
pub fn auth_forgot_password(db: State<'_, DbState>, email: String) -> AppResult<Option<String>> {
    let r = client_for(&db)
        .forgot_password(&email)
        .map_err(map_sync_err)?;
    Ok(r.dev_token)
}

#[tauri::command]
pub fn auth_reset_password(
    db: State<'_, DbState>,
    token: String,
    new_password: String,
) -> AppResult<()> {
    client_for(&db)
        .reset_password(&token, &new_password)
        .map_err(map_sync_err)
}
