"""System tray icon."""
from __future__ import annotations

from PyQt6.QtGui import QIcon, QPixmap, QPainter, QColor, QBrush, QFont
from PyQt6.QtWidgets import QApplication, QMenu, QSystemTrayIcon

from app.config import APP_NAME, ASSETS_DIR


def _fallback_icon() -> QIcon:
    pix = QPixmap(64, 64)
    pix.fill(QColor("#18181b"))
    painter = QPainter(pix)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)
    painter.setBrush(QBrush(QColor("#fafafa")))
    painter.setPen(QColor("#fafafa"))
    font = painter.font()
    font.setBold(True)
    font.setPointSize(28)
    painter.setFont(font)
    painter.drawText(pix.rect(), 0x84, "P")  # Qt.AlignCenter == 0x84
    painter.end()
    return QIcon(pix)


def _load_icon() -> QIcon:
    for name in ("icon.png", "icon.ico"):
        candidate = ASSETS_DIR / name
        if candidate.exists():
            return QIcon(str(candidate))
    return _fallback_icon()


def install(controller) -> QSystemTrayIcon:
    """Install a system tray icon bound to the given AppController."""
    app = QApplication.instance()
    icon = _load_icon()
    tray = QSystemTrayIcon(icon, app)
    tray.setToolTip(APP_NAME)

    menu = QMenu()
    menu.addAction("显示抽屉", controller.show_drawer_initial)
    menu.addAction("设置…", controller.open_settings)
    menu.addSeparator()
    menu.addAction("退出", app.quit)
    tray.setContextMenu(menu)

    def _on_activated(reason: QSystemTrayIcon.ActivationReason) -> None:
        if reason == QSystemTrayIcon.ActivationReason.Trigger:
            controller.show_drawer_initial()

    tray.activated.connect(_on_activated)
    tray.show()
    return tray
