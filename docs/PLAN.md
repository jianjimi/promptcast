# 提示词管理工具 — 架构与实施计划

> 本文件是项目的"宪法"，所有 AI 协作（Claude / Gemini / Codex）在生成或修改代码前都应先阅读此文档与 `TODO.md`。

## 1. 产品定义

一个常驻后台的跨平台桌面工具（macOS + Windows），用全局快捷键唤起侧边抽屉，让用户快速选中预设的提示词并**自动注入**到上一个聚焦的输入框（注入失败时静默回退到剪贴板复制）。

**核心使用闭环**：按快捷键 → 抽屉滑入 → 搜索/筛选 → Enter → 注入到目标软件 → 抽屉隐藏。

## 2. 非目标 (Non-Goals)

- ❌ 云端同步（MVP 范围内只做本地存储 + JSON 导入导出）
- ❌ 多用户/团队协作
- ❌ 提示词市场或社区分享
- ❌ AI 能力集成（不调用任何大模型 API）
- ❌ 移动端
- ❌ 变量占位符（`{{var}}`）— 列入 v2

## 3. 技术栈

| 层 | 选型 | 备注 |
|---|---|---|
| 桌面壳 | **Tauri 2** | 体积小、启动快、原生 WebView |
| 前端框架 | **Vue 3 + 选项式 API** | 用户偏好；`<script>` 用 Options API，**不用 `<script setup>` Composition** |
| 语言 | TypeScript（前端） + Rust（Tauri 后端） | TS 仅做类型约束，不引入复杂泛型 |
| 样式 | **传统 CSS + CSS 变量**，按组件 scoped | **不使用 Tailwind / UnoCSS**，主题靠 CSS 变量切换 |
| 数据存储 | **SQLite** (rusqlite)  | 全量加载到内存（数据 <100 条），前端做模糊搜索 |
| 模糊搜索 | **Fuse.js** | 仅前端使用 |
| 全局快捷键 | `tauri-plugin-global-shortcut` | 用户首次启动自定义 |
| 键盘注入 | `enigo` (Rust) | 模拟 Cmd/Ctrl+V，剪贴板用 `arboard` |
| 窗口非激活 | macOS NSPanel + Win `WS_EX_NOACTIVATE` | 通过 `tauri-nspanel` 或自写 Rust 调系统 API |
| 包管理 | **pnpm** | |

### 关键技术决策

- **不用 Composition API**：用户表达过排斥。所有 Vue 组件使用 `data() / methods / computed / watch / props / emits` 选项式写法。
- **不用 Tailwind**：用普通 CSS + `<style scoped>`，主题完全靠 CSS 变量驱动。
- **数据规模假设**：< 100 条提示词，因此前端全量加载、JS 模糊搜索即可，不引入后端 FTS5。但**存储层抽象保留扩展空间**。
- **样式还原**：UI 设计稿存放于 `prototype/app.pen`，**实施阶段也通过 pencil MCP 读取设计稿**以保证还原度，不靠肉眼模仿截图。

## 4. 目录结构

