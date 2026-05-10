"""Schema migrations driven by PRAGMA user_version."""
from __future__ import annotations

import sqlite3

CURRENT_VERSION = 1

V1 = """
CREATE TABLE IF NOT EXISTS folders (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL UNIQUE,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
  id     INTEGER PRIMARY KEY AUTOINCREMENT,
  name   TEXT NOT NULL UNIQUE,
  color  TEXT
);

CREATE TABLE IF NOT EXISTS prompts (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT NOT NULL,
  content      TEXT NOT NULL,
  folder_id    INTEGER REFERENCES folders(id) ON DELETE SET NULL,
  is_favorite  INTEGER NOT NULL DEFAULT 0,
  is_pinned    INTEGER NOT NULL DEFAULT 0,
  use_count    INTEGER NOT NULL DEFAULT 0,
  last_used_at INTEGER,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_tags (
  prompt_id INTEGER NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  tag_id    INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
  PRIMARY KEY (prompt_id, tag_id)
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sites (
  id                 INTEGER PRIMARY KEY AUTOINCREMENT,
  name               TEXT NOT NULL,
  url                TEXT NOT NULL,
  favicon_blob       BLOB,
  favicon_mime       TEXT,
  favicon_fetched_at INTEGER,
  sort_order         INTEGER NOT NULL DEFAULT 0,
  created_at         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompts_folder   ON prompts(folder_id);
CREATE INDEX IF NOT EXISTS idx_prompts_lastused ON prompts(last_used_at);
CREATE INDEX IF NOT EXISTS idx_prompts_pinned   ON prompts(is_pinned, is_favorite);
"""


def migrate(conn: sqlite3.Connection) -> None:
    cur = conn.execute("PRAGMA user_version")
    version = cur.fetchone()[0]

    if version < 1:
        conn.executescript(V1)

    if version != CURRENT_VERSION:
        conn.execute(f"PRAGMA user_version = {CURRENT_VERSION}")
        conn.commit()
