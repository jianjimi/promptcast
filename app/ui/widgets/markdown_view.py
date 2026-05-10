"""Markdown renderer using markdown-it-py + Pygments, displayed in QTextBrowser."""
from __future__ import annotations

from functools import lru_cache

from markdown_it import MarkdownIt
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QDesktopServices
from PyQt6.QtWidgets import QTextBrowser

from app.services import theme as theme_service
from app.ui.styles.tokens import palette


@lru_cache(maxsize=2)
def _renderer() -> MarkdownIt:
    return MarkdownIt("commonmark", {"html": False, "linkify": True, "breaks": True}).enable("table")


def _wrap(html: str, theme: str) -> str:
    p = palette(theme)
    css = f"""
    body {{ font-family: {p['font_sans']}; font-size: {p['fs_13']}px; color: {p['text_primary']}; line-height: 1.55; padding: 12px 16px; }}
    h1, h2, h3 {{ color: {p['text_primary']}; }}
    h1 {{ font-size: {p['fs_18']}px; }}
    h2 {{ font-size: {p['fs_16']}px; }}
    h3 {{ font-size: {p['fs_14']}px; }}
    a {{ color: {p['accent']}; text-decoration: none; }}
    code {{ font-family: {p['font_mono']}; background: {p['bg_hover']}; padding: 1px 4px; border-radius: 3px; font-size: {p['fs_12']}px; }}
    pre {{ background: {p['bg_hover']}; border: 1px solid {p['border']}; border-radius: 6px; padding: 8px 10px; }}
    pre code {{ background: transparent; padding: 0; }}
    blockquote {{ border-left: 3px solid {p['border_strong']}; color: {p['text_secondary']}; margin: 0; padding-left: 10px; }}
    table {{ border-collapse: collapse; }}
    th, td {{ border: 1px solid {p['border']}; padding: 4px 8px; }}
    """
    return f"<style>{css}</style>{html}"


class MarkdownView(QTextBrowser):
    def __init__(self, parent=None) -> None:
        super().__init__(parent)
        self.setOpenExternalLinks(False)
        self.setOpenLinks(False)
        self.anchorClicked.connect(self._open_link)
        self.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)

    def set_markdown(self, text: str) -> None:
        html = _renderer().render(text or "")
        self.setHtml(_wrap(html, theme_service.manager().resolved))

    def _open_link(self, url) -> None:
        QDesktopServices.openUrl(url)
