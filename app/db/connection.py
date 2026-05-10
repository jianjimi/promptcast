"""SQLite connection helpers."""
from __future__ import annotations

import sqlite3
import threading
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

from app.config import DB_PATH, ensure_dirs
from app.db.migrations import migrate

_LOCK = threading.Lock()
_CONN: sqlite3.Connection | None = None


def _open(path: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(path, detect_types=sqlite3.PARSE_DECLTYPES, check_same_thread=False)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    return conn


def get_conn() -> sqlite3.Connection:
    global _CONN
    with _LOCK:
        if _CONN is None:
            ensure_dirs()
            _CONN = _open(DB_PATH)
            migrate(_CONN)
        return _CONN


@contextmanager
def transaction() -> Iterator[sqlite3.Connection]:
    """Atomic write block — commits on success, rolls back on error."""
    conn = get_conn()
    with _LOCK:
        try:
            yield conn
            conn.commit()
        except Exception:
            conn.rollback()
            raise


def reset_for_tests(path: Path | None = None) -> sqlite3.Connection:
    """Test helper — open a fresh DB at `path` (or in-memory)."""
    global _CONN
    with _LOCK:
        if _CONN is not None:
            _CONN.close()
        _CONN = _open(path or Path(":memory:"))
        migrate(_CONN)
        return _CONN
