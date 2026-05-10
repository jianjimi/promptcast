"""Folder CRUD + reorder."""
from __future__ import annotations

from collections.abc import Iterable

from app.db.connection import get_conn, transaction
from app.models import Folder, now_ms


def list_all() -> list[Folder]:
    rows = get_conn().execute(
        "SELECT * FROM folders ORDER BY sort_order ASC, id ASC"
    ).fetchall()
    return [Folder.from_row(r) for r in rows]


def create(name: str) -> int:
    with transaction() as conn:
        row = conn.execute("SELECT COALESCE(MAX(sort_order), -1) + 1 AS next FROM folders").fetchone()
        cur = conn.execute(
            "INSERT INTO folders(name, sort_order, created_at) VALUES (?, ?, ?)",
            (name, row["next"], now_ms()),
        )
        return cur.lastrowid


def rename(folder_id: int, name: str) -> None:
    with transaction() as conn:
        conn.execute("UPDATE folders SET name = ? WHERE id = ?", (name, folder_id))


def delete(folder_id: int) -> None:
    with transaction() as conn:
        conn.execute("DELETE FROM folders WHERE id = ?", (folder_id,))


def reorder(ids_in_order: Iterable[int]) -> None:
    with transaction() as conn:
        for index, fid in enumerate(ids_in_order):
            conn.execute("UPDATE folders SET sort_order = ? WHERE id = ?", (index, fid))
