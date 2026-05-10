"""Scrollable list of prompts with title/preview/star + context menu."""
from __future__ import annotations

from PyQt6.QtCore import QPoint, QSize, Qt, pyqtSignal
from PyQt6.QtGui import QAction, QFont, QFontMetrics, QPainter, QPainterPath, QColor
from PyQt6.QtCore import QPointF
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QListWidget,
    QListWidgetItem,
    QMenu,
    QSizePolicy,
    QToolButton,
    QVBoxLayout,
    QWidget,
)

from app.models import Prompt


class PromptList(QWidget):
    selected = pyqtSignal(int)
    activated = pyqtSignal(int)
    favoriteToggled = pyqtSignal(int)
    pinToggled = pyqtSignal(int)
    edited = pyqtSignal(int)
    duplicated = pyqtSignal(int)
    deleted = pyqtSignal(int)
    copied = pyqtSignal(int)

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
        self._list.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self._list.customContextMenuRequested.connect(self._on_context_menu)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 0, 8, 0)
        layout.setSpacing(0)
        layout.addWidget(self._list)

        self._items: list[Prompt] = []
        self._empty_label: QLabel | None = None

    def set_prompts(self, prompts: list[Prompt], selected_id: int | None = None) -> None:
        self._items = prompts
        self._list.blockSignals(True)
        self._list.clear()
        for p in prompts:
            wid = _PromptRow(p)
            wid.favoriteClicked.connect(lambda _checked=False, _p=p: self.favoriteToggled.emit(_p.id))
            item = QListWidgetItem(self._list)
            item.setSizeHint(QSize(0, _PromptRow.ROW_HEIGHT))
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

    def _on_context_menu(self, pos: QPoint) -> None:
        item = self._list.itemAt(pos)
        if item is None:
            return
        sid = item.data(Qt.ItemDataRole.UserRole)
        prompt = next((p for p in self._items if p.id == sid), None)
        if prompt is None:
            return
        self._list.setCurrentItem(item)

        menu = QMenu(self)
        menu.addAction("注入", lambda: self.activated.emit(sid))
        menu.addAction("复制内容", lambda: self.copied.emit(sid))
        menu.addAction("编辑…", lambda: self.edited.emit(sid))
        menu.addSeparator()
        fav = QAction("取消收藏" if prompt.is_favorite else "收藏", menu)
        fav.triggered.connect(lambda: self.favoriteToggled.emit(sid))
        menu.addAction(fav)
        pin = QAction("取消置顶" if prompt.is_pinned else "置顶", menu)
        pin.triggered.connect(lambda: self.pinToggled.emit(sid))
        menu.addAction(pin)
        menu.addSeparator()
        menu.addAction("复制为新条目", lambda: self.duplicated.emit(sid))
        delete_action = menu.addAction("删除")
        delete_action.triggered.connect(lambda: self.deleted.emit(sid))
        menu.exec(self._list.viewport().mapToGlobal(pos))


class _StarButton(QToolButton):
    """Always-visible star button drawn with QPainter."""

    def __init__(self, filled: bool, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._filled = filled
        self.setFixedSize(28, 28)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setAutoRaise(True)
        self.setStyleSheet("QToolButton { border: none; background: transparent; }")

    def paintEvent(self, _event) -> None:  # type: ignore[override]
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)
        rect = self.rect().adjusted(6, 6, -6, -6)
        cx, cy = rect.center().x(), rect.center().y()
        r_outer = min(rect.width(), rect.height()) / 2
        r_inner = r_outer * 0.42

        path = QPainterPath()
        import math
        for i in range(10):
            angle = -math.pi / 2 + i * math.pi / 5
            r = r_outer if i % 2 == 0 else r_inner
            x = cx + math.cos(angle) * r
            y = cy + math.sin(angle) * r
            if i == 0:
                path.moveTo(QPointF(x, y))
            else:
                path.lineTo(QPointF(x, y))
        path.closeSubpath()

        if self._filled:
            painter.fillPath(path, QColor("#f59e0b"))
        else:
            pen = painter.pen()
            pen.setColor(QColor("#a1a1aa"))
            pen.setWidthF(1.4)
            painter.setPen(pen)
            painter.drawPath(path)


class _PromptRow(QWidget):
    favoriteClicked = pyqtSignal()
    ROW_HEIGHT = 60

    def __init__(self, prompt: Prompt, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setFixedHeight(self.ROW_HEIGHT)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)

        title_text = prompt.title.strip() or "(无标题)"
        if prompt.is_pinned:
            title_text = "📌  " + title_text
        title = QLabel(title_text)
        title.setProperty("role", "title")
        title.setWordWrap(False)
        title.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)
        title.setFixedHeight(24)

        first_line = next(
            (line for line in (prompt.content or "").splitlines() if line.strip()),
            "",
        )
        if len(first_line) > 80:
            first_line = first_line[:80] + "…"
        preview = QLabel(first_line)
        preview.setProperty("role", "muted")
        preview.setWordWrap(False)
        preview.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)
        preview.setFixedHeight(20)

        star = _StarButton(prompt.is_favorite)
        star.clicked.connect(self.favoriteClicked)

        text_col = QVBoxLayout()
        text_col.setContentsMargins(0, 0, 0, 0)
        text_col.setSpacing(2)
        text_col.addWidget(title)
        text_col.addWidget(preview)
        text_col.addStretch(1)

        row = QHBoxLayout(self)
        row.setContentsMargins(10, 6, 6, 6)
        row.setSpacing(8)
        row.addLayout(text_col, 1)
        row.addWidget(star, 0, Qt.AlignmentFlag.AlignVCenter)
