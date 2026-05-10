"""Pill-shaped search input."""
from __future__ import annotations

from PyQt6.QtCore import pyqtSignal
from PyQt6.QtWidgets import QHBoxLayout, QLineEdit, QWidget


class SearchBar(QWidget):
    textChanged = pyqtSignal(str)
    submitted = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.input = QLineEdit()
        self.input.setObjectName("SearchInput")
        self.input.setPlaceholderText("搜索 prompts…")
        self.input.setClearButtonEnabled(True)
        self.input.textChanged.connect(self.textChanged)
        self.input.returnPressed.connect(self.submitted)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 8)
        layout.addWidget(self.input)

    def focus(self) -> None:
        self.input.setFocus()
        self.input.selectAll()

    def text(self) -> str:
        return self.input.text()

    def clear(self) -> None:
        self.input.clear()
