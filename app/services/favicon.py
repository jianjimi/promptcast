"""Async favicon fetcher — runs in a Qt thread pool, writes to sites repo."""
from __future__ import annotations

import logging
from typing import Callable
from urllib.parse import urlparse

from PyQt6.QtCore import QObject, QRunnable, QThreadPool, pyqtSignal

from app.db.repositories import sites as sites_repo

log = logging.getLogger(__name__)


class _Signals(QObject):
    done = pyqtSignal(int, bool)  # site_id, ok


class _FetchTask(QRunnable):
    def __init__(self, site_id: int, url: str) -> None:
        super().__init__()
        self.site_id = site_id
        self.url = url
        self.signals = _Signals()

    def run(self) -> None:
        try:
            blob, mime = _fetch_blob(self.url)
            sites_repo.set_favicon(self.site_id, blob, mime)
            self.signals.done.emit(self.site_id, True)
        except Exception as exc:
            log.warning("favicon fetch failed for %s: %s", self.url, exc)
            self.signals.done.emit(self.site_id, False)


def _fetch_blob(url: str) -> tuple[bytes, str]:
    import requests

    parsed = urlparse(url)
    if not parsed.scheme:
        url = "https://" + url
        parsed = urlparse(url)
    candidates = [
        f"{parsed.scheme}://{parsed.netloc}/favicon.ico",
        f"https://www.google.com/s2/favicons?sz=64&domain={parsed.netloc}",
    ]
    last_err: Exception | None = None
    for candidate in candidates:
        try:
            r = requests.get(candidate, timeout=6)
            if r.status_code == 200 and r.content:
                mime = r.headers.get("content-type", "image/x-icon").split(";")[0].strip()
                return r.content, mime
        except Exception as e:
            last_err = e
    raise RuntimeError(f"no favicon fetched ({last_err})")


def fetch_async(site_id: int, url: str, *, on_done: Callable[[], None] | None = None) -> None:
    task = _FetchTask(site_id, url)
    if on_done is not None:
        task.signals.done.connect(lambda _sid, _ok: on_done())
    QThreadPool.globalInstance().start(task)
