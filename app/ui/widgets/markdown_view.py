"""Markdown renderer using markdown-it-py + Pygments, displayed in QTextBrowser.

QTextBrowser uses Qt's HTML subset (CSS support is limited and inline styles
are the most reliable). We post-process the markdown-it HTML to inject inline
styles for headings, code blocks, blockquotes, etc.
"""
from __future__ import annotations

import re
from functools import lru_cache

from markdown_it import MarkdownIt
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QDesktopServices
from PyQt6.QtWidgets import QTextBrowser

from app.services import theme as theme_service
from app.ui.styles.tokens import palette


@lru_cache(maxsize=2)
def _renderer(theme: str) -> MarkdownIt:
    md = MarkdownIt("commonmark", {"html": False, "linkify": True, "breaks": True}).enable("table")

    def fence_render(self, tokens, idx, _options, _env):
        tok = tokens[idx]
        return _highlight(tok.content, (tok.info or "").strip() or None, theme=theme)

    md.add_render_rule("fence", fence_render)
    return md


def _highlight(code: str, lang: str | None, *, theme: str = "light") -> str:
    """Pygments → inline-styled HTML; falls back to <pre> on failure."""
    try:
        from pygments import highlight
        from pygments.lexers import TextLexer, get_lexer_by_name
        from pygments.formatters import HtmlFormatter
    except Exception:
        return f"<pre>{_escape(code)}</pre>"

    lexer = TextLexer()
    if lang:
        try:
            lexer = get_lexer_by_name(lang, stripall=True)
        except Exception:
            pass
    style = "monokai" if theme == "dark" else "default"
    formatter = HtmlFormatter(noclasses=True, nowrap=False, style=style)
    return highlight(code, lexer, formatter)


def _escape(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
    )


def _inline_style(html: str, theme: str) -> str:
    """Inject inline styles QTextBrowser can render reliably."""
    p = palette(theme)
    border = p["border"]
    code_bg = p["bg_hover"]
    secondary = p["text_secondary"]
    accent = p["accent"]
    mono = p["font_mono"]

    # Headings
    html = re.sub(r"<h1>", f'<h1 style="font-size:20px;margin:12px 0 6px 0;">', html)
    html = re.sub(r"<h2>", f'<h2 style="font-size:17px;margin:10px 0 6px 0;">', html)
    html = re.sub(r"<h3>", f'<h3 style="font-size:15px;margin:8px 0 4px 0;">', html)
    # Inline code
    html = re.sub(
        r"<code>",
        f'<code style="font-family:{mono};background:{code_bg};padding:1px 4px;'
        f'border-radius:3px;font-size:12px;">',
        html,
    )
    # Pre blocks (code fences) — wrap in styled div
    html = re.sub(
        r"<pre>",
        f'<pre style="background:{code_bg};border:1px solid {border};'
        f'border-radius:6px;padding:8px 10px;font-family:{mono};font-size:12px;">',
        html,
    )
    # Blockquote
    html = re.sub(
        r"<blockquote>",
        f'<blockquote style="border-left:3px solid {border};color:{secondary};'
        f'margin:6px 0;padding:0 10px;">',
        html,
    )
    # Links
    html = re.sub(r"<a href=", f'<a style="color:{accent};text-decoration:none;" href=', html)
    return html


class MarkdownView(QTextBrowser):
    def __init__(self, parent=None) -> None:
        super().__init__(parent)
        self.setOpenExternalLinks(False)
        self.setOpenLinks(False)
        self.anchorClicked.connect(self._open_link)
        self.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)

    def set_markdown(self, text: str) -> None:
        theme = theme_service.manager().resolved
        html = _renderer(theme).render(text or "")
        styled = _inline_style(html, theme)
        p = palette(theme)
        wrapper = (
            f'<div style="font-family:{p["font_sans"]};font-size:13px;'
            f'color:{p["text_primary"]};line-height:1.55;">{styled}</div>'
        )
        self.setHtml(wrapper)

    def _open_link(self, url) -> None:
        QDesktopServices.openUrl(url)
