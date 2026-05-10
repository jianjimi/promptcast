"""Site management panel."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QDialog,
    QDialogButtonBox,
    QFormLayout,
    QHBoxLayout,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMessageBox,
    QPushButton,
    QVBoxLayout,
    QWidget,
)

from app.db.repositories import sites as sites_repo


class _SiteDialog(QDialog):
    def __init__(self, parent: QWidget | None = None, *, name: str = "", url: str = "") -> None:
        super().__init__(parent)
        self.setWindowTitle("站点")
        self.name = QLineEdit(name)
        self.url = QLineEdit(url)
        form = QFormLayout(self)
        form.addRow("名称", self.name)
        form.addRow("URL", self.url)
        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        bb.accepted.connect(self.accept)
        bb.rejected.connect(self.reject)
        form.addRow(bb)

    def result_pair(self) -> tuple[str, str] | None:
        n, u = self.name.text().strip(), self.url.text().strip()
        if not n or not u:
            return None
        return n, u


class SitesPanel(QWidget):
    changed = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.list = QListWidget()
        self.list.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.list.model().rowsMoved.connect(lambda *_: self._persist_order())

        add = QPushButton("新建")
        add.setProperty("role", "primary")
        add.clicked.connect(self._on_add)

        edit = QPushButton("编辑")
        edit.clicked.connect(self._on_edit)

        delete = QPushButton("删除")
        delete.clicked.connect(self._on_delete)

        refresh = QPushButton("刷新 favicon")
        refresh.clicked.connect(self._on_refresh)

        actions = QHBoxLayout()
        for b in (add, edit, delete, refresh):
            actions.addWidget(b)
        actions.addStretch(1)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addLayout(actions)
        layout.addWidget(self.list, 1)

        self.reload()

    def reload(self) -> None:
        self.list.clear()
        for s in sites_repo.list_all():
            item = QListWidgetItem(f"{s.name}  —  {s.url}")
            item.setData(Qt.ItemDataRole.UserRole, s.id)
            self.list.addItem(item)

    def _selected(self) -> int | None:
        item = self.list.currentItem()
        return item.data(Qt.ItemDataRole.UserRole) if item else None

    def _on_add(self) -> None:
        dlg = _SiteDialog(self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            pair = dlg.result_pair()
            if pair is None:
                return
            sid = sites_repo.create(*pair)
            self._fetch_favicon(sid, pair[1])
            self.reload()
            self.changed.emit()

    def _on_edit(self) -> None:
        sid = self._selected()
        if sid is None:
            return
        site = sites_repo.get(sid)
        if site is None:
            return
        dlg = _SiteDialog(self, name=site.name, url=site.url)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            pair = dlg.result_pair()
            if pair is None:
                return
            sites_repo.update(sid, name=pair[0], url=pair[1])
            if pair[1] != site.url:
                self._fetch_favicon(sid, pair[1])
            self.reload()
            self.changed.emit()

    def _on_delete(self) -> None:
        sid = self._selected()
        if sid is None:
            return
        if QMessageBox.question(self, "删除站点", "确定删除？") != QMessageBox.StandardButton.Yes:
            return
        sites_repo.delete(sid)
        self.reload()
        self.changed.emit()

    def _on_refresh(self) -> None:
        sid = self._selected()
        if sid is None:
            return
        site = sites_repo.get(sid)
        if site:
            self._fetch_favicon(sid, site.url)

    def _fetch_favicon(self, site_id: int, url: str) -> None:
        # Lazy import — keeps GUI startup fast and avoids requests dep on demand.
        from app.services.favicon import fetch_async
        fetch_async(site_id, url, on_done=lambda: self.changed.emit())

    def _persist_order(self) -> None:
        ids = [self.list.item(i).data(Qt.ItemDataRole.UserRole) for i in range(self.list.count())]
        sites_repo.reorder(ids)
        self.changed.emit()
