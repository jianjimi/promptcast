"""Data import/export panel."""
from __future__ import annotations

from PyQt6.QtCore import pyqtSignal
from PyQt6.QtWidgets import (
    QFileDialog,
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QPushButton,
    QRadioButton,
    QVBoxLayout,
    QWidget,
)


class DataPanel(QWidget):
    changed = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)

        export_btn = QPushButton("导出 JSON…")
        export_btn.setProperty("role", "primary")
        export_btn.clicked.connect(self._on_export)

        import_btn = QPushButton("导入 JSON…")
        import_btn.clicked.connect(self._on_import)

        self.merge_radio = QRadioButton("合并到现有数据")
        self.merge_radio.setChecked(True)
        self.replace_radio = QRadioButton("替换现有数据")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(10)

        layout.addWidget(QLabel("把所有 prompts / folders / tags / sites 一次性导出或导入。"))
        row = QHBoxLayout()
        row.addWidget(export_btn)
        row.addWidget(import_btn)
        row.addStretch(1)
        layout.addLayout(row)

        layout.addWidget(QLabel("导入模式"))
        layout.addWidget(self.merge_radio)
        layout.addWidget(self.replace_radio)
        layout.addStretch(1)

    def _on_export(self) -> None:
        path, _ = QFileDialog.getSaveFileName(self, "导出", "promptcast.json", "JSON (*.json)")
        if not path:
            return
        from app.services.importer import export_to_file
        export_to_file(path)
        QMessageBox.information(self, "PromptCast", "导出完成")

    def _on_import(self) -> None:
        path, _ = QFileDialog.getOpenFileName(self, "导入", "", "JSON (*.json)")
        if not path:
            return
        replace = self.replace_radio.isChecked()
        if replace and QMessageBox.question(self, "确认", "替换模式会清空现有数据，继续？") != QMessageBox.StandardButton.Yes:
            return
        from app.services.importer import import_from_file
        try:
            import_from_file(path, replace=replace)
        except Exception as exc:
            QMessageBox.warning(self, "PromptCast", f"导入失败: {exc}")
            return
        QMessageBox.information(self, "PromptCast", "导入完成")
        self.changed.emit()
