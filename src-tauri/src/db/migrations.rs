// db/migrations.rs — 基于 PRAGMA user_version 的简易版本化迁移。
use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use super::schema;

const CURRENT_VERSION: i64 = 2;

pub fn migrate(conn: &Connection) -> AppResult<()> {
    let v: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| AppError::Db(e.to_string()))?;

    if v < 1 {
        conn.execute_batch(schema::V1)
            .map_err(|e| AppError::Db(format!("apply v1: {e}")))?;
    }

    if v < 2 {
        conn.execute_batch(schema::V2)
            .map_err(|e| AppError::Db(format!("apply v2: {e}")))?;
    }

    if v != CURRENT_VERSION {
        conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_VERSION};"))
            .map_err(|e| AppError::Db(e.to_string()))?;
    }
    Ok(())
}
