"""JSON import/export for prompts/folders/tags/sites."""
from __future__ import annotations

import base64
import json
from pathlib import Path

from app.db.connection import transaction
from app.db.repositories import folders, prompts, sites, tags


def export_to_file(path: str | Path) -> None:
    payload = {
        "version": 1,
        "folders": [
            {"id": f.id, "name": f.name, "sort_order": f.sort_order, "created_at": f.created_at}
            for f in folders.list_all()
        ],
        "tags": [
            {"id": t.id, "name": t.name, "color": t.color}
            for t in tags.list_all()
        ],
        "prompts": [
            {
                "id": p.id, "title": p.title, "content": p.content,
                "folder_id": p.folder_id, "is_favorite": p.is_favorite,
                "is_pinned": p.is_pinned, "use_count": p.use_count,
                "last_used_at": p.last_used_at, "created_at": p.created_at,
                "updated_at": p.updated_at, "tag_ids": p.tag_ids,
            }
            for p in prompts.list_all()
        ],
        "sites": [
            {
                "id": s.id, "name": s.name, "url": s.url,
                "favicon_blob_b64": base64.b64encode(s.favicon_blob).decode("ascii") if s.favicon_blob else None,
                "favicon_mime": s.favicon_mime,
                "favicon_fetched_at": s.favicon_fetched_at,
                "sort_order": s.sort_order, "created_at": s.created_at,
            }
            for s in sites.list_all()
        ],
    }
    Path(path).write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")


def import_from_file(path: str | Path, *, replace: bool) -> None:
    raw = json.loads(Path(path).read_text(encoding="utf-8"))
    if replace:
        with transaction() as conn:
            conn.execute("DELETE FROM prompt_tags")
            conn.execute("DELETE FROM prompts")
            conn.execute("DELETE FROM tags")
            conn.execute("DELETE FROM folders")
            conn.execute("DELETE FROM sites")

    folder_id_map: dict[int, int] = {}
    for f in raw.get("folders", []):
        new_id = folders.create(f["name"])
        folder_id_map[f["id"]] = new_id

    tag_id_map: dict[int, int] = {}
    for t in raw.get("tags", []):
        new_id = tags.get_or_create(t["name"])
        tag_id_map[t["id"]] = new_id
        if t.get("color"):
            tags.set_color(new_id, t["color"])

    for p in raw.get("prompts", []):
        prompts.create(
            title=p["title"],
            content=p.get("content", ""),
            folder_id=folder_id_map.get(p.get("folder_id")) if p.get("folder_id") else None,
            tag_ids=[tag_id_map[t] for t in p.get("tag_ids", []) if t in tag_id_map],
        )

    for s in raw.get("sites", []):
        sid = sites.create(s["name"], s["url"])
        blob_b64 = s.get("favicon_blob_b64")
        if blob_b64 and s.get("favicon_mime"):
            sites.set_favicon(sid, base64.b64decode(blob_b64), s["favicon_mime"])
