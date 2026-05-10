"""Settings window with left-nav + page stack."""
from __future__ import annotations

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QCheckBox,
    QComboBox,
    QFormLayout,
    QHBoxLayout,
    QLabel,
    QListWidget,
    QListWidgetItem,
    QPushButton,
    QStackedWidget,
    QVBoxLayout,
    QWidget,
)

from app import __version__
from app.config import LOG_DIR
from app.db.repositories import settings as settings_repo
from app.services import theme as theme_service
from app.ui.widgets.data_panel import DataPanel
from app.ui.widgets.folders_panel import FoldersPanel
from app.ui.widgets.hotkey_recorder import HotkeyRecorder
from app.ui.widgets.sites_panel import SitesPanel


class SettingsWindow(QWidget):
    hotkeyChanged = pyqtSignal(str)
    themeChanged = pyqtSignal(str)
    dataChanged = pyqtSignal()

    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("PromptCast — 设置")
        self.resize(760, 580)

        self.nav = QListWidget()
        self.nav.setObjectName("SettingsNav")
        self.nav.setFixedWidth(170)

        self.stack = QStackedWidget()

        self._add_page("通用", self._build_general())
        self._add_page("热键", self._build_hotkey())
        self._add_page("主题", self._build_theme())
        self._add_page("文件夹", self._build_folders())
        self._add_page("站点", self._build_sites())
        self._add_page("数据", self._build_data())
        self._add_page("关于", self._build_about())

        self.nav.currentRowChanged.connect(self.stack.setCurrentIndex)
        self.nav.setCurrentRow(0)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        layout.addWidget(self.nav)
        layout.addWidget(self.stack, 1)

    # ---- nav helpers ---------------------------------------------------------------
    def _add_page(self, name: str, widget: QWidget) -> None:
        self.nav.addItem(QListWidgetItem(name))
        self.stack.addWidget(self._wrap(widget))

    @staticmethod
    def _wrap(widget: QWidget) -> QWidget:
        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(20, 20, 20, 20)
        layout.addWidget(widget)
        return container

    # ---- pages ---------------------------------------------------------------------
    def _build_general(self) -> QWidget:
        w = QWidget()
        form = QFormLayout(w)

        action = QComboBox()
        action.addItems(["注入到目标应用 (inject)", "仅复制到剪贴板 (copy_only)"])
        action.setCurrentIndex(0 if settings_repo.get("default_action", "inject") == "inject" else 1)
        action.currentIndexChanged.connect(
            lambda i: settings_repo.set_value("default_action", "inject" if i == 0 else "copy_only")
        )

        sort = QComboBox()
        sort.addItems(["最近使用", "创建时间", "更新时间", "标题"])
        idx = ["recent_used", "created", "updated", "title"].index(settings_repo.get("sort_mode", "recent_used"))
        sort.setCurrentIndex(idx)
        sort.currentIndexChanged.connect(
            lambda i: settings_repo.set_value("sort_mode", ["recent_used", "created", "updated", "title"][i])
        )

        pin_default = QCheckBox("默认钉住抽屉")
        pin_default.setChecked(bool(settings_repo.get("pin_default", False)))
        pin_default.toggled.connect(lambda v: settings_repo.set_value("pin_default", v))

        autostart = QCheckBox("开机自动启动")
        autostart.setChecked(bool(settings_repo.get("auto_start", False)))
        autostart.toggled.connect(self._on_autostart_toggled)
        self._autostart_box = autostart

        form.addRow("默认动作", action)
        form.addRow("排序方式", sort)
        form.addRow(pin_default)
        form.addRow(autostart)
        return w

    def _build_hotkey(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.addWidget(QLabel("点击下方输入框，按下你想要的组合键："))
        recorder = HotkeyRecorder()
        recorder.set_value(settings_repo.get("hotkey", "ctrl+shift+space"))
        recorder.hotkeyChanged.connect(self._on_hotkey_changed)
        layout.addWidget(recorder)
        layout.addStretch(1)
        self._hotkey_recorder = recorder
        return w

    def _build_theme(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.addWidget(QLabel("外观主题"))
        combo = QComboBox()
        combo.addItem("跟随系统", "system")
        combo.addItem("浅色", "light")
        combo.addItem("深色", "dark")
        current = settings_repo.get("theme", "system")
        combo.setCurrentIndex(["system", "light", "dark"].index(current))
        combo.currentIndexChanged.connect(lambda i: self._on_theme_changed(combo.itemData(i)))
        layout.addWidget(combo)
        layout.addStretch(1)
        return w

    def _build_folders(self) -> QWidget:
        panel = FoldersPanel()
        panel.changed.connect(self.dataChanged)
        return panel

    def _build_sites(self) -> QWidget:
        panel = SitesPanel()
        panel.changed.connect(self.dataChanged)
        return panel

    def _build_data(self) -> QWidget:
        panel = DataPanel()
        panel.changed.connect(self.dataChanged)
        return panel

    def _build_about(self) -> QWidget:
        w = QWidget()
        layout = QVBoxLayout(w)
        layout.addWidget(QLabel(f"<b>PromptCast</b>  v{__version__}"))
        layout.addWidget(QLabel("纯 Python + PyQt6 重构版"))
        open_logs = QPushButton("打开日志目录")
        open_logs.clicked.connect(self._open_log_dir)
        layout.addWidget(open_logs)
        layout.addStretch(1)
        return w

    # ---- handlers ------------------------------------------------------------------
    def _on_hotkey_changed(self, value: str) -> None:
        settings_repo.set_value("hotkey", value)
        self.hotkeyChanged.emit(value)

    def _on_theme_changed(self, choice: str) -> None:
        settings_repo.set_value("theme", choice)
        theme_service.manager().apply(choice)
        self.themeChanged.emit(choice)

    def _on_autostart_toggled(self, enabled: bool) -> None:
        settings_repo.set_value("auto_start", enabled)
        try:
            from app.services.autostart import set_enabled
            set_enabled(enabled)
        except Exception as exc:
            self._autostart_box.setChecked(not enabled)
            settings_repo.set_value("auto_start", not enabled)
            from PyQt6.QtWidgets import QMessageBox
            QMessageBox.warning(self, "PromptCast", f"自启动设置失败: {exc}")

    @staticmethod
    def _open_log_dir() -> None:
        from PyQt6.QtCore import QUrl
        from PyQt6.QtGui import QDesktopServices
        QDesktopServices.openUrl(QUrl.fromLocalFile(str(LOG_DIR)))


