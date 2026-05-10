"""Live keyboard shortcut recorder."""
from __future__ import annotations

import sys

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QKeyEvent
from PyQt6.QtWidgets import QHBoxLayout, QLabel, QLineEdit, QPushButton, QWidget

# Map Qt key names to a portable hotkey notation accepted by pynput
# (e.g. "<ctrl>+<shift>+space"). The display string uses ⌘ ⇧ ⌥ on macOS.
_MOD_TO_NAME = {
    Qt.KeyboardModifier.ControlModifier: "ctrl",
    Qt.KeyboardModifier.ShiftModifier: "shift",
    Qt.KeyboardModifier.AltModifier: "alt",
    Qt.KeyboardModifier.MetaModifier: "cmd",
}

_MOD_DISPLAY_DARWIN = {"ctrl": "⌃", "shift": "⇧", "alt": "⌥", "cmd": "⌘"}
_MOD_DISPLAY_OTHER = {"ctrl": "Ctrl", "shift": "Shift", "alt": "Alt", "cmd": "Win"}

_SPECIAL_KEY_NAMES = {
    Qt.Key.Key_Space: "space",
    Qt.Key.Key_Tab: "tab",
    Qt.Key.Key_Return: "enter",
    Qt.Key.Key_Enter: "enter",
    Qt.Key.Key_Escape: "esc",
    Qt.Key.Key_Backspace: "backspace",
    Qt.Key.Key_Delete: "delete",
    Qt.Key.Key_Up: "up",
    Qt.Key.Key_Down: "down",
    Qt.Key.Key_Left: "left",
    Qt.Key.Key_Right: "right",
    Qt.Key.Key_Home: "home",
    Qt.Key.Key_End: "end",
    Qt.Key.Key_PageUp: "page_up",
    Qt.Key.Key_PageDown: "page_down",
}
for i in range(1, 13):
    _SPECIAL_KEY_NAMES[getattr(Qt.Key, f"Key_F{i}")] = f"f{i}"


def parse_hotkey(value: str) -> tuple[list[str], str | None]:
    """Split a stored "ctrl+shift+space" string into (mods, key)."""
    parts = [p.strip().lower() for p in value.split("+") if p.strip()]
    mods = [p for p in parts if p in _MOD_TO_NAME.values()]
    keys = [p for p in parts if p not in mods]
    return mods, keys[-1] if keys else None


def format_for_display(value: str) -> str:
    mods, key = parse_hotkey(value)
    table = _MOD_DISPLAY_DARWIN if sys.platform == "darwin" else _MOD_DISPLAY_OTHER
    parts = [table.get(m, m.title()) for m in mods]
    if key:
        parts.append(key.upper() if len(key) == 1 else key.title())
    sep = " " if sys.platform == "darwin" else " + "
    return sep.join(parts)


def to_pynput(value: str) -> str:
    """Convert "ctrl+shift+space" → "<ctrl>+<shift>+<space>" for pynput."""
    mods, key = parse_hotkey(value)
    parts = [f"<{m}>" for m in mods]
    if key:
        parts.append(f"<{key}>" if len(key) > 1 else key)
    return "+".join(parts)


class _Capture(QLineEdit):
    captured = pyqtSignal(str)

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setReadOnly(True)
        self.setPlaceholderText("点击 → 按下组合键…")
        self._recording = False

    def mousePressEvent(self, event) -> None:  # type: ignore[override]
        self._recording = True
        self.setText("按下组合键…")
        self.setFocus()
        super().mousePressEvent(event)

    def keyPressEvent(self, event: QKeyEvent) -> None:  # type: ignore[override]
        if not self._recording:
            return
        key = event.key()
        if key in (Qt.Key.Key_Control, Qt.Key.Key_Shift, Qt.Key.Key_Alt, Qt.Key.Key_Meta):
            return  # waiting for non-modifier

        mods: list[str] = []
        m = event.modifiers()
        for flag, name in _MOD_TO_NAME.items():
            if m & flag:
                mods.append(name)

        if key in _SPECIAL_KEY_NAMES:
            key_name = _SPECIAL_KEY_NAMES[key]
        else:
            text = event.text()
            if text and text.isprintable():
                key_name = text.lower()
            else:
                return

        combo = "+".join(mods + [key_name])
        self._recording = False
        self.setText(format_for_display(combo))
        self.captured.emit(combo)


class HotkeyRecorder(QWidget):
    hotkeyChanged = pyqtSignal(str)

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._value = ""
        self._capture = _Capture(self)
        self._capture.captured.connect(self._on_captured)

        clear = QPushButton("清除")
        clear.setProperty("role", "ghost")
        clear.clicked.connect(self._clear)

        self._hint = QLabel("")
        self._hint.setProperty("role", "hint")

        row = QHBoxLayout(self)
        row.setContentsMargins(0, 0, 0, 0)
        row.setSpacing(8)
        row.addWidget(self._capture, 1)
        row.addWidget(clear)
        row.addWidget(self._hint)

    def set_value(self, raw: str) -> None:
        self._value = raw
        self._capture.setText(format_for_display(raw))

    def value(self) -> str:
        return self._value

    def _on_captured(self, combo: str) -> None:
        self._value = combo
        self.hotkeyChanged.emit(combo)

    def _clear(self) -> None:
        self._value = ""
        self._capture.setText("")
        self.hotkeyChanged.emit("")
