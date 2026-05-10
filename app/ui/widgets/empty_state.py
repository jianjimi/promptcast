"""Empty-state placeholder shown when no prompts match."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import QLabel, QPushButton, QVBoxLayout, QWidget


class EmptyState(QWidget):
    actionClicked = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._icon = QLabel("✦")
        self._icon.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._icon.setStyleSheet("font-size: 36px; color: #a1a1aa;")

        self._title = QLabel("还没有 prompt")
        self._title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._title.setProperty("role", "title")

        self._body = QLabel("按 Ctrl+N 新建一条，或试试不同的搜索词。")
        self._body.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._body.setProperty("role", "muted")
        self._body.setWordWrap(True)

        self._cta = QPushButton("新建 Prompt")
        self._cta.setProperty("role", "primary")
        self._cta.setFixedWidth(140)
        self._cta.setCursor(Qt.CursorShape.PointingHandCursor)
        self._cta.clicked.connect(self.actionClicked)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 60, 24, 60)
        layout.setSpacing(10)
        layout.addStretch(1)
        layout.addWidget(self._icon)
        layout.addWidget(self._title)
        layout.addWidget(self._body)
        layout.addSpacing(12)
        layout.addWidget(self._cta, 0, Qt.AlignmentFlag.AlignHCenter)
        layout.addStretch(2)

    def configure(self, *, has_query: bool, has_filter: bool) -> None:
        if has_query:
            self._title.setText("没有匹配的 prompt")
            self._body.setText("换个关键词试试。")
            self._cta.setText("清空搜索")
        elif has_filter:
            self._title.setText("此分类下没有 prompt")
            self._body.setText("切到「全部」或新建一条。")
            self._cta.setText("新建 Prompt")
        else:
            self._title.setText("还没有 prompt")
            self._body.setText("按 Ctrl+N 新建一条，把常用提示词攒起来。")
            self._cta.setText("新建 Prompt")