```
提示词工具/
├── docs/                    设计与协作文档（本目录）
│   ├── PLAN.md              架构总览（本文件）
│   ├── TODO.md              分阶段任务清单
│   ├── DATA_MODEL.md        数据库 schema 与 IPC 契约（M1 后产出）
│   └── DEV_NOTES.md         开发踩坑与决策日志（持续更新）
│
├── prototype/
│   └── app.pen              pencil 原型源文件（设计稿真源）
│
├── src/                     前端 Vue 应用（Tauri WebView 加载）
│   ├── main.ts              入口：Vue 实例化、路由、全局样式注入
│   ├── App.vue              路由 outlet（极薄）
│   │
│   ├── views/               顶层视图（每个对应一个 Tauri 窗口）
│   │   ├── DrawerView.vue       侧边抽屉主窗口
│   │   ├── EditorView.vue       新建/编辑独立窗口
│   │   └── SettingsView.vue     设置独立窗口
│   │
│   ├── components/          可复用组件（每个 ≤ 250 行）
│   │   ├── drawer/
│   │   │   ├── SearchBar.vue        顶部搜索框 + pin 按钮
│   │   │   ├── FilterChips.vue      分类/标签筛选
│   │   │   ├── PromptList.vue       中间列表（虚拟滚动可选）
│   │   │   ├── PromptListItem.vue   单条列表项（标题+片段+收藏星）
│   │   │   └── PromptDetail.vue     右侧详情（只读 Markdown 渲染）
│   │   ├── editor/
│   │   │   ├── TitleField.vue
│   │   │   └── MarkdownField.vue    Markdown 编辑（轻量 textarea + 预览切换）
│   │   ├── settings/
│   │   │   ├── HotkeyRecorder.vue   快捷键录制控件
│   │   │   ├── ThemeSelector.vue
│   │   │   └── DataPanel.vue        导入/导出 JSON
│   │   └── ui/                  基础控件
│   │       ├── BaseButton.vue
│   │       ├── BaseInput.vue
│   │       ├── BaseModal.vue
│   │       ├── BaseTag.vue
│   │       └── BaseToast.vue
│   │
│   ├── stores/              状态管理（Pinia 选项式）
│   │   ├── prompts.ts           提示词列表、CRUD、搜索结果
│   │   ├── folders.ts
│   │   ├── tags.ts
│   │   ├── settings.ts
│   │   └── ui.ts                抽屉显隐、当前选中、toast
│   │
│   ├── api/                 IPC 包装层（前端唯一调用 Tauri 的入口）
│   │   ├── index.ts             统一导出
│   │   ├── prompts.ts           prompt_list/get/create/update/delete
│   │   ├── folders.ts
│   │   ├── tags.ts
│   │   ├── settings.ts
│   │   ├── inject.ts            注入与剪贴板
│   │   └── window.ts            窗口控制（show/hide/pin）
│   │
│   ├── composables/         可复用逻辑（虽用选项式，仍可写函数）
│   │   ├── useFuzzySearch.ts    Fuse.js 包装
│   │   ├── useKeyboardNav.ts    上下键 + Enter 列表导航
│   │   └── useTheme.ts          监听系统主题、切换 CSS 变量
│   │
│   ├── types/               TS 类型（与 Rust 端镜像对齐）
│   │   ├── prompt.ts
│   │   ├── folder.ts
│   │   ├── tag.ts
│   │   └── settings.ts
│   │
│   ├── styles/
│   │   ├── tokens.css           CSS 变量（颜色、间距、字体、动画）
│   │   ├── theme-light.css      浅色变量值
│   │   ├── theme-dark.css       深色变量值
│   │   ├── reset.css
│   │   └── global.css           全局基础样式（极薄）
│   │
│   └── utils/
│       ├── debounce.ts
│       ├── format.ts            日期、字数等
│       └── markdown.ts          Markdown 渲染（marked + DOMPurify）
│
└── src-tauri/               Rust 后端（Tauri）
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    └── src/
        ├── main.rs              入口：注册插件、setup、命令
        ├── lib.rs               run() 入口（保持薄）
        │
        ├── commands/            IPC 命令处理（每个命令一个 fn）
        │   ├── mod.rs
        │   ├── prompts.rs
        │   ├── folders.rs
        │   ├── tags.rs
        │   ├── settings.rs
        │   ├── inject.rs        剪贴板写入 + 模拟粘贴
        │   └── window.rs        窗口显隐、pin、首次定位
        │
        ├── db/                  数据访问层（仓储模式）
        │   ├── mod.rs           连接管理 (r2d2)
        │   ├── schema.rs        建表 SQL
        │   ├── migrations.rs    版本化迁移
        │   ├── prompts.rs       SQL CRUD
        │   ├── folders.rs
        │   ├── tags.rs
        │   └── settings.rs
        │
        ├── models/              领域模型（serde 序列化）
        │   ├── mod.rs
        │   ├── prompt.rs
        │   ├── folder.rs
        │   ├── tag.rs
        │   └── settings.rs
        │
        ├── platform/            跨平台封装（条件编译）
        │   ├── mod.rs
        │   ├── inject_macos.rs  enigo + AX 权限检查
        │   ├── inject_windows.rs
        │   ├── window_macos.rs  NSPanel 改造
        │   └── window_windows.rs
        │
        └── error.rs             统一错误类型 (thiserror)
```

**单文件硬上限：500 行，目标 ≤ 400 行**。超出即拆分子模块。

## 5. 数据模型（SQLite）

