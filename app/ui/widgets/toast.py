"""Lightweight toast overlay anchored bottom-center of its parent."""
from __future__ import annotations

from PyQt6.QtCore import QTimer, Qt
from PyQt6.QtWidgets import QFrame, QHBoxLayout, QLabel, QWidget

_LEVEL_COLOR = {
    "info": "#52525b",
    "success": "#16a34a",
    "warning": "#d97706",
    "danger": "#dc2626",
}


class Toast(QFrame):
    def __init__(self, parent: QWidget) -> None:
        super().__init__(parent)
        self.setObjectName("ToastFrame")
        self.setAttribute(Qt.WidgetAttribute.WA_TransparentForMouseEvents)
        self._label = QLabel("")
        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 6, 12, 6)
        layout.addWidget(self._label)
        self.hide()

        self._timer = QTimer(self)
        self._timer.setSingleShot(True)
        self._timer.timeout.connect(self.hide)

    def show_message(self, text: str, level: str = "info", ms: int = 1800) -> None:
        color = _LEVEL_COLOR.get(level, _LEVEL_COLOR["info"])
        self._label.setText(text)
        self._label.setStyleSheet(f"color: {color};")
        self.adjustSize()
        self._reposition()
        self.show()
        self.raise_()
        self._timer.start(ms)

    def _reposition(self) -> None:
        parent = self.parentWidget()
        if parent is None:
            return
        x = (parent.width() - self.width()) // 2
        y = parent.height() - self.height() - 48
        self.move(max(8, x), max(8, y))
