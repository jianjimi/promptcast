"""Theme management — applies QSS to the QApplication."""
from __future__ import annotations

import logging
import sys
from pathlib import Path

from PyQt6.QtCore import QObject, pyqtSignal
from PyQt6.QtGui import QColor, QGuiApplication, QPalette
from PyQt6.QtWidgets import QApplication

from app.config import STYLES_DIR
from app.ui.styles.tokens import palette

log = logging.getLogger(__name__)

_BASE_QSS_PATH: Path = STYLES_DIR / "base.qss"


def _resolve_theme(choice: str) -> str:
    """Map 'system' to a concrete theme by querying Qt's color scheme."""
    if choice in ("light", "dark"):
        return choice
    hints = QGuiApplication.styleHints()
    scheme = getattr(hints, "colorScheme", lambda: None)()
    return "dark" if str(scheme).endswith("Dark") else "light"


def render_qss(theme: str) -> str:
    template = _BASE_QSS_PATH.read_text(encoding="utf-8")
    return template.format(**palette(theme))


def _make_palette(theme: str) -> QPalette:
    """Build a QPalette so framework-drawn controls (tooltips, native dialogs,
    QFormLayout buddy labels) pick up the right colors instead of system defaults."""
    p = palette(theme)
    pal = QPalette()
    text = QColor(p["text_primary"])
    secondary = QColor(p["text_secondary"])
    base = QColor(p["bg_surface"])
    window = QColor(p["bg_base"])
    accent = QColor(p["accent"])
    accent_fg = QColor(p["accent_fg"])

    pal.setColor(QPalette.ColorRole.Window, window)
    pal.setColor(QPalette.ColorRole.WindowText, text)
    pal.setColor(QPalette.ColorRole.Base, base)
    pal.setColor(QPalette.ColorRole.AlternateBase, QColor(p["bg_hover"]))
    pal.setColor(QPalette.ColorRole.Text, text)
    pal.setColor(QPalette.ColorRole.Button, base)
    pal.setColor(QPalette.ColorRole.ButtonText, text)
    pal.setColor(QPalette.ColorRole.Highlight, accent)
    pal.setColor(QPalette.ColorRole.HighlightedText, accent_fg)
    pal.setColor(QPalette.ColorRole.PlaceholderText, secondary)
    pal.setColor(QPalette.ColorRole.ToolTipBase, base)
    pal.setColor(QPalette.ColorRole.ToolTipText, text)
    return pal


class ThemeManager(QObject):
    """Singleton-style helper that re-applies QSS on theme change."""

    themeChanged = pyqtSignal(str)  # emitted with the resolved theme ("light"/"dark")

    def __init__(self) -> None:
        super().__init__()
        self._choice = "system"
        self._resolved = "light"

        hints = QGuiApplication.styleHints()
        if hasattr(hints, "colorSchemeChanged"):
            hints.colorSchemeChanged.connect(self._on_system_scheme_changed)

    @property
    def resolved(self) -> str:
        return self._resolved

    def apply(self, choice: str) -> str:
        self._choice = choice
        resolved = _resolve_theme(choice)
        self._resolved = resolved
        qss = render_qss(resolved)
        app = QApplication.instance()
        if app is not None:
            app.setPalette(_make_palette(resolved))
            app.setStyleSheet(qss)
        log.info("theme applied: %s (resolved=%s)", choice, resolved)
        self.themeChanged.emit(resolved)
        return resolved

    def _on_system_scheme_changed(self, *_args) -> None:
        if self._choice == "system":
            self.apply("system")


_manager: ThemeManager | None = None


def manager() -> ThemeManager:
    global _manager
    if _manager is None:
        _manager = ThemeManager()
    return _manager