```sql
CREATE TABLE prompts (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT NOT NULL,
  content      TEXT NOT NULL,
  folder_id    INTEGER REFERENCES folders(id) ON DELETE SET NULL,
  is_favorite  INTEGER NOT NULL DEFAULT 0,  -- 收藏（可单独筛选）
  is_pinned    INTEGER NOT NULL DEFAULT 0,  -- 置顶（永远列表最顶）
  use_count    INTEGER NOT NULL DEFAULT 0,
  last_used_at INTEGER,                     -- Unix ms
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

CREATE TABLE folders (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL UNIQUE,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);

CREATE TABLE tags (
  id     INTEGER PRIMARY KEY AUTOINCREMENT,
  name   TEXT NOT NULL UNIQUE,
  color  TEXT
);

CREATE TABLE prompt_tags (
  prompt_id INTEGER NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  tag_id    INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
  PRIMARY KEY (prompt_id, tag_id)
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE INDEX idx_prompts_folder    ON prompts(folder_id);
CREATE INDEX idx_prompts_lastused  ON prompts(last_used_at);
CREATE INDEX idx_prompts_pinned    ON prompts(is_pinned, is_favorite);
```

**`is_favorite` 与 `is_pinned` 是两个独立字段**：
- `is_favorite`：用户标星，可作为侧栏过滤项"只看收藏"
- `is_pinned`：列表永远顶置，不参与排序模式

## 6. 模块边界

```
┌──────────────────────────── 前端 (Vue) ────────────────────────────┐
│  views → components → stores → api → [Tauri IPC]                  │
│                          ↑                                         │
│                          └── composables / utils / types           │
└────────────────────────────────────────────────────────────────────┘
                                   ↕ (invoke + emit)
┌──────────────────────────── 后端 (Rust) ───────────────────────────┐
│  commands → db / platform → models                                │
│                          ↑                                         │
│                          └── error                                 │
└────────────────────────────────────────────────────────────────────┘
```

**铁律**：
1. Vue 组件**只**通过 `src/api/*` 调用后端，不允许直接 `invoke`
2. Pinia store **只**通过 `src/api/*` 拿数据，不持有业务逻辑细节
3. Rust `commands/*` 只做参数校验和编排，业务在 `db/` 或 `platform/`
4. 跨平台代码**必须**进 `platform/`，用 `#[cfg(target_os = "...")]` 隔离

## 7. IPC 命令清单（v0 草案）

| 命令 | 入参 | 返回 | 说明 |
|---|---|---|---|
| `prompts_list` | `{ folder_id?, tag_id?, sort: 'recent_used'\|'created'\|'updated'\|'title' }` | `Prompt[]` | 全量返回，前端做模糊搜索 |
| `prompts_get` | `{ id }` | `Prompt` | |
| `prompts_create` | `{ title, content, folder_id?, tag_ids[] }` | `Prompt` | |
| `prompts_update` | `{ id, ...patch }` | `Prompt` | |
| `prompts_delete` | `{ id }` | `void` | |
| `prompts_toggle_favorite` | `{ id }` | `Prompt` | |
| `prompts_toggle_pin` | `{ id }` | `Prompt` | |
| `prompts_record_use` | `{ id }` | `void` | 注入/复制成功后调用，更新 use_count + last_used_at |
| `folders_list` / `_create` / `_rename` / `_delete` | | | |
| `tags_list` / `_create` / `_rename` / `_delete` | | | |
| `settings_get_all` / `settings_set` | | | |
| `inject_paste` | `{ content }` | `{ ok: bool, fallback: 'clipboard'\|null }` | 写剪贴板 + 隐藏窗口 + 模拟粘贴 |
| `inject_copy_only` | `{ content }` | `void` | |
| `window_show_drawer` / `_hide` / `_set_pin` | | | |
| `permissions_check_accessibility` | | `bool` | macOS 辅助功能权限 |
| `data_export_json` | | `string` (json) | |
| `data_import_json` | `{ json, mode: 'merge'\|'replace' }` | `{ inserted, updated }` | |

## 8. 关键技术点

### 8.1 焦点保持（不抢焦点）
- **macOS**：通过 `tauri-nspanel` 或在 `setup()` 里调 objc 把 NSWindow 转为 NSPanel + `NSWindowStyleMaskNonactivatingPanel`
- **Windows**：`SetWindowLongPtr(hwnd, GWL_EXSTYLE, WS_EX_NOACTIVATE)`
- **配套**：用户选中后立即调 `window_hide_drawer` → 等 ~50ms → 触发模拟粘贴

