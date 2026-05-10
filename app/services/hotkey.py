"""Global hotkey listener — bridges pynput thread → Qt main thread."""
from __future__ import annotations

import logging

from PyQt6.QtCore import QObject, pyqtSignal

from app.ui.widgets.hotkey_recorder import to_pynput

log = logging.getLogger(__name__)


class HotkeyService(QObject):
    triggered = pyqtSignal()

    def __init__(self) -> None:
        super().__init__()
        self._listener = None
        self._current = ""

    def register(self, hotkey: str) -> bool:
        """Register `hotkey` (notation: "ctrl+shift+space"). Replaces any prior binding."""
        self.unregister()
        if not hotkey:
            return False
        try:
            from pynput.keyboard import GlobalHotKeys
            mapping = {to_pynput(hotkey): self._on_fire}
            listener = GlobalHotKeys(mapping)
            listener.daemon = True
            listener.start()
            self._listener = listener
            self._current = hotkey
            log.info("global hotkey registered: %s -> %s", hotkey, list(mapping.keys()))
            return True
        except Exception as exc:
            log.warning("hotkey register failed for %r: %s", hotkey, exc)
            self._listener = None
            return False

    def unregister(self) -> None:
        if self._listener is not None:
            try:
                self._listener.stop()
            except Exception as exc:
                log.warning("hotkey stop failed: %s", exc)
            self._listener = None
        self._current = ""

    @property
    def current(self) -> str:
        return self._current

    def _on_fire(self) -> None:
        # pynput callback runs in its listener thread; emit signal so the slot
        # executes in the Qt main thread (signals queue across threads).
        log.info("global hotkey fired")
        self.triggered.emit()


_service: HotkeyService | None = None


def service() -> HotkeyService:
    global _service
    if _service is None:
        _service = HotkeyService()
    return _service
