"""Frameless 400×720 drawer window."""
from __future__ import annotations

import logging
from typing import Callable

from PyQt6.QtCore import QEvent, Qt, pyqtSignal
from PyQt6.QtGui import QGuiApplication, QKeyEvent, QShortcut, QKeySequence
from PyQt6.QtWidgets import (
    QFrame,
    QGraphicsDropShadowEffect,
    QVBoxLayout,
    QWidget,
)

from app.db.repositories import folders, prompts as prompts_repo, sites as sites_repo, tags
from app.models import Prompt, SortMode
from app.ui.widgets.filter_chips import ChipSpec, FilterChips
from app.ui.widgets.hint_bar import HintBar
from app.ui.widgets.prompt_list import PromptList
from app.ui.widgets.search_bar import SearchBar
from app.ui.widgets.site_launcher import SiteLauncher
from app.ui.widgets.toast import Toast

log = logging.getLogger(__name__)

DRAWER_WIDTH = 400
DRAWER_HEIGHT = 720
SHADOW_PAD = 24


class DrawerWindow(QWidget):
    """Top-level borderless drawer window."""

    requestInject = pyqtSignal(int)
    requestCopy = pyqtSignal(int)
    requestEdit = pyqtSignal(int)
    requestPreview = pyqtSignal(int)
    requestNew = pyqtSignal()
    requestOpenSite = pyqtSignal(int)
    requestAddSite = pyqtSignal()
    requestHide = pyqtSignal()

    def __init__(self) -> None:
        super().__init__(
            None,
            Qt.WindowType.FramelessWindowHint
            | Qt.WindowType.Tool
            | Qt.WindowType.WindowStaysOnTopHint,
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground, True)
        self.setObjectName("DrawerRoot")
        self.setFixedSize(DRAWER_WIDTH + SHADOW_PAD * 2, DRAWER_HEIGHT + SHADOW_PAD * 2)

        self._pinned = False
        self._all_prompts: list[Prompt] = []
        self._filtered: list[Prompt] = []
        self._selected_id: int | None = None
        self._sort_mode = SortMode.RECENT_USED

        self._build_ui()
        self._wire_shortcuts()
        self.center_on_screen()

    # ---- public API ----------------------------------------------------------------
    def reload(self) -> None:
        self._all_prompts = prompts_repo.list_all(self._sort_mode)
        self._refresh_chips()
        self._refresh_sites()
        self._apply_filters()

    def toggle_visible(self) -> None:
        if self.isVisible():
            self.hide()
        else:
            self.reload()
            self.show()
            self.raise_()
            self.activateWindow()
            self._search.focus()

    def set_pinned(self, value: bool) -> None:
        self._pinned = value

    def set_sort_mode(self, mode: SortMode) -> None:
        if mode != self._sort_mode:
            self._sort_mode = mode
            self.reload()

    def show_toast(self, message: str, level: str = "info") -> None:
        self._toast.show_message(message, level)

    # ---- construction --------------------------------------------------------------
    def _build_ui(self) -> None:
        outer = QVBoxLayout(self)
        outer.setContentsMargins(SHADOW_PAD, SHADOW_PAD, SHADOW_PAD, SHADOW_PAD)
        outer.setSpacing(0)

        card = QFrame(self)
        card.setObjectName("DrawerCard")
        card.setFixedSize(DRAWER_WIDTH, DRAWER_HEIGHT)
        outer.addWidget(card, 0, Qt.AlignmentFlag.AlignCenter)

        shadow = QGraphicsDropShadowEffect(card)
        shadow.setBlurRadius(36)
        shadow.setOffset(0, 12)
        shadow.setColor(Qt.GlobalColor.black)
        card.setGraphicsEffect(shadow)

        col = QVBoxLayout(card)
        col.setContentsMargins(0, 0, 0, 0)
        col.setSpacing(0)

        self._search = SearchBar(card)
        self._chips = FilterChips(card)
        self._list = PromptList(card)
        self._sites = SiteLauncher(card)
        self._hint = HintBar(card)

        col.addWidget(self._search)
        col.addWidget(self._chips)
        col.addWidget(self._list, 1)
        col.addWidget(self._sites)
        col.addWidget(self._hint)

        self._toast = Toast(card)

        self._search.textChanged.connect(self._on_query)
        self._search.submitted.connect(self._emit_inject)
        self._chips.chipChanged.connect(lambda _key: self._apply_filters())
        self._list.selected.connect(self._on_select)
        self._list.activated.connect(self.requestInject)
        self._list.favoriteToggled.connect(self._toggle_fav)
        self._list.edited.connect(self.requestEdit)
        self._sites.siteClicked.connect(self.requestOpenSite)
        self._sites.addRequested.connect(self.requestAddSite)

    def _wire_shortcuts(self) -> None:
        def sc(seq: str, slot: Callable[[], None]) -> None:
            QShortcut(QKeySequence(seq), self, activated=slot)

        sc("Down", lambda: self._list.move_selection(1))
        sc("Up", lambda: self._list.move_selection(-1))
        sc("Ctrl+C", self._emit_copy)
        sc("Ctrl+E", self._emit_edit)
        sc("Ctrl+N", self.requestNew.emit)
        sc("Ctrl+F", self._search.focus)
        sc("Tab", lambda: self._chips.cycle(1))
        sc("Shift+Tab", lambda: self._chips.cycle(-1))
        sc("Space", self._emit_preview)
        sc("Esc", self._on_escape)

    # ---- behavior ------------------------------------------------------------------
    def center_on_screen(self) -> None:
        screen = QGuiApplication.primaryScreen()
        if screen is None:
            return
        geo = screen.availableGeometry()
        x = geo.x() + (geo.width() - self.width()) // 2
        y = geo.y() + (geo.height() - self.height()) // 2
        self.move(x, y)

    def changeEvent(self, event: QEvent) -> None:  # type: ignore[override]
        if event.type() == QEvent.Type.ActivationChange and not self.isActiveWindow():
            if not self._pinned and self.isVisible():
                self.requestHide.emit()
        super().changeEvent(event)

    def keyPressEvent(self, event: QKeyEvent) -> None:  # type: ignore[override]
        # Letters typed while a list item is focused should fall through to the
        # search box for a snappy "type to search" feel.
        if (
            event.text()
            and event.text().isprintable()
            and not self._search.input.hasFocus()
            and event.modifiers() in (Qt.KeyboardModifier.NoModifier, Qt.KeyboardModifier.ShiftModifier)
        ):
            self._search.input.setFocus()
            self._search.input.setText(self._search.input.text() + event.text())
            return
        super().keyPressEvent(event)

    # ---- data wiring ---------------------------------------------------------------
    def _on_query(self, _text: str) -> None:
        self._apply_filters()

    def _apply_filters(self) -> None:
        chip_key = self._chips.active
        query = self._search.text().strip().lower()

        def chip_match(p: Prompt) -> bool:
            if chip_key == "all":
                return True
            if chip_key == "favorites":
                return p.is_favorite
            if chip_key.startswith("folder:"):
                return p.folder_id == int(chip_key.split(":", 1)[1])
            if chip_key.startswith("tag:"):
                return int(chip_key.split(":", 1)[1]) in p.tag_ids
            return True

        def text_match(p: Prompt) -> bool:
            if not query:
                return True
            return query in p.title.lower() or query in p.content.lower()

        self._filtered = [p for p in self._all_prompts if chip_match(p) and text_match(p)]
        self._list.set_prompts(self._filtered, self._selected_id)
        self._hint.set_count(len(self._filtered))

    def _refresh_chips(self) -> None:
        all_count = len(self._all_prompts)
        fav_count = sum(1 for p in self._all_prompts if p.is_favorite)
        chips = [
            ChipSpec("all", "全部", all_count),
            ChipSpec("favorites", "★ 收藏", fav_count),
        ]
        for f in folders.list_all():
            cnt = sum(1 for p in self._all_prompts if p.folder_id == f.id)
            chips.append(ChipSpec(f"folder:{f.id}", f.name, cnt))
        for t in tags.list_all():
            cnt = sum(1 for p in self._all_prompts if t.id in p.tag_ids)
            if cnt:
                chips.append(ChipSpec(f"tag:{t.id}", f"#{t.name}", cnt))
        self._chips.set_chips(chips)

    def _refresh_sites(self) -> None:
        self._sites.set_sites(sites_repo.list_all())

    def _on_select(self, prompt_id: int) -> None:
        self._selected_id = prompt_id

    def _toggle_fav(self, prompt_id: int) -> None:
        prompts_repo.toggle_favorite(prompt_id)
        self.reload()

    def _on_escape(self) -> None:
        if self._search.text():
            self._search.clear()
            return
        if not self._pinned:
            self.requestHide.emit()

    def _emit_inject(self) -> None:
        if self._selected_id is not None:
            self.requestInject.emit(self._selected_id)

    def _emit_copy(self) -> None:
        if self._selected_id is not None:
            self.requestCopy.emit(self._selected_id)

    def _emit_edit(self) -> None:
        if self._selected_id is not None:
            self.requestEdit.emit(self._selected_id)

    def _emit_preview(self) -> None:
        if self._selected_id is not None:
            self.requestPreview.emit(self._selected_id)
