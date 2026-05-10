"""Folder management panel."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QInputDialog,
    QListWidget,
    QListWidgetItem,
    QMessageBox,
    QPushButton,
    QVBoxLayout,
    QWidget,
)

from app.db.repositories import folders as folders_repo


class FoldersPanel(QWidget):
    changed = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.list = QListWidget()
        self.list.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.list.model().rowsMoved.connect(lambda *_: self._persist_order())

        add_btn = QPushButton("新建")
        add_btn.setProperty("role", "primary")
        add_btn.clicked.connect(self._on_add)

        rename_btn = QPushButton("重命名")
        rename_btn.clicked.connect(self._on_rename)

        del_btn = QPushButton("删除")
        del_btn.clicked.connect(self._on_delete)

        actions = QHBoxLayout()
        actions.addWidget(add_btn)
        actions.addWidget(rename_btn)
        actions.addWidget(del_btn)
        actions.addStretch(1)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addLayout(actions)
        layout.addWidget(self.list, 1)

        self.reload()

    def reload(self) -> None:
        self.list.clear()
        for f in folders_repo.list_all():
            item = QListWidgetItem(f.name)
            item.setData(Qt.ItemDataRole.UserRole, f.id)
            self.list.addItem(item)

    def _selected_id(self) -> int | None:
        item = self.list.currentItem()
        return item.data(Qt.ItemDataRole.UserRole) if item else None

    def _on_add(self) -> None:
        name, ok = QInputDialog.getText(self, "新建文件夹", "名称")
        if ok and name.strip():
            folders_repo.create(name.strip())
            self.reload()
            self.changed.emit()

    def _on_rename(self) -> None:
        fid = self._selected_id()
        if fid is None:
            return
        item = self.list.currentItem()
        name, ok = QInputDialog.getText(self, "重命名", "新名称", text=item.text())
        if ok and name.strip():
            folders_repo.rename(fid, name.strip())
            self.reload()
            self.changed.emit()

    def _on_delete(self) -> None:
        fid = self._selected_id()
        if fid is None:
            return
        if QMessageBox.question(self, "删除文件夹", "确定删除？关联的 prompt 不会被删除。") != QMessageBox.StandardButton.Yes:
            return
        folders_repo.delete(fid)
        self.reload()
        self.changed.emit()

    def _persist_order(self) -> None:
        ids = [self.list.item(i).data(Qt.ItemDataRole.UserRole) for i in range(self.list.count())]
        folders_repo.reorder(ids)
        self.changed.emit()
