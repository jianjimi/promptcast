"""Prompt CRUD + ordering."""
from __future__ import annotations

from collections import defaultdict
from collections.abc import Iterable

from app.db.connection import get_conn, transaction
from app.models import Prompt, SortMode, now_ms

_ORDER_SQL: dict[SortMode, str] = {
    SortMode.RECENT_USED: "is_pinned DESC, COALESCE(last_used_at, 0) DESC, updated_at DESC",
    SortMode.CREATED: "is_pinned DESC, created_at DESC",
    SortMode.UPDATED: "is_pinned DESC, updated_at DESC",
    SortMode.TITLE: "is_pinned DESC, title COLLATE NOCASE ASC",
}


def _load_tag_map(prompt_ids: Iterable[int]) -> dict[int, list[int]]:
    ids = list(prompt_ids)
    if not ids:
        return {}
    placeholders = ",".join("?" * len(ids))
    rows = get_conn().execute(
        f"SELECT prompt_id, tag_id FROM prompt_tags WHERE prompt_id IN ({placeholders})",
        ids,
    ).fetchall()
    out: dict[int, list[int]] = defaultdict(list)
    for r in rows:
        out[r["prompt_id"]].append(r["tag_id"])
    return out


def list_all(sort_mode: SortMode = SortMode.RECENT_USED) -> list[Prompt]:
    order = _ORDER_SQL[sort_mode]
    rows = get_conn().execute(f"SELECT * FROM prompts ORDER BY {order}").fetchall()
    tags = _load_tag_map(r["id"] for r in rows)
    return [Prompt.from_row(r, tags.get(r["id"], [])) for r in rows]


def get(prompt_id: int) -> Prompt | None:
    row = get_conn().execute("SELECT * FROM prompts WHERE id = ?", (prompt_id,)).fetchone()
    if row is None:
        return None
    tags = _load_tag_map([prompt_id]).get(prompt_id, [])
    return Prompt.from_row(row, tags)


def create(*, title: str, content: str, folder_id: int | None = None, tag_ids: Iterable[int] = ()) -> int:
    ts = now_ms()
    with transaction() as conn:
        cur = conn.execute(
            "INSERT INTO prompts(title, content, folder_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            (title, content, folder_id, ts, ts),
        )
        prompt_id = cur.lastrowid
        _replace_tags(conn, prompt_id, tag_ids)
        return prompt_id


def update(
    prompt_id: int,
    *,
    title: str | None = None,
    content: str | None = None,
    folder_id: int | None = None,
    tag_ids: Iterable[int] | None = None,
    clear_folder: bool = False,
) -> None:
    fields: list[str] = []
    params: list[object] = []
    if title is not None:
        fields.append("title = ?")
        params.append(title)
    if content is not None:
        fields.append("content = ?")
        params.append(content)
    if folder_id is not None or clear_folder:
        fields.append("folder_id = ?")
        params.append(None if clear_folder else folder_id)
    fields.append("updated_at = ?")
    params.append(now_ms())
    params.append(prompt_id)

    with transaction() as conn:
        if fields:
            conn.execute(f"UPDATE prompts SET {', '.join(fields)} WHERE id = ?", params)
        if tag_ids is not None:
            _replace_tags(conn, prompt_id, tag_ids)


def delete(prompt_id: int) -> None:
    with transaction() as conn:
        conn.execute("DELETE FROM prompts WHERE id = ?", (prompt_id,))


def toggle_favorite(prompt_id: int) -> bool:
    with transaction() as conn:
        conn.execute("UPDATE prompts SET is_favorite = 1 - is_favorite WHERE id = ?", (prompt_id,))
        row = conn.execute("SELECT is_favorite FROM prompts WHERE id = ?", (prompt_id,)).fetchone()
        return bool(row["is_favorite"]) if row else False


def toggle_pin(prompt_id: int) -> bool:
    with transaction() as conn:
        conn.execute("UPDATE prompts SET is_pinned = 1 - is_pinned WHERE id = ?", (prompt_id,))
        row = conn.execute("SELECT is_pinned FROM prompts WHERE id = ?", (prompt_id,)).fetchone()
        return bool(row["is_pinned"]) if row else False


def record_use(prompt_id: int) -> None:
    ts = now_ms()
    with transaction() as conn:
        conn.execute(
            "UPDATE prompts SET use_count = use_count + 1, last_used_at = ? WHERE id = ?",
            (ts, prompt_id),
        )


def _replace_tags(conn, prompt_id: int, tag_ids: Iterable[int]) -> None:
    conn.execute("DELETE FROM prompt_tags WHERE prompt_id = ?", (prompt_id,))
    rows = [(prompt_id, tid) for tid in tag_ids]
    if rows:
        conn.executemany("INSERT INTO prompt_tags(prompt_id, tag_id) VALUES (?, ?)", rows)
