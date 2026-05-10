"""Markdown preview window."""
from __future__ import annotations

from PyQt6.QtCore import pyqtSignal
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QPushButton,
    QVBoxLayout,
    QWidget,
)

from app.db.repositories import prompts as prompts_repo
from app.ui.widgets.markdown_view import MarkdownView


class PreviewWindow(QWidget):
    requestInject = pyqtSignal(int)
    requestCopy = pyqtSignal(int)

    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("PromptCast — Preview")
        self.resize(560, 660)
        self._prompt_id: int | None = None

        self.title_label = QLabel("")
        self.title_label.setProperty("role", "title")
        self.title_label.setStyleSheet("font-size: 16px; font-weight: 600;")

        self.view = MarkdownView()

        inject_btn = QPushButton("注入")
        inject_btn.setProperty("role", "primary")
        inject_btn.clicked.connect(self._on_inject)

        copy_btn = QPushButton("复制")
        copy_btn.clicked.connect(self._on_copy)

        actions = QHBoxLayout()
        actions.addWidget(self.title_label, 1)
        actions.addWidget(copy_btn)
        actions.addWidget(inject_btn)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(8)
        layout.addLayout(actions)
        layout.addWidget(self.view, 1)

    def open_for_id(self, prompt_id: int) -> None:
        p = prompts_repo.get(prompt_id)
        if p is None:
            QMessageBox.warning(self, "PromptCast", "找不到 prompt")
            return
        self._prompt_id = prompt_id
        self.title_label.setText(p.title)
        self.view.set_markdown(p.content)
        self.show()
        self.raise_()
        self.activateWindow()

    def _on_inject(self) -> None:
        if self._prompt_id is not None:
            self.requestInject.emit(self._prompt_id)

    def _on_copy(self) -> None:
        if self._prompt_id is not None:
            self.requestCopy.emit(self._prompt_id)
