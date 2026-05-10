"""Centralized logging configuration."""
from __future__ import annotations

import logging
from logging.handlers import RotatingFileHandler

from app.config import LOG_DIR, ensure_dirs

_FMT = "%(asctime)s | %(levelname)-7s | %(name)s | %(message)s"


def setup_logging(level: int = logging.INFO) -> None:
    ensure_dirs()
    root = logging.getLogger()
    if root.handlers:
        return
    root.setLevel(level)

    file_handler = RotatingFileHandler(
        LOG_DIR / "app.log",
        maxBytes=2_000_000,
        backupCount=5,
        encoding="utf-8",
    )
    file_handler.setFormatter(logging.Formatter(_FMT))
    root.addHandler(file_handler)

    stream = logging.StreamHandler()
    stream.setFormatter(logging.Formatter(_FMT))
    root.addHandler(stream)
