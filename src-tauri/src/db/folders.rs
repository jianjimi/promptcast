// db/folders.rs
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::folder::Folder;
use super::now_ms;

pub fn list(conn: &Connection) -> AppResult<Vec<Folder>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, sort_order, created_at FROM folders \
             ORDER BY sort_order ASC, id ASC",
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Folder {
                id: r.get(0)?,
                name: r.get(1)?,
                sort_order: r.get(2)?,
                created_at: r.get(3)?,
            })
        })
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| AppError::Db(e.to_string()))?);
    }
    Ok(out)
}

pub fn create(conn: &Connection, name: &str) -> AppResult<Folder> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("folder name is empty".into()));
    }
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM folders",
            [],
            |r| r.get(0),
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    let now = now_ms();
    conn.execute(
        "INSERT INTO folders (name, sort_order, created_at) VALUES (?1, ?2, ?3)",
        params![name, max_order + 1, now],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    let id = conn.last_insert_rowid();
    Ok(Folder {
        id,
        name: name.to_string(),
        sort_order: max_order + 1,
        created_at: now,
    })
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("folder name is empty".into()));
    }
    let n = conn
        .execute("UPDATE folders SET name = ?1 WHERE id = ?2", params![name, id])
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("folder {id}")));
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    let n = conn
        .execute("DELETE FROM folders WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("folder {id}")));
    }
    Ok(())
}

pub fn reorder(conn: &mut Connection, ordered_ids: &[i64]) -> AppResult<()> {
    let tx = conn.transaction().map_err(|e| AppError::Db(e.to_string()))?;
    for (i, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE folders SET sort_order = ?1 WHERE id = ?2",
            params![i as i64, id],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    }
    tx.commit().map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}
