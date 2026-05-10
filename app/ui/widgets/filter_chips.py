"""Horizontal chip row for filtering by all/favorites/folder/tag."""
from __future__ import annotations

from dataclasses import dataclass

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QPushButton,
    QScrollArea,
    QWidget,
)


@dataclass(slots=True)
class ChipSpec:
    key: str
    label: str
    count: int | None = None


class FilterChips(QWidget):
    chipChanged = pyqtSignal(str)  # key

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._active = "all"
        self._buttons: dict[str, QPushButton] = {}

        self._inner = QWidget()
        self._row = QHBoxLayout(self._inner)
        self._row.setContentsMargins(12, 0, 12, 8)
        self._row.setSpacing(6)
        self._row.addStretch(1)

        scroll = QScrollArea(self)
        scroll.setWidgetResizable(True)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        scroll.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        scroll.setFrameShape(QScrollArea.Shape.NoFrame)
        scroll.setFixedHeight(36)
        scroll.setWidget(self._inner)

        outer = QHBoxLayout(self)
        outer.setContentsMargins(0, 0, 0, 0)
        outer.addWidget(scroll)

    def set_chips(self, chips: list[ChipSpec]) -> None:
        # Drop existing buttons
        while self._row.count() > 1:
            item = self._row.takeAt(0)
            w = item.widget()
            if w is not None:
                w.deleteLater()
        self._buttons.clear()

        for spec in chips:
            btn = QPushButton(self._format(spec))
            btn.setProperty("role", "chip")
            btn.setProperty("selected", "true" if spec.key == self._active else "false")
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(lambda _checked=False, k=spec.key: self.set_active(k))
            self._buttons[spec.key] = btn
            self._row.insertWidget(self._row.count() - 1, btn)
        self._restyle()

    def set_active(self, key: str) -> None:
        if key == self._active or key not in self._buttons:
            return
        self._active = key
        for k, btn in self._buttons.items():
            btn.setProperty("selected", "true" if k == key else "false")
        self._restyle()
        self.chipChanged.emit(key)

    def cycle(self, direction: int) -> None:
        keys = list(self._buttons.keys())
        if not keys:
            return
        try:
            idx = keys.index(self._active)
        except ValueError:
            idx = 0
        nxt = keys[(idx + direction) % len(keys)]
        self.set_active(nxt)

    @property
    def active(self) -> str:
        return self._active

    def _restyle(self) -> None:
        for btn in self._buttons.values():
            btn.style().unpolish(btn)
            btn.style().polish(btn)

    @staticmethod
    def _format(spec: ChipSpec) -> str:
        if spec.count is None:
            return spec.label
        return f"{spec.label}  {spec.count}"
