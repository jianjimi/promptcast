"""Horizontal favicon row + add button."""
from __future__ import annotations

from PyQt6.QtCore import QSize, Qt, pyqtSignal
from PyQt6.QtGui import QPixmap
from PyQt6.QtWidgets import QHBoxLayout, QPushButton, QSizePolicy, QWidget

from app.models import Site


class SiteLauncher(QWidget):
    siteClicked = pyqtSignal(int)
    addRequested = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._row = QHBoxLayout(self)
        self._row.setContentsMargins(12, 4, 12, 4)
        self._row.setSpacing(6)
        self._row.addStretch(1)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)
        self.setFixedHeight(40)

    def set_sites(self, sites: list[Site]) -> None:
        while self._row.count() > 1:
            item = self._row.takeAt(0)
            w = item.widget()
            if w is not None:
                w.deleteLater()

        for site in sites:
            btn = self._make_button(site)
            self._row.insertWidget(self._row.count() - 1, btn)

        add = QPushButton("+")
        add.setProperty("role", "ghost")
        add.setFixedSize(28, 28)
        add.setCursor(Qt.CursorShape.PointingHandCursor)
        add.setToolTip("添加站点")
        add.clicked.connect(self.addRequested)
        self._row.insertWidget(self._row.count() - 1, add)

    def _make_button(self, site: Site) -> QPushButton:
        btn = QPushButton()
        btn.setProperty("role", "ghost")
        btn.setFixedSize(28, 28)
        btn.setIconSize(QSize(18, 18))
        btn.setCursor(Qt.CursorShape.PointingHandCursor)
        btn.setToolTip(site.name)
        if site.favicon_blob:
            pix = QPixmap()
            pix.loadFromData(site.favicon_blob)
            if not pix.isNull():
                from PyQt6.QtGui import QIcon
                btn.setIcon(QIcon(pix))
            else:
                btn.setText(site.name[:1])
        else:
            btn.setText(site.name[:1])
        btn.clicked.connect(lambda _checked=False, sid=site.id: self.siteClicked.emit(sid))
        return btn
