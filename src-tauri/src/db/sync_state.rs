// db/sync_state.rs — 同步元数据单例（device_id、拉取游标、登录用户、开关）。
// 由 V3 迁移插入单例行（id=1）。同步引擎（Phase 2）读写游标/用户/开关。
// 这些访问器在 Phase 2 同步引擎接入前暂未被调用。
#![allow(dead_code)]
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct SyncStateRow {
    pub device_id: String,
    pub last_pull_cursor: i64,
    pub last_sync_at: Option<i64>,
    pub user_id: Option<String>,
    pub sync_enabled: bool,
}

pub fn get(conn: &Connection) -> AppResult<SyncStateRow> {
    conn.query_row(
        "SELECT device_id, last_pull_cursor, last_sync_at, user_id, sync_enabled \
         FROM sync_state WHERE id = 1",
        [],
        |r| {
            Ok(SyncStateRow {
                device_id: r.get(0)?,
                last_pull_cursor: r.get(1)?,
                last_sync_at: r.get(2)?,
                user_id: r.get(3)?,
                sync_enabled: r.get::<_, i64>(4)? != 0,
            })
        },
    )
    .map_err(|e| AppError::Db(e.to_string()))
}

pub fn set_cursor(conn: &Connection, cursor: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET last_pull_cursor = ?1 WHERE id = 1",
        params![cursor],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

pub fn set_user(conn: &Connection, user_id: Option<&str>) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET user_id = ?1 WHERE id = 1",
        params![user_id],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

pub fn set_enabled(conn: &Connection, enabled: bool) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET sync_enabled = ?1 WHERE id = 1",
        params![enabled as i64],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

pub fn touch_last_sync(conn: &Connection, at: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE sync_state SET last_sync_at = ?1 WHERE id = 1",
        params![at],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}
