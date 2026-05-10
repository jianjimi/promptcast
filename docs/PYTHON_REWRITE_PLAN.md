# PromptCast — Python + PyQt6 重构计划

## 0. 背景与目标
原 Tauri (Rust + Vue) 版本在 Windows 平台的窗口激活、键盘路由、注入链路上调试成本过高，且本机 Tauri 工具链复杂。决定**完全弃用 Tauri**，使用 **Python 3.11+ / PyQt6** 重写为纯 Python 桌面应用。

**核心目标：**
- 功能与现 Tauri 版**完全对齐**（抽屉、编辑、预览、设置、热键、注入、SQLite、收藏夹、标签、站点、JSON 导入导出）。
- 视觉风格**保持一致**：浅/深主题、圆角、阴影、间距 token、JetBrains Mono、抽屉 400×720 无边框。
- 全程纯 Python，无 Web 前端、无 Node、无 Rust。

---

## 1. 技术栈

| 维度 | 选型 | 理由 |
|------|------|------|
| GUI | **PyQt6** (≥ 6.7) | 信号槽 / QSS / QFrame 满足无边框、自定义渲染 |
| Markdown | `markdown-it-py` + `Pygments` → QTextBrowser | 纯 Python，可用 QSS 控制 |
| 数据库 | `sqlite3`（标准库） | 复用 Tauri 版表结构 |
| 全局热键 | `pynput.keyboard.GlobalHotKeys` + 退化 `keyboard` | Win/macOS 通用 |
| 键盘注入 | `pynput.keyboard.Controller` + 平台层 | 模拟 Ctrl+V / Cmd+V |
| 剪贴板 | `QGuiApplication.clipboard()` | 原生 |
| 前台窗口（Win） | `pywin32` + `ctypes` (`SetForegroundWindow` / `AttachThreadInput` / `GetForegroundWindow` / `GetWindowThreadProcessId` / `EnumWindows`) | 复刻 Rust 平台层 |
| 前台窗口（macOS） | `pyobjc-framework-Cocoa` (`NSWorkspace.frontmostApplication`, `NSRunningApplication.activate`) + `AXIsProcessTrusted` (`pyobjc-framework-ApplicationServices`) | 复刻 Rust 平台层 |
| 自启动 | Win: 注册表 `Run`；macOS: `LaunchAgent` plist | 自实现，无三方包 |
| 打包 | `PyInstaller`（onedir + `--windowed`） | 跨平台 |
| 日志 | `logging` + `RotatingFileHandler` | 写入 `%APPDATA%/PromptCast/logs/` |
| 包管理 | `uv` 或 `pip` + `pyproject.toml` | 简洁 |

---

## 2. 目录结构（新增 `app/`，保留旧文件直到删除任务）

```
promptcast/
├── app/
│   ├── __init__.py
│   ├── __main__.py              # 入口：python -m app
│   ├── main.py                  # QApplication 引导
│   ├── config.py                # 路径常量（数据目录、日志目录）
│   ├── db/
│   │   ├── __init__.py
│   │   ├── connection.py        # sqlite3 连接、外键、WAL
│   │   ├── migrations.py        # schema 版本号 + DDL
│   │   └── repositories/
│   │       ├── prompts.py
│   │       ├── folders.py
│   │       ├── tags.py
│   │       ├── sites.py
│   │       └── settings.py
│   ├── models/                  # @dataclass：Prompt / Folder / Tag / Site / Settings
│   ├── services/
│   │   ├── inject.py            # 注入流程编排
│   │   ├── hotkey.py            # GlobalHotKeys 注册/反注册
│   │   ├── favicon.py           # 异步抓 favicon
│   │   ├── importer.py          # JSON 导入导出
│   │   └── theme.py             # light/dark/system + QSS 加载
│   ├── platform/
│   │   ├── __init__.py          # get_platform() 工厂
│   │   ├── base.py              # ABC：foreground_pid/hwnd, activate_app, simulate_paste, check_accessibility
│   │   ├── windows.py
│   │   └── macos.py
│   ├── ui/
│   │   ├── windows/
│   │   │   ├── drawer.py        # 主抽屉窗口（无边框、置顶、400×720）
│   │   │   ├── editor.py        # 编辑器窗口
│   │   │   ├── preview.py       # 预览窗口
│   │   │   └── settings.py      # 设置窗口（左侧导航 + tab）
│   │   ├── widgets/
│   │   │   ├── search_bar.py
│   │   │   ├── filter_chips.py
│   │   │   ├── prompt_list.py
│   │   │   ├── prompt_list_item.py
│   │   │   ├── site_launcher.py
│   │   │   ├── hint_bar.py
│   │   │   ├── markdown_view.py
│   │   │   ├── hotkey_recorder.py
│   │   │   ├── folders_panel.py
│   │   │   ├── sites_panel.py
│   │   │   ├── data_panel.py
│   │   │   └── toast.py
│   │   └── styles/
│   │       ├── tokens.qss       # CSS 变量 → QSS 占位
│   │       ├── theme_light.qss
│   │       ├── theme_dark.qss
│   │       └── base.qss
│   └── tray.py                  # 系统托盘
├── assets/                      # icon、字体、内置 favicon
├── pyproject.toml
├── requirements.txt
└── docs/
    ├── PYTHON_REWRITE_PLAN.md   # 本文件
    └── PYTHON_REWRITE_TODO.md
```