### 8.2 注入流程（`inject_paste`）
1. `arboard::Clipboard::set_text(content)`
2. 触发当前主窗口隐藏并失去焦点（`hide` + `set_focus(false)`）
3. 等待 50ms 让系统切换焦点回到原窗口
4. macOS 用 `enigo` 模拟 `Cmd+V`；Windows 模拟 `Ctrl+V`
5. 返回 `{ ok: true }`；任一步失败仅写剪贴板，返回 `{ ok: false, fallback: 'clipboard' }`

### 8.3 全局快捷键
- 首次启动空白：进入"快捷键引导"流程，弹出 `HotkeyRecorder` 录制
- 录制时通过 `tauri-plugin-global-shortcut` 注册；冲突时提示
- 持久化到 `settings.hotkey`

### 8.4 主题与毛玻璃
- HTML 根上挂 `data-theme="light|dark"`
- 监听系统外观变化（macOS `NSDistributedNotificationCenter` / Win `WM_SETTINGCHANGE`），通过 Tauri 事件广播给前端
- 毛玻璃：macOS `NSVisualEffectView`、Win `DwmEnableBlurBehindWindow` 或 `SetWindowCompositionAttribute`（acrylic）；用 `tauri-plugin-vibrancy`

### 8.5 三窗口模型
- 抽屉、编辑、设置 — 三个独立 `WebviewWindow`，共用同一 Vue bundle，路由区分
- 抽屉默认 `decorations: false, transparent: true, skip_taskbar: true, always_on_top: true(可关)`
- 编辑/设置：标准窗口，可独立打开、可同时存在

## 9. 协作与编码约定

### 9.1 文件大小（硬约束）
- 单文件 **≤ 400 行（目标）/ 500 行（硬上限）**
- 超过就拆：组件 → 按子区域拆；store → 按 action 类别拆；Rust 文件 → 按职责拆模块

### 9.2 命名
- Vue 组件：`PascalCase.vue`，文件名与组件名一致
- TS / Rust：`snake_case` 文件名，导出 `camelCase` (TS) / `snake_case` (Rust)
- IPC 命令：动词短语 `noun_verb`，如 `prompts_create`

### 9.3 AI 协作友好原则
1. **每个文件顶部 5–10 行注释**：本文件职责、依赖、对外暴露
2. **避免长函数**：单函数 ≤ 60 行，超出拆纯函数
3. **类型先行**：所有 IPC 入参/返回先在 `src/types/` 与 Rust `models/` 对齐再实现
4. **不写隐式行为**：副作用（写库、改剪贴板、隐藏窗口）只能在 `commands/` 与 `stores/` 里发生

### 9.4 Git 提交节奏
- 文档独立提交（不与代码混）
- 每个 milestone 完成做一次提交，commit message 用 `[Mx] subject` 前缀
- 小修复用 `fix:`，重构 `refactor:`，新功能 `feat:`

## 10. 风险与缓解

| 风险 | 缓解 |
|---|---|
| macOS 辅助功能权限被拒绝 → 注入不可用 | 启动检查 + 设置页提供"重新引导"；失败永远回退复制 |
| Windows 杀软误报 enigo 模拟键盘 | 准备签名证书；首发版本附文档说明 |
| 抽屉非激活窗口在不同系统版本上行为不一致 | macOS 12+ / Windows 10 1809+ 才支持完整能力，README 标注最低系统版本 |
| Rust 学习曲线 | `commands/` 只做转发，业务逻辑限制在 `db/`，AI 可独立完成 |
| 单文件膨胀 | code review 时 `wc -l` 卡线，超 400 行立即拆 |

## 11. 里程碑总览（详见 `TODO.md`）

- **M0**：项目骨架 + 三窗口空壳 + 路由
- **M1**：数据层（SQLite + 迁移 + CRUD IPC）
- **M2**：抽屉 UI 与列表/搜索/筛选
- **M3**：编辑窗口 + 设置窗口
- **M4**：全局快捷键 + 注入 + 焦点保持
- **M5**：主题打磨 + 毛玻璃 + 打包分发
- **v2（暂不进入 MVP）**：变量占位符、云同步、提示词模板市场
