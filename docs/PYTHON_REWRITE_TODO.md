# PromptCast Python 重构 — TODO

> 与 `PYTHON_REWRITE_PLAN.md` 配套。每完成一项把 `[ ]` 改成 `[x]`。
> 顺序按里程碑，里程碑内可并行。

---

## M0 — 脚手架
- [ ] 创建 `pyproject.toml`（项目元数据、Python ≥ 3.11）
- [ ] 写 `requirements.txt`：PyQt6、pynput、markdown-it-py、Pygments、pywin32（仅 win32 平台）、pyobjc-framework-Cocoa（仅 darwin）、pyobjc-framework-ApplicationServices（仅 darwin）
- [ ] 创建目录骨架（`app/`、`app/db/`、`app/db/repositories/`、`app/services/`、`app/platform/`、`app/ui/windows/`、`app/ui/widgets/`、`app/ui/styles/`、`app/models/`、`assets/`）
- [ ] `app/__main__.py` + `app/main.py`：能 `python -m app` 弹一个空 QMainWindow
- [ ] `app/config.py`：定义 `DATA_DIR`、`LOG_DIR`、`DB_PATH`，自动创建目录
- [ ] 配置 logging（RotatingFileHandler，输出到 `LOG_DIR/app.log`）
- [ ] 把 `assets/` 放入 logo / 托盘图标占位

## M1 — 数据层
- [ ] `app/db/connection.py`：`get_conn()`，开启 `PRAGMA foreign_keys=ON`、`journal_mode=WAL`
- [ ] `app/db/migrations.py`：`schema_version` 表 + 初始 DDL（prompts/folders/tags/prompt_tags/settings/sites + 索引），照搬 Rust 版字段
- [ ] `app/models/`：dataclass Prompt / Folder / Tag / Site / Settings
- [ ] `repositories/prompts.py`：list(sort_mode), get, create, update, delete, toggle_favorite, toggle_pin, record_use
- [ ] `repositories/folders.py`：list, create, rename, delete, reorder
- [ ] `repositories/tags.py`：list, create, rename, delete
- [ ] `repositories/sites.py`：list, create, update, delete, reorder, set_favicon
- [ ] `repositories/settings.py`：get_all, get(key), set(key, value)
- [ ] 写少量 pytest（至少 prompts CRUD + sort_mode）

## M2 — 抽屉 UI 静态
- [ ] `ui/windows/drawer.py`：FramelessWindowHint + Tool + StaysOnTop + WA_TranslucentBackground，固定 400×720，居中
- [ ] 自绘圆角容器 QFrame，外层留透明边距给阴影
- [ ] `widgets/search_bar.py`：QLineEdit + 占位文字 + 清除按钮
- [ ] `widgets/filter_chips.py`：横向滚动的 chip 列表，支持选中态
- [ ] `widgets/prompt_list.py`：QListView + 自定义 delegate（或 QListWidget 占位）
- [ ] `widgets/prompt_list_item.py`：标题 + 1 行预览 + 星标
- [ ] `widgets/site_launcher.py`：横向 favicon 行 + 加号按钮
- [ ] `widgets/hint_bar.py`：底部快捷键说明栏
- [ ] 用假数据 wire 起来，能在抽屉里看到完整布局

## M3 — 主题与样式
- [ ] `ui/styles/tokens.py`：把 CSS variable 翻译成 LIGHT_TOKENS / DARK_TOKENS dict
- [ ] `ui/styles/base.qss`、`theme_light.qss`、`theme_dark.qss`：基础控件样式
- [ ] `services/theme.py`：load_qss(theme) → 渲染并 setStyleSheet
- [ ] 监听系统主题（Win：注册表；macOS：NSDistributedNotificationCenter；首版仅在切换时拉一次也可）
- [ ] 设置 JetBrains Mono（assets 里嵌入字体或 fallback）
- [ ] 给抽屉容器加 QGraphicsDropShadowEffect

