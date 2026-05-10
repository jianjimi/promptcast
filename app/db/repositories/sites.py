"""Site CRUD + favicon storage."""
from __future__ import annotations

from collections.abc import Iterable

from app.db.connection import get_conn, transaction
from app.models import Site, now_ms


def list_all() -> list[Site]:
    rows = get_conn().execute(
        "SELECT * FROM sites ORDER BY sort_order ASC, id ASC"
    ).fetchall()
    return [Site.from_row(r) for r in rows]


def get(site_id: int) -> Site | None:
    row = get_conn().execute("SELECT * FROM sites WHERE id = ?", (site_id,)).fetchone()
    return Site.from_row(row) if row else None


def create(name: str, url: str) -> int:
    with transaction() as conn:
        row = conn.execute("SELECT COALESCE(MAX(sort_order), -1) + 1 AS next FROM sites").fetchone()
        cur = conn.execute(
            "INSERT INTO sites(name, url, sort_order, created_at) VALUES (?, ?, ?, ?)",
            (name, url, row["next"], now_ms()),
        )
        return cur.lastrowid


def update(site_id: int, *, name: str | None = None, url: str | None = None) -> None:
    fields, params = [], []
    if name is not None:
        fields.append("name = ?")
        params.append(name)
    if url is not None:
        fields.append("url = ?")
        params.append(url)
    if not fields:
        return
    params.append(site_id)
    with transaction() as conn:
        conn.execute(f"UPDATE sites SET {', '.join(fields)} WHERE id = ?", params)


def delete(site_id: int) -> None:
    with transaction() as conn:
        conn.execute("DELETE FROM sites WHERE id = ?", (site_id,))


def reorder(ids_in_order: Iterable[int]) -> None:
    with transaction() as conn:
        for index, sid in enumerate(ids_in_order):
            conn.execute("UPDATE sites SET sort_order = ? WHERE id = ?", (index, sid))


def set_favicon(site_id: int, blob: bytes, mime: str) -> None:
    with transaction() as conn:
        conn.execute(
            "UPDATE sites SET favicon_blob = ?, favicon_mime = ?, favicon_fetched_at = ? WHERE id = ?",
            (blob, mime, now_ms(), site_id),
        )
