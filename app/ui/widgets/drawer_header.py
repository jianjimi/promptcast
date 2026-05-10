"""Drawer top bar — drag handle + pin + sort + new + settings buttons."""
from __future__ import annotations

from PyQt6.QtCore import QPoint, Qt, pyqtSignal
from PyQt6.QtGui import QAction, QMouseEvent
from PyQt6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QLabel,
    QMenu,
    QToolButton,
    QWidget,
)

from app.models import SortMode

_SORT_LABELS = {
    SortMode.RECENT_USED: "最近使用",
    SortMode.CREATED: "创建时间",
    SortMode.UPDATED: "更新时间",
    SortMode.TITLE: "标题",
}


class DrawerHeader(QFrame):
    pinToggled = pyqtSignal(bool)
    sortChanged = pyqtSignal(object)  # SortMode
    newRequested = pyqtSignal()
    settingsRequested = pyqtSignal()
    dragMoved = pyqtSignal(QPoint)  # delta in screen coords

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setObjectName("DrawerHeader")
        self.setFixedHeight(36)
        self._pinned = False
        self._sort = SortMode.RECENT_USED
        self._drag_origin: QPoint | None = None

        title = QLabel("PromptCast")
        title.setProperty("role", "section")
        title.setStyleSheet("padding-left: 4px;")

        self._sort_btn = QToolButton()
        self._sort_btn.setText("↕")
        self._sort_btn.setToolTip("排序")
        self._sort_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self._sort_btn.setPopupMode(QToolButton.ToolButtonPopupMode.InstantPopup)
        self._sort_menu = QMenu(self)
        for mode, label in _SORT_LABELS.items():
            act = QAction(label, self)
            act.setCheckable(True)
            act.setChecked(mode == self._sort)
            act.triggered.connect(lambda _checked=False, m=mode: self._set_sort(m))
            self._sort_menu.addAction(act)
        self._sort_btn.setMenu(self._sort_menu)

        self._pin_btn = QToolButton()
        self._pin_btn.setCheckable(True)
        self._pin_btn.setText("📌")
        self._pin_btn.setToolTip("钉住抽屉 (不会因失焦关闭)")
        self._pin_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self._pin_btn.toggled.connect(self._on_pin_toggled)

        new_btn = QToolButton()
        new_btn.setText("＋")
        new_btn.setToolTip("新建 prompt (Ctrl+N)")
        new_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        new_btn.clicked.connect(self.newRequested)

        settings_btn = QToolButton()
        settings_btn.setText("⚙")
        settings_btn.setToolTip("设置")
        settings_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        settings_btn.clicked.connect(self.settingsRequested)

        for btn in (self._sort_btn, self._pin_btn, new_btn, settings_btn):
            btn.setProperty("role", "ghost")
            btn.setFixedSize(28, 28)

        row = QHBoxLayout(self)
        row.setContentsMargins(10, 4, 6, 4)
        row.setSpacing(2)
        row.addWidget(title, 1)
        row.addWidget(self._sort_btn)
        row.addWidget(self._pin_btn)
        row.addWidget(new_btn)
        row.addWidget(settings_btn)

    # ---- pin -----------------------------------------------------------------------
    def set_pinned(self, value: bool) -> None:
        self._pinned = value
        self._pin_btn.blockSignals(True)
        self._pin_btn.setChecked(value)
        self._pin_btn.blockSignals(False)

    def _on_pin_toggled(self, checked: bool) -> None:
        self._pinned = checked
        self.pinToggled.emit(checked)

    # ---- sort ----------------------------------------------------------------------
    def set_sort_mode(self, mode: SortMode) -> None:
        self._sort = mode
        for act in self._sort_menu.actions():
            act.setChecked(act.text() == _SORT_LABELS[mode])

    def _set_sort(self, mode: SortMode) -> None:
        self._sort = mode
        for act in self._sort_menu.actions():
            act.setChecked(act.text() == _SORT_LABELS[mode])
        self.sortChanged.emit(mode)

    # ---- drag-to-move --------------------------------------------------------------
    def mousePressEvent(self, event: QMouseEvent) -> None:  # type: ignore[override]
        if event.button() == Qt.MouseButton.LeftButton:
            self._drag_origin = event.globalPosition().toPoint()
        super().mousePressEvent(event)

    def mouseMoveEvent(self, event: QMouseEvent) -> None:  # type: ignore[override]
        if self._drag_origin is not None and event.buttons() & Qt.MouseButton.LeftButton:
            now = event.globalPosition().toPoint()
            delta = now - self._drag_origin
            self._drag_origin = now
            self.dragMoved.emit(delta)
        super().mouseMoveEvent(event)

    def mouseReleaseEvent(self, event: QMouseEvent) -> None:  # type: ignore[override]
        self._drag_origin = None
        super().mouseReleaseEvent(event)
