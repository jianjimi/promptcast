// sync/engine.rs — 一次同步周期：push dirty → pull 增量 → apply（LWW）→ 推进游标 → emit 事件。
use std::collections::HashSet;

use tauri::{AppHandle, Manager};

use super::client::{Client, SyncError};
use super::{clear_refresh, load_refresh, server_url, store_refresh, SyncRuntime};
use crate::db::{sync_repo, sync_state, DbState};
use crate::error::{AppError, AppResult};
use crate::events;
use crate::models::sync::{ENTITY_FOLDER, ENTITY_PROMPT, ENTITY_SITE, ENTITY_TAG};

const PULL_LIMIT: i64 = 500;
const MAX_PAGES: usize = 50;

#[derive(Debug, Default)]
pub struct Outcome {
    pub pushed: usize,
    pub pulled: usize,
}

/// 跑一次完整同步。未登录 / 未开启 → 直接返回（Outcome 全 0）。
/// 网络/401 等错误向上抛；调用方（sync_loop）记录并下拍重试。
pub fn sync_once(app: &AppHandle) -> AppResult<Outcome> {
    let db = app.state::<DbState>();
    let rt = app.state::<SyncRuntime>();

    let (enabled, user_id, mut cursor, base) = {
        let conn = db.0.lock();
        let s = sync_state::get(&conn)?;
        let base = server_url(&conn);
        (s.sync_enabled, s.user_id, s.last_pull_cursor, base)
    };
    if !enabled || user_id.is_none() {
        return Ok(Outcome::default());
    }

    let client = Client::new(&base);
    let mut access = match rt.access.lock().clone() {
        Some(a) => a,
        None => refresh_access(&client, &rt)?,
    };

    // ---- PUSH ----
    let dirty = {
        let conn = db.0.lock();
        sync_repo::collect_dirty(&conn)?
    };
    let mut pushed = 0usize;
    if !dirty.is_empty() {
        let resp = match client.push(&access, &dirty) {
            Ok(r) => r,
            Err(SyncError::Unauthorized) => {
                access = refresh_access(&client, &rt)?;
                client.push(&access, &dirty).map_err(to_apperr)?
            }
            Err(e) => return Err(to_apperr(e)),
        };
        let conn = db.0.lock();
        for item in &resp.results {
            if item.applied {
                if let Some(ch) = dirty.iter().find(|c| c.uuid == item.uuid) {
                    sync_repo::clear_dirty(&conn, &ch.entity, &item.uuid)?;
                    pushed += 1;
                }
            }
        }
    }

    // ---- PULL（翻页）----
    let mut pulled = 0usize;
    let mut changed: HashSet<String> = HashSet::new();
    for _ in 0..MAX_PAGES {
        let resp = match client.pull(&access, cursor, PULL_LIMIT) {
            Ok(r) => r,
            Err(SyncError::Unauthorized) => {
                access = refresh_access(&client, &rt)?;
                client
                    .pull(&access, cursor, PULL_LIMIT)
                    .map_err(to_apperr)?
            }
            Err(e) => return Err(to_apperr(e)),
        };
        if resp.changes.is_empty() {
            break;
        }
        // 排序：folders/tags 先于 prompts（让 prompt 的 folder_uuid/tag_uuids 解析），sites 末。
        let mut batch = resp.changes;
        batch.sort_by_key(|c| entity_order(&c.entity));
        {
            let mut conn = db.0.lock();
            for ch in &batch {
                if sync_repo::apply_change(&mut conn, ch)? {
                    changed.insert(ch.entity.clone());
                    pulled += 1;
                }
            }
            sync_state::set_cursor(&conn, resp.next_cursor)?;
        }
        cursor = resp.next_cursor;
        if !resp.has_more {
            break;
        }
    }

    {
        let conn = db.0.lock();
        sync_repo::touch_synced(&conn)?;
    }

    if changed.contains(ENTITY_FOLDER) {
        events::emit_folders_changed(app);
    }
    if changed.contains(ENTITY_TAG) {
        events::emit_tags_changed(app);
    }
    if changed.contains(ENTITY_PROMPT) {
        events::emit_prompts_changed(app);
    }
    if changed.contains(ENTITY_SITE) {
        events::emit_sites_changed(app);
    }

    Ok(Outcome { pushed, pulled })
}

fn entity_order(entity: &str) -> u8 {
    match entity {
        ENTITY_FOLDER => 0,
        ENTITY_TAG => 1,
        ENTITY_PROMPT => 2,
        ENTITY_SITE => 3,
        _ => 4,
    }
}

/// 用 keyring 里的 refresh token 换新的 access（并轮换 refresh）。失败视为登出。
fn refresh_access(client: &Client, rt: &SyncRuntime) -> AppResult<String> {
    let refresh = load_refresh().ok_or_else(|| AppError::Internal("not logged in".into()))?;
    match client.refresh(&refresh) {
        Ok(t) => {
            store_refresh(&t.refresh);
            *rt.access.lock() = Some(t.access.clone());
            Ok(t.access)
        }
        Err(e) => {
            clear_refresh();
            *rt.access.lock() = None;
            Err(AppError::Internal(format!("refresh failed: {e}")))
        }
    }
}

fn to_apperr(e: SyncError) -> AppError {
    AppError::Internal(format!("sync: {e}"))
}
