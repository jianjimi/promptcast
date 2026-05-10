"""Domain entities — plain dataclasses mapped from sqlite rows."""
from __future__ import annotations

import sqlite3
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Self


def now_ms() -> int:
    return int(time.time() * 1000)


class SortMode(str, Enum):
    RECENT_USED = "recent_used"
    CREATED = "created"
    UPDATED = "updated"
    TITLE = "title"


@dataclass(slots=True, kw_only=True)
class Folder:
    id: int
    name: str
    sort_order: int = 0
    created_at: int = field(default_factory=now_ms)

    @classmethod
    def from_row(cls, row: sqlite3.Row) -> Self:
        return cls(id=row["id"], name=row["name"], sort_order=row["sort_order"], created_at=row["created_at"])


@dataclass(slots=True, kw_only=True)
class Tag:
    id: int
    name: str
    color: str | None = None

    @classmethod
    def from_row(cls, row: sqlite3.Row) -> Self:
        return cls(id=row["id"], name=row["name"], color=row["color"])


@dataclass(slots=True, kw_only=True)
class Prompt:
    id: int
    title: str
    content: str
    folder_id: int | None = None
    is_favorite: bool = False
    is_pinned: bool = False
    use_count: int = 0
    last_used_at: int | None = None
    created_at: int = field(default_factory=now_ms)
    updated_at: int = field(default_factory=now_ms)
    tag_ids: list[int] = field(default_factory=list)

    @classmethod
    def from_row(cls, row: sqlite3.Row, tag_ids: list[int] | None = None) -> Self:
        return cls(
            id=row["id"],
            title=row["title"],
            content=row["content"],
            folder_id=row["folder_id"],
            is_favorite=bool(row["is_favorite"]),
            is_pinned=bool(row["is_pinned"]),
            use_count=row["use_count"],
            last_used_at=row["last_used_at"],
            created_at=row["created_at"],
            updated_at=row["updated_at"],
            tag_ids=tag_ids or [],
        )


@dataclass(slots=True, kw_only=True)
class Site:
    id: int
    name: str
    url: str
    favicon_blob: bytes | None = None
    favicon_mime: str | None = None
    favicon_fetched_at: int | None = None
    sort_order: int = 0
    created_at: int = field(default_factory=now_ms)

    @classmethod
    def from_row(cls, row: sqlite3.Row) -> Self:
        return cls(
            id=row["id"],
            name=row["name"],
            url=row["url"],
            favicon_blob=row["favicon_blob"],
            favicon_mime=row["favicon_mime"],
            favicon_fetched_at=row["favicon_fetched_at"],
            sort_order=row["sort_order"],
            created_at=row["created_at"],
        )
