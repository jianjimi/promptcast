"""Prompt editor window."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QFont, QKeySequence, QShortcut
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
    saved = pyqtSignal(int)
    deleted = pyqtSignal(int)

    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("PromptCast — Editor")
        self.resize(720, 640)
        self._editing_id: int | None = None

        self.title_input = QLineEdit()
        self.title_input.setPlaceholderText("标题")

        self.folder_combo = QComboBox()
        self.tags_input = QLineEdit()
        self.tags_input.setPlaceholderText("标签 (用逗号分隔)")

        self.content_input = QPlainTextEdit()
        self.content_input.setPlaceholderText("Markdown 内容…")
        mono = QFont()
        mono.setFamily("JetBrains Mono")
        mono.setStyleHint(QFont.StyleHint.TypeWriter)
        mono.setPointSize(11)
        self.content_input.setFont(mono)
        self.content_input.setTabChangesFocus(False)

        self.save_btn = QPushButton("保存  (Ctrl+S)")
        self.save_btn.setProperty("role", "primary")
        self.save_btn.clicked.connect(self._on_save)

        self.cancel_btn = QPushButton("取消  (Esc)")
        self.cancel_btn.clicked.connect(self.close)

        self.delete_btn = QPushButton("删除")
        self.delete_btn.setProperty("role", "ghost")
        self.delete_btn.clicked.connect(self._on_delete)

        meta_row = QHBoxLayout()
        meta_row.setSpacing(10)
        f_label = QLabel("文件夹")
        f_label.setMinimumWidth(48)
        t_label = QLabel("标签")
        t_label.setMinimumWidth(36)
        meta_row.addWidget(f_label)
        meta_row.addWidget(self.folder_combo, 1)
        meta_row.addSpacing(8)
        meta_row.addWidget(t_label)
        meta_row.addWidget(self.tags_input, 2)

        actions = QHBoxLayout()
        actions.addWidget(self.delete_btn)
        actions.addStretch(1)
        actions.addWidget(self.cancel_btn)
        actions.addWidget(self.save_btn)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(18, 18, 18, 18)
        layout.setSpacing(10)
        layout.addWidget(self.title_input)
        layout.addLayout(meta_row)
        layout.addWidget(self.content_input, 1)
        layout.addLayout(actions)

        QShortcut(QKeySequence("Ctrl+S"), self, activated=self._on_save)
        QShortcut(QKeySequence("Ctrl+Return"), self, activated=self._on_save)
        QShortcut(QKeySequence("Esc"), self, activated=self.close)

    def open_for_id(self, prompt_id: int | None) -> None:
        self._editing_id = prompt_id
        self._reload_folders()
        self._reload_tags_for(prompt_id)
        if prompt_id is None:
            self.title_input.clear()
            self.content_input.clear()
            self.folder_combo.setCurrentIndex(0)
            self.delete_btn.hide()
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
            self.delete_btn.show()
            self.setWindowTitle(f"PromptCast — 编辑 «{p.title}»")
        self.show()
        self.raise_()
        self.activateWindow()
        self.title_input.setFocus()

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

    def _on_delete(self) -> None:
        if self._editing_id is None:
            return
        if QMessageBox.question(
            self, "删除 Prompt", "确定删除？此操作不可撤销。"
        ) != QMessageBox.StandardButton.Yes:
            return
        pid = self._editing_id
        prompts_repo.delete(pid)
        self.deleted.emit(pid)
        self.close()
