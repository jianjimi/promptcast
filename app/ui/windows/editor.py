"""Prompt editor window."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QComboBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QMessageBox,
    QPlainTextEdit,
    QPushButton,
    QVBoxLayout,
    QWidget,
)

from app.db.repositories import folders as folders_repo, prompts as prompts_repo, tags as tags_repo
from app.models import Prompt


class EditorWindow(QWidget):
    saved = pyqtSignal(int)  # prompt id

    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("PromptCast — Editor")
        self.resize(640, 600)
        self._editing_id: int | None = None

        self.title_input = QLineEdit()
        self.title_input.setPlaceholderText("标题")

        self.folder_combo = QComboBox()
        self.tags_input = QLineEdit()
        self.tags_input.setPlaceholderText("标签 (用逗号分隔)")

        self.content_input = QPlainTextEdit()
        self.content_input.setPlaceholderText("Markdown 内容…")

        save_btn = QPushButton("保存")
        save_btn.setProperty("role", "primary")
        save_btn.clicked.connect(self._on_save)

        cancel_btn = QPushButton("取消")
        cancel_btn.clicked.connect(self.close)

        meta_row = QHBoxLayout()
        meta_row.addWidget(QLabel("文件夹"))
        meta_row.addWidget(self.folder_combo, 1)
        meta_row.addSpacing(12)
        meta_row.addWidget(QLabel("标签"))
        meta_row.addWidget(self.tags_input, 2)

        actions = QHBoxLayout()
        actions.addStretch(1)
        actions.addWidget(cancel_btn)
        actions.addWidget(save_btn)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(10)
        layout.addWidget(self.title_input)
        layout.addLayout(meta_row)
        layout.addWidget(self.content_input, 1)
        layout.addLayout(actions)

    # ---- public API ----------------------------------------------------------------
    def open_for_id(self, prompt_id: int | None) -> None:
        self._editing_id = prompt_id
        self._reload_folders()
        self._reload_tags_for(prompt_id)
        if prompt_id is None:
            self.title_input.clear()
            self.content_input.clear()
            self.folder_combo.setCurrentIndex(0)
            self.setWindowTitle("PromptCast — 新建 Prompt")
        else:
            p = prompts_repo.get(prompt_id)
            if p is None:
                QMessageBox.warning(self, "PromptCast", "找不到 prompt")
                self.close()
                return
            self.title_input.setText(p.title)
            self.content_input.setPlainText(p.content)
            self._select_folder(p.folder_id)
            self.setWindowTitle(f"PromptCast — 编辑 «{p.title}»")
        self.show()
        self.raise_()
        self.activateWindow()
        self.title_input.setFocus()

    # ---- internals -----------------------------------------------------------------
    def _reload_folders(self) -> None:
        self.folder_combo.clear()
        self.folder_combo.addItem("（无文件夹）", None)
        for f in folders_repo.list_all():
            self.folder_combo.addItem(f.name, f.id)

    def _reload_tags_for(self, prompt_id: int | None) -> None:
        if prompt_id is None:
            self.tags_input.clear()
            return
        p = prompts_repo.get(prompt_id)
        if p is None:
            return
        all_tags = {t.id: t.name for t in tags_repo.list_all()}
        names = [all_tags[t] for t in p.tag_ids if t in all_tags]
        self.tags_input.setText(", ".join(names))

    def _select_folder(self, folder_id: int | None) -> None:
        for i in range(self.folder_combo.count()):
            if self.folder_combo.itemData(i) == folder_id:
                self.folder_combo.setCurrentIndex(i)
                return
        self.folder_combo.setCurrentIndex(0)

    def _parse_tag_ids(self) -> list[int]:
        names = [n.strip() for n in self.tags_input.text().split(",") if n.strip()]
        return [tags_repo.get_or_create(n) for n in names]

    def _on_save(self) -> None:
        title = self.title_input.text().strip()
        if not title:
            QMessageBox.warning(self, "PromptCast", "标题不能为空")
            return
        content = self.content_input.toPlainText()
        folder_id = self.folder_combo.currentData()
        tag_ids = self._parse_tag_ids()

        if self._editing_id is None:
            new_id = prompts_repo.create(title=title, content=content, folder_id=folder_id, tag_ids=tag_ids)
            self.saved.emit(new_id)
        else:
            prompts_repo.update(
                self._editing_id,
                title=title,
                content=content,
                folder_id=folder_id,
                clear_folder=folder_id is None,
                tag_ids=tag_ids,
            )
            self.saved.emit(self._editing_id)
        self.close()
