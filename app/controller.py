"""App-level controller wiring the drawer + companion windows + services."""
from __future__ import annotations

import logging
from typing import Optional

from PyQt6.QtCore import QObject, QUrl
from PyQt6.QtGui import QDesktopServices

from app.db.repositories import settings as settings_repo, sites as sites_repo
from app.platform import ForegroundRef, get_platform
from app.services import theme as theme_service
from app.services.hotkey import service as hotkey_service
from app.services.inject import service as inject_service
from app.ui.windows.drawer import DrawerWindow
from app.ui.windows.editor import EditorWindow
from app.ui.windows.preview import PreviewWindow
from app.ui.windows.settings import SettingsWindow

log = logging.getLogger(__name__)


class AppController(QObject):
    def __init__(self) -> None:
        super().__init__()
        self._foreground = ForegroundRef()

        self.drawer = DrawerWindow()
        self.editor: Optional[EditorWindow] = None
        self.preview: Optional[PreviewWindow] = None
        self.settings: Optional[SettingsWindow] = None

        self._wire_drawer()
        self._wire_hotkey()
        inject_service().message.connect(self.drawer.show_toast)
        self.drawer.set_pinned(bool(settings_repo.get("pin_default", False)))
        self.drawer.reload()

    # ---- public ----------------------------------------------------------------
    def show_drawer_initial(self) -> None:
        self.drawer.show()
        self.drawer.raise_()
        self.drawer.activateWindow()

    # ---- wiring ----------------------------------------------------------------
    def _wire_drawer(self) -> None:
        self.drawer.requestInject.connect(self._on_inject)
        self.drawer.requestCopy.connect(self._on_copy)
        self.drawer.requestEdit.connect(self.open_editor)
        self.drawer.requestPreview.connect(self.open_preview)
        self.drawer.requestNew.connect(lambda: self.open_editor(None))
        self.drawer.requestOpenSite.connect(self._on_open_site)
        self.drawer.requestAddSite.connect(self.open_settings_to_sites)
        self.drawer.requestHide.connect(self._hide_drawer)

    def _wire_hotkey(self) -> None:
        svc = hotkey_service()
        svc.triggered.connect(self._on_hotkey)
        bound = settings_repo.get("hotkey", "ctrl+shift+space")
        if bound:
            svc.register(bound)

    # ---- handlers --------------------------------------------------------------
    def _on_hotkey(self) -> None:
        log.info("hotkey -> toggle drawer")
        if self.drawer.isVisible():
            self._hide_drawer()
        else:
            try:
                self._foreground = get_platform().capture_foreground()
            except Exception as exc:
                log.warning("capture foreground failed: %s", exc)
                self._foreground = ForegroundRef()
            self.drawer.toggle_visible()

    def _hide_drawer(self) -> None:
        self.drawer.hide()

    def _on_inject(self, prompt_id: int) -> None:
        self._hide_drawer()
        inject_service().inject(prompt_id, self._foreground)

    def _on_copy(self, prompt_id: int) -> None:
        inject_service().copy_only(prompt_id)
        self._hide_drawer()

    def _on_open_site(self, site_id: int) -> None:
        site = sites_repo.get(site_id)
        if site is None:
            return
        QDesktopServices.openUrl(QUrl(site.url))
        self._hide_drawer()

    # ---- companion windows -----------------------------------------------------
    def open_editor(self, prompt_id: int | None) -> None:
        if self.editor is None:
            self.editor = EditorWindow()
            self.editor.saved.connect(lambda _id: self.drawer.reload())
        self.editor.open_for_id(prompt_id)

    def open_preview(self, prompt_id: int) -> None:
        if self.preview is None:
            self.preview = PreviewWindow()
            self.preview.requestInject.connect(self._on_inject)
            self.preview.requestCopy.connect(self._on_copy)
        self.preview.open_for_id(prompt_id)

    def open_settings(self) -> None:
        if self.settings is None:
            self.settings = SettingsWindow()
            self.settings.hotkeyChanged.connect(self._on_hotkey_setting_changed)
            self.settings.themeChanged.connect(lambda _t: self.drawer.reload())
            self.settings.dataChanged.connect(self.drawer.reload)
        self.settings.show()
        self.settings.raise_()
        self.settings.activateWindow()

    def open_settings_to_sites(self) -> None:
        self.open_settings()
        if self.settings is not None:
            self.settings.nav.setCurrentRow(4)  # 0=general 1=hotkey 2=theme 3=folders 4=sites

    def _on_hotkey_setting_changed(self, value: str) -> None:
        ok = hotkey_service().register(value)
        if not ok:
            self.drawer.show_toast("热键注册失败", "danger")
        else:
            theme_service.manager()  # touch to keep linter happy
            self.drawer.show_toast(f"已绑定快捷键: {value}", "success")
