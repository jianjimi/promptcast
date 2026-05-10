"""Inject a prompt's content into the previously focused application.

Pipeline (mirrors the Rust version):
  1. Backup current clipboard text.
  2. Write prompt content to clipboard.
  3. Check accessibility (macOS); fall back to copy-only when denied.
  4. Sleep ~100ms (drawer hide already triggered by caller).
  5. Activate the previously focused app via the platform layer.
  6. Sleep ~120ms.
  7. Synthesize Cmd/Ctrl+V.
  8. After 600ms restore the original clipboard if it's still ours.

All clipboard access happens on the GUI thread; activate + sleep + paste
runs in a worker thread so the UI stays responsive.
"""
from __future__ import annotations

import logging
import threading
import time

from PyQt6.QtCore import QObject, QTimer, pyqtSignal
from PyQt6.QtGui import QGuiApplication

from app.db.repositories import prompts as prompts_repo
from app.platform import ForegroundRef, get_platform

log = logging.getLogger(__name__)


def _set_clipboard(text: str) -> None:
    QGuiApplication.clipboard().setText(text)


def _read_clipboard() -> str:
    return QGuiApplication.clipboard().text()


class InjectService(QObject):
    """Singleton coordinator. Exposes a `message` signal for toasts."""

    message = pyqtSignal(str, str)        # text, level
    _restoreRequested = pyqtSignal(str, str)  # original, written

    def __init__(self) -> None:
        super().__init__()
        self._restoreRequested.connect(self._handle_restore)

    # ---- public API ----------------------------------------------------------------
    def copy_only(self, prompt_id: int) -> None:
        p = prompts_repo.get(prompt_id)
        if p is None:
            return
        _set_clipboard(p.content)
        prompts_repo.record_use(prompt_id)
        log.info("copy_only id=%s", prompt_id)
        self.message.emit("已复制到剪贴板", "success")

    def inject(self, prompt_id: int, foreground: ForegroundRef) -> None:
        p = prompts_repo.get(prompt_id)
        if p is None:
            return

        platform = get_platform()
        original = _read_clipboard()
        _set_clipboard(p.content)
        prompts_repo.record_use(prompt_id)

        if not platform.check_accessibility():
            self.message.emit("无辅助功能权限 · 已复制到剪贴板", "warning")
            return

        if not (foreground.pid or foreground.hwnd):
            self.message.emit("未捕获目标窗口 · 已复制到剪贴板", "warning")
            return

        written = p.content

        def _worker() -> None:
            try:
                time.sleep(0.10)
                ok = platform.activate(foreground)
                if not ok:
                    log.warning("activate failed; staying with copy-only")
                    self.message.emit("目标窗口激活失败 · 已复制", "warning")
                    self._restoreRequested.emit(original, written)
                    return
                time.sleep(0.12)
                platform.simulate_paste()
                log.info("inject paste sent for prompt id=%s", prompt_id)
                self._restoreRequested.emit(original, written)
            except Exception as exc:
                log.exception("inject worker failed: %s", exc)
                self.message.emit(f"注入失败: {exc}", "danger")

        threading.Thread(target=_worker, daemon=True).start()

    # ---- internals -----------------------------------------------------------------
    def _handle_restore(self, original: str, written: str) -> None:
        def _do_restore() -> None:
            try:
                current = _read_clipboard()
                if current == written and original:
                    _set_clipboard(original)
                    log.info("clipboard restored")
            except Exception as exc:
                log.warning("clipboard restore failed: %s", exc)
        QTimer.singleShot(600, _do_restore)


_service: InjectService | None = None


def service() -> InjectService:
    global _service
    if _service is None:
        _service = InjectService()
    return _service
