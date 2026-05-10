"""Horizontal favicon row + add button."""
from __future__ import annotations

from PyQt6.QtCore import QSize, Qt, pyqtSignal
from PyQt6.QtGui import QBrush, QColor, QIcon, QPainter, QPixmap
from PyQt6.QtWidgets import QHBoxLayout, QPushButton, QSizePolicy, QToolButton, QWidget

from app.models import Site

_PALETTE = ["#fee2e2", "#fef3c7", "#dcfce7", "#dbeafe", "#ede9fe", "#fce7f3", "#cffafe"]


def _fallback_icon(letter: str, seed: int) -> QIcon:
    pix = QPixmap(28, 28)
    pix.fill(Qt.GlobalColor.transparent)
    painter = QPainter(pix)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)
    color = QColor(_PALETTE[seed % len(_PALETTE)])
    painter.setBrush(QBrush(color))
    painter.setPen(Qt.PenStyle.NoPen)
    painter.drawEllipse(0, 0, 28, 28)
    painter.setPen(QColor("#52525b"))
    font = painter.font()
    font.setBold(True)
    font.setPointSize(11)
    painter.setFont(font)
    painter.drawText(pix.rect(), Qt.AlignmentFlag.AlignCenter, (letter or "?")[:1].upper())
    painter.end()
    return QIcon(pix)


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

        for index, site in enumerate(sites):
            self._row.insertWidget(self._row.count() - 1, self._make_button(site, index))

        add = QToolButton()
        add.setProperty("role", "ghost")
        add.setText("＋")
        add.setFixedSize(28, 28)
        add.setCursor(Qt.CursorShape.PointingHandCursor)
        add.setToolTip("添加站点")
        add.clicked.connect(self.addRequested)
        self._row.insertWidget(self._row.count() - 1, add)

    def _make_button(self, site: Site, index: int) -> QToolButton:
        btn = QToolButton()
        btn.setProperty("role", "ghost")
        btn.setFixedSize(28, 28)
        btn.setIconSize(QSize(20, 20))
        btn.setCursor(Qt.CursorShape.PointingHandCursor)
        btn.setToolTip(site.name)
        if site.favicon_blob:
            pix = QPixmap()
            pix.loadFromData(site.favicon_blob)
            if not pix.isNull():
                btn.setIcon(QIcon(pix))
            else:
                btn.setIcon(_fallback_icon(site.name, index))
        else:
            btn.setIcon(_fallback_icon(site.name, index))
        btn.clicked.connect(lambda _checked=False, sid=site.id: self.siteClicked.emit(sid))
        return btn
