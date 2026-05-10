"""Bottom shortcut legend with auto-eliding for narrow widths."""
from __future__ import annotations

import sys

from PyQt6.QtCore import Qt
from PyQt6.QtGui import QFontMetrics
from PyQt6.QtWidgets import QFrame, QHBoxLayout, QLabel, QWidget

_CMD = "⌘" if sys.platform == "darwin" else "Ctrl"
_FULL_HINTS = [
    "↑↓ 选择", "↵ 注入",
    f"{_CMD}C 复制", f"{_CMD}E 编辑", f"{_CMD}N 新建",
    "Space 预览", "Esc 关闭",
]
_SHORT_HINTS = ["↑↓", "↵ 注入", f"{_CMD}C", f"{_CMD}E", f"{_CMD}N", "␣", "Esc"]


class HintBar(QFrame):
    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setObjectName("HintBar")
        self.setFixedHeight(28)
        self._count = 0

        self._hints_label = QLabel("")
        self._hints_label.setObjectName("HintBarLabel")
        self._hints_label.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)

        self._count_label = QLabel("")
        self._count_label.setObjectName("HintBarLabel")
        self._count_label.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)

        row = QHBoxLayout(self)
        row.setContentsMargins(12, 0, 12, 0)
        row.setSpacing(8)
        row.addWidget(self._hints_label, 1)
        row.addWidget(self._count_label, 0)

        self._refresh()

    def set_count(self, count: int) -> None:
        self._count = count
        self._refresh()

    def resizeEvent(self, event) -> None:  # type: ignore[override]
        super().resizeEvent(event)
        self._refresh()

    def _refresh(self) -> None:
        self._count_label.setText(f"{self._count} 项")
        avail = max(0, self._hints_label.width())
        fm = QFontMetrics(self._hints_label.font())

        full = "  ".join(_FULL_HINTS)
        if avail == 0 or fm.horizontalAdvance(full) <= avail:
            self._hints_label.setText(full)
            return

        compact = "  ".join(_SHORT_HINTS)
        if fm.horizontalAdvance(compact) <= avail:
            self._hints_label.setText(compact)
            return

        # Last resort — drop tokens until it fits.
        for n in range(len(_SHORT_HINTS) - 1, 0, -1):
            text = "  ".join(_SHORT_HINTS[:n])
            if fm.horizontalAdvance(text) <= avail:
                self._hints_label.setText(text)
                return
        self._hints_label.setText("")
