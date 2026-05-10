"""Bottom shortcut legend."""
from __future__ import annotations

import sys

from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import QFrame, QHBoxLayout, QLabel, QWidget

_CMD = "⌘" if sys.platform == "darwin" else "Ctrl"


def _hint_text(count: int) -> str:
    return (
        f"↑↓ 选择   ↵ 注入   {_CMD} C 复制   {_CMD} E 编辑   "
        f"{_CMD} N 新建   Space 预览   Esc 关闭   ·  共 {count} 项"
    )


class HintBar(QFrame):
    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setObjectName("HintBar")
        self.setFixedHeight(28)
        self._label = QLabel(_hint_text(0))
        self._label.setObjectName("HintBarLabel")
        self._label.setAlignment(Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)

        row = QHBoxLayout(self)
        row.setContentsMargins(12, 0, 12, 0)
        row.addWidget(self._label)

    def set_count(self, count: int) -> None:
        self._label.setText(_hint_text(count))
