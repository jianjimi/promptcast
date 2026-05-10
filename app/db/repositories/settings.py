"""Key/value settings store."""
from __future__ import annotations

import json
from typing import Any

from app.db.connection import get_conn, transaction


def get_all() -> dict[str, Any]:
    rows = get_conn().execute("SELECT key, value FROM settings").fetchall()
    return {r["key"]: _decode(r["value"]) for r in rows}


def get(key: str, default: Any = None) -> Any:
    row = get_conn().execute("SELECT value FROM settings WHERE key = ?", (key,)).fetchone()
    return _decode(row["value"]) if row else default


def set_value(key: str, value: Any) -> None:
    encoded = json.dumps(value, ensure_ascii=False)
    with transaction() as conn:
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?, ?) "
            "ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, encoded),
        )


def delete(key: str) -> None:
    with transaction() as conn:
        conn.execute("DELETE FROM settings WHERE key = ?", (key,))


def _decode(raw: str) -> Any:
    try:
        return json.loads(raw)
    except (json.JSONDecodeError, TypeError):
        return raw


# Canonical keys with defaults
DEFAULTS: dict[str, Any] = {
    "hotkey": "ctrl+shift+space",
    "theme": "system",          # system | light | dark
    "default_action": "inject", # inject | copy_only
    "pin_default": False,
    "sort_mode": "recent_used",
    "auto_start": False,
    "accessibility_granted": False,
}
