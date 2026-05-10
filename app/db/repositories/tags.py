"""Tag CRUD."""
from __future__ import annotations

from app.db.connection import get_conn, transaction
from app.models import Tag


def list_all() -> list[Tag]:
    rows = get_conn().execute("SELECT * FROM tags ORDER BY name COLLATE NOCASE ASC").fetchall()
    return [Tag.from_row(r) for r in rows]


def create(name: str, color: str | None = None) -> int:
    with transaction() as conn:
        cur = conn.execute("INSERT INTO tags(name, color) VALUES (?, ?)", (name, color))
        return cur.lastrowid


def rename(tag_id: int, name: str) -> None:
    with transaction() as conn:
        conn.execute("UPDATE tags SET name = ? WHERE id = ?", (name, tag_id))


def set_color(tag_id: int, color: str | None) -> None:
    with transaction() as conn:
        conn.execute("UPDATE tags SET color = ? WHERE id = ?", (color, tag_id))


def delete(tag_id: int) -> None:
    with transaction() as conn:
        conn.execute("DELETE FROM tags WHERE id = ?", (tag_id,))


def get_or_create(name: str) -> int:
    with transaction() as conn:
        row = conn.execute("SELECT id FROM tags WHERE name = ?", (name,)).fetchone()
        if row:
            return row["id"]
        cur = conn.execute("INSERT INTO tags(name) VALUES (?)", (name,))
        return cur.lastrowid
