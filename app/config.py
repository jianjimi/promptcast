"""Filesystem paths and runtime constants."""
from __future__ import annotations

import os
import sys
from pathlib import Path

APP_NAME = "PromptCast"


def _data_root() -> Path:
    if sys.platform == "win32":
        base = os.environ.get("APPDATA") or str(Path.home() / "AppData" / "Roaming")
        return Path(base) / APP_NAME
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / APP_NAME
    return Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share")) / APP_NAME


DATA_DIR: Path = _data_root()
LOG_DIR: Path = DATA_DIR / "logs"
DB_PATH: Path = DATA_DIR / "promptcast.db"

ASSETS_DIR: Path = Path(__file__).resolve().parent.parent / "assets"
STYLES_DIR: Path = Path(__file__).resolve().parent / "ui" / "styles"


def ensure_dirs() -> None:
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