## M4 — 编辑 / 预览 / 设置窗口
- [ ] `ui/windows/editor.py`：标题输入 + Markdown 文本框 + folder picker + tag 选择 + 保存/取消
- [ ] `ui/windows/preview.py`：QTextBrowser + Markdown → HTML（markdown-it-py + Pygments），底部"注入"/"复制"按钮
- [ ] `widgets/markdown_view.py`：封装渲染逻辑
- [ ] `ui/windows/settings.py`：左侧导航 7 项（通用、热键、主题、文件夹、站点、数据、关于），右侧 stack
- [ ] `widgets/hotkey_recorder.py`：捕获 keyPress，组合 modifier，输出 "CmdOrCtrl+Shift+P" 字符串 + 显示 "⌘ ⇧ P"
- [ ] `widgets/folders_panel.py`：QListWidget + 增删改 + 拖拽 reorder
- [ ] `widgets/sites_panel.py`：站点 CRUD + 触发 favicon 抓取
- [ ] `widgets/data_panel.py`：导出 / 导入 JSON 按钮
- [ ] About 面板：版本 + 打开日志目录按钮
- [ ] 三窗口为单例，复用同一实例

## M5 — 全局热键 + 平台层
- [ ] `platform/base.py`：`Platform` 抽象基类（capture_foreground / activate / simulate_paste / check_accessibility / request_accessibility）
- [ ] `platform/__init__.py`：`get_platform()` 工厂
- [ ] `platform/windows.py`：
  - [ ] `GetForegroundWindow` / `GetWindowThreadProcessId` 捕获前台
  - [ ] `SetForegroundWindow` + `AttachThreadInput` 激活
  - [ ] `EnumWindows` 按 PID 兜底
  - [ ] `ShowWindow(SW_RESTORE)` 解最小化
  - [ ] `simulate_paste`：pynput Ctrl+V
- [ ] `services/hotkey.py`：pynput GlobalHotKeys，回调用 Qt signal 跨线程
- [ ] 把"按下热键 → 抽屉显示/隐藏 + 抓取前台 PID"接通
- [ ] 抽屉失焦/Esc 自动隐藏（pin 状态除外）

## M6 — 注入链路
- [ ] `services/inject.py`：编排剪贴板备份 → 写入 → 隐藏抽屉 → sleep → activate → sleep → 模拟粘贴 → 异步恢复剪贴板
- [ ] 接 Enter 快捷键到 `inject.run()`
- [ ] 实现"仅复制"模式（Ctrl/Cmd+C）
- [ ] 调用 `prompts_record_use` 更新计数和 last_used_at
- [ ] 失败时通过 toast 提示降级
- [ ] `widgets/toast.py`：抽屉内浮层提示

## M7 — 收尾功能
- [ ] `services/importer.py`：JSON 导入 / 导出（merge/replace 模式）
- [ ] 自启动：Win 写注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`；macOS 写 LaunchAgent plist
- [ ] `app/tray.py`：QSystemTrayIcon + 菜单（显示抽屉 / 设置 / 退出）
- [ ] `services/favicon.py`：QThreadPool 异步抓取 + 写 BLOB + 30 天过期
- [ ] 全部 settings key 接到 settings 仓库

## M8 — macOS 平台层（仅当 macOS 上测试）
- [ ] `platform/macos.py`：
  - [ ] `NSWorkspace.frontmostApplication` 抓 PID
  - [ ] `NSRunningApplication.runningApplicationWithProcessIdentifier_().activateWithOptions_()`
  - [ ] `AXIsProcessTrusted` 权限检查
  - [ ] 触发系统弹辅助功能授权
  - [ ] simulate_paste：pynput Cmd+V
- [ ] App 设置 `LSUIElement` 等价行为：`NSApp.setActivationPolicy_(NSApplicationActivationPolicyAccessory)`

## M9 — 打包
- [ ] PyInstaller spec：onedir + windowed，带入 assets / qss / 字体
- [ ] Win：生成 .exe + 测试在干净机器双击运行
- [ ] macOS：生成 .app（如需）
- [ ] 写 README 启动 / 打包说明

## M10 — 清理
- [ ] 删除 `src/`、`src-tauri/`、`prototype/`、`dist/`、`node_modules/`
- [ ] 删除 `index.html`、`vite.config.ts`、`package.json`、`pnpm-lock.yaml`、`tsconfig.json`、`tsconfig.node.json`
- [ ] 更新根 `README.md`：改为 Python 启动说明
- [ ] 提 PR / 合并到 main
