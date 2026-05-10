"""Scrollable list of prompts with title/preview/star."""
from __future__ import annotations

from PyQt6.QtCore import QSize, Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QListWidget,
    QListWidgetItem,
    QPushButton,
    QSizePolicy,
    QVBoxLayout,
    QWidget,
)

from app.models import Prompt


class PromptList(QWidget):
    selected = pyqtSignal(int)         # prompt id
    activated = pyqtSignal(int)        # prompt id (double-click / Enter)
    favoriteToggled = pyqtSignal(int)
    edited = pyqtSignal(int)

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._list = QListWidget(self)
        self._list.setObjectName("PromptListView")
        self._list.setSelectionMode(QListWidget.SelectionMode.SingleSelection)
        self._list.setVerticalScrollMode(QListWidget.ScrollMode.ScrollPerPixel)
        self._list.setUniformItemSizes(False)
        self._list.setSpacing(2)
        self._list.itemSelectionChanged.connect(self._on_selection)
        self._list.itemDoubleClicked.connect(self._on_dbl)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 0, 8, 0)
        layout.setSpacing(0)
        layout.addWidget(self._list)

        self._items: list[Prompt] = []

    def set_prompts(self, prompts: list[Prompt], selected_id: int | None = None) -> None:
        self._items = prompts
        self._list.blockSignals(True)
        self._list.clear()
        for p in prompts:
            wid = _PromptRow(p)
            wid.favoriteClicked.connect(lambda _p=p: self.favoriteToggled.emit(_p.id))
            wid.editClicked.connect(lambda _p=p: self.edited.emit(_p.id))
            item = QListWidgetItem(self._list)
            item.setSizeHint(QSize(0, wid.sizeHint().height()))
            item.setData(Qt.ItemDataRole.UserRole, p.id)
            self._list.addItem(item)
            self._list.setItemWidget(item, wid)
        self._list.blockSignals(False)
        if selected_id is not None:
            self.select(selected_id)
        elif prompts:
            self._list.setCurrentRow(0)

    def select(self, prompt_id: int) -> None:
        for i in range(self._list.count()):
            if self._list.item(i).data(Qt.ItemDataRole.UserRole) == prompt_id:
                self._list.setCurrentRow(i)
                return

    def selected_id(self) -> int | None:
        item = self._list.currentItem()
        return item.data(Qt.ItemDataRole.UserRole) if item else None

    def move_selection(self, direction: int) -> None:
        if not self._items:
            return
        cur = self._list.currentRow()
        n = self._list.count()
        nxt = (cur + direction) % n if cur >= 0 else 0
        self._list.setCurrentRow(nxt)

    def _on_selection(self) -> None:
        sid = self.selected_id()
        if sid is not None:
            self.selected.emit(sid)

    def _on_dbl(self, item: QListWidgetItem) -> None:
        sid = item.data(Qt.ItemDataRole.UserRole)
        if sid is not None:
            self.activated.emit(sid)


class _PromptRow(QWidget):
    favoriteClicked = pyqtSignal()
    editClicked = pyqtSignal()

    def __init__(self, prompt: Prompt, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

        title = QLabel(prompt.title or "(无标题)")
        title.setProperty("role", "title")
        title.setMaximumHeight(20)
        title.setTextInteractionFlags(Qt.TextInteractionFlag.NoTextInteraction)

        preview_text = (prompt.content or "").strip().splitlines()
        first = preview_text[0] if preview_text else ""
        if len(first) > 70:
            first = first[:70] + "…"
        preview = QLabel(first or " ")
        preview.setProperty("role", "muted")
        preview.setMaximumHeight(18)

        star = QPushButton("★" if prompt.is_favorite else "☆")
        star.setProperty("role", "ghost")
        star.setFixedWidth(28)
        star.setCursor(Qt.CursorShape.PointingHandCursor)
        star.clicked.connect(self.favoriteClicked)

        text_col = QVBoxLayout()
        text_col.setContentsMargins(0, 0, 0, 0)
        text_col.setSpacing(2)
        text_col.addWidget(title)
        text_col.addWidget(preview)

        row = QHBoxLayout(self)
        row.setContentsMargins(8, 6, 6, 6)
        row.setSpacing(8)
        row.addLayout(text_col, 1)
        row.addWidget(star, 0, Qt.AlignmentFlag.AlignTop)