**删除：** `src/`、`src-tauri/`、`index.html`、`vite.config.ts`、`package.json`、`pnpm-lock.yaml`、`tsconfig*.json`、`node_modules/`、`dist/`、`prototype/`（确认后）。

---

## 3. 关键实现要点

### 3.1 抽屉窗口（`drawer.py`）
- `QWidget` + `Qt.WindowType.FramelessWindowHint | Qt.WindowType.Tool | Qt.WindowType.WindowStaysOnTopHint`
- `setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground, True)`
- 自绘圆角容器（`QFrame` + QSS `border-radius`）
- 显示前：调用 `platform.capture_foreground()` → 记录 PID/HWND
- 隐藏触发：失焦事件 + Esc + 点击外部（除非 pin）
- 居中：`QScreen.availableGeometry()`

### 3.2 全局热键
- 主线程启动 `pynput.keyboard.GlobalHotKeys({hk_str: callback})`
- 回调通过 `QMetaObject.invokeMethod` 跨线程到 GUI
- 重新注册时先 `stop()` 再新建实例

### 3.3 注入流程（保持与 Rust 版一致）
1. 备份当前剪贴板
2. 写入 prompt 内容到剪贴板
3. 检查权限（macOS：`AXIsProcessTrusted`；Win：返回 True）
4. 隐藏抽屉
5. `time.sleep(0.1)`
6. `platform.activate_previous(pid, hwnd)`
7. `time.sleep(0.12)`
8. `pynput.Controller().tap(Key.cmd/ctrl + 'v')`
9. 600ms 后异步线程恢复原剪贴板（仅当当前剪贴板还是我们写入的内容）
10. 调用 `prompts.record_use(id)`

### 3.4 QSS 主题映射
- 把 `src/styles/tokens.css` 里的 `--space-*`、`--fs-*`、`--color-*` 翻译成 Python 字典
- 在 `theme.py` 里 `theme_light.qss.format(**TOKENS_LIGHT)` 渲染最终 QSS
- 通过 `QApplication.setStyleSheet()` 全局应用，主题切换时整体重新设置

### 3.5 SQLite 复用
- 表结构与 Rust 版完全一致（prompts / folders / tags / prompt_tags / settings / sites）
- `migrations.py` 维护 `schema_version` 表，从空库初始化
- 数据目录：Win `%APPDATA%/PromptCast/promptcast.db`，macOS `~/Library/Application Support/PromptCast/promptcast.db`
- **不**强求迁移老数据（Rust 版数据库可手动 copy 复用，结构相同）

### 3.6 键盘快捷键（抽屉内）
直接使用 `QShortcut`，列表与 Vue 版完全一致：↑/↓、Enter、Ctrl/Cmd+C/E/N/F、Space、Tab/Shift+Tab、Esc。

---

## 4. 里程碑

| M | 内容 | 完成标志 |
|---|------|---------|
| **M0** | 脚手架 | `python -m app` 弹空 QMainWindow，pyproject + 依赖装好 |
| **M1** | 数据层 | repositories 单测通过，能 CRUD prompts/folders/tags/sites/settings |
| **M2** | 抽屉 UI 静态 | 无边框 400×720 抽屉，搜索框 + 列表 + 筛选 chips + 站点行 + 提示栏渲染（假数据） |
| **M3** | 主题与样式 | light/dark QSS 切换正常，与 Tauri 版肉眼一致 |
| **M4** | 编辑/预览/设置三窗口 | 7 个 tab 可切换，能 CRUD folder/site，能录制热键 |
| **M5** | 全局热键 + 平台层 | 热键唤起抽屉，Win 平台能记录前台窗口并激活回去 |
| **M6** | 注入链路 | Enter 后内容能粘贴到目标窗口，剪贴板恢复正常 |
| **M7** | 数据导入导出 + 自启动 + 托盘 | JSON 双向，开机自启可勾选 |
| **M8** | macOS 平台层 | macOS 上等价行为（仅当用户在 macOS 测试时） |
| **M9** | 打包发布 | PyInstaller 产物可双击运行 |
| **M10** | 删除 Tauri/前端遗留代码 | 仓库清干净 |

---

## 5. 风险与对策

| 风险 | 对策 |
|------|------|
| PyQt6 无边框窗口在 Win11 失去阴影 | 用 `QGraphicsDropShadowEffect` 给容器加阴影，或留 8px 透明边距 |
| `pynput` 全局热键和 GUI 线程冲突 | 回调里只发信号，不直接动 UI |
| Win 上 `SetForegroundWindow` 失败 | 复刻 Rust 版的 `AttachThreadInput` 双线程 trick |
| Markdown 渲染样式 | QTextBrowser + 自定义 QSS，代码块用 Pygments inline 样式 |
| favicon 抓取阻塞 | `QThreadPool` + `QRunnable` 异步 |
| 打包后 SQLite/字体路径 | 用 `sys._MEIPASS` 兼容 PyInstaller |

---

## 6. 不做的事
- 不保留任何 Tauri / Vue / Node 代码
- 不引入 Web 视图（QWebEngineView 体积太大）
- 不做插件系统、不做云同步、不做多语言（首版仅中文 UI）
- 不做 Linux 支持（首版仅 Win + macOS）
