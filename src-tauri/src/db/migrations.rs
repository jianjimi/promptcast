// db/migrations.rs — 基于 PRAGMA user_version 的简易版本化迁移。
use rusqlite::Connection;

use super::schema;
use crate::error::{AppError, AppResult};

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // 二次运行不应报错，且版本停在 CURRENT_VERSION。
        migrate(&conn).unwrap();
        let v: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, CURRENT_VERSION);
        // 关键表都已建立。
        for t in [
            "prompts",
            "folders",
            "tags",
            "prompt_tags",
            "settings",
            "sites",
            "clipboard_history",
        ] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [t],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "table {t} should exist");
        }
    }
}
