// db/tags.rs
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::tag::Tag;

pub fn list(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut stmt = conn
        .prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE ASC")
        .map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Tag {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
            })
        })
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| AppError::Db(e.to_string()))?);
    }
    Ok(out)
}

pub fn create(conn: &Connection, name: &str, color: Option<&str>) -> AppResult<Tag> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("tag name is empty".into()));
    }
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(Tag {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        color: color.map(String::from),
    })
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> AppResult<()> {
    let n = conn
        .execute(
            "UPDATE tags SET name = ?1 WHERE id = ?2",
            params![name.trim(), id],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    let n = conn
        .execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}
