"""Application entry point."""
from __future__ import annotations

import logging
import sys

from PyQt6.QtWidgets import QApplication

from app.bootstrap import seed_if_empty
from app.config import APP_NAME, ensure_dirs
from app.db.connection import get_conn
from app.db.repositories import settings as settings_repo
from app.logging_setup import setup_logging
from app.services.theme import manager as theme_manager
from app.ui.windows.drawer import DrawerWindow

log = logging.getLogger(__name__)


def run() -> int:
    ensure_dirs()
    setup_logging()
    log.info("starting %s", APP_NAME)

    app = QApplication.instance() or QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    app.setQuitOnLastWindowClosed(False)

    get_conn()
    seed_if_empty()

    theme_choice = settings_repo.get("theme", "system")
    theme_manager().apply(theme_choice)

    drawer = DrawerWindow()
    drawer.requestHide.connect(drawer.hide)
    drawer.reload()
    drawer.show()

    return app.exec()


if __name__ == "__main__":
    raise SystemExit(run())
