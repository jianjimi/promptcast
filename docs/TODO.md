# 实施 TODO（按里程碑）

> 每条 todo 必须有可验收的"完成判据"。AI 协作时**先勾选目标，再写代码**。

---

## M0 · 项目骨架（半天）

- [ ] **M0-1** 在 `~/Desktop/提示词工具/` 下用 `npm create tauri-app@latest` 生成 Tauri 工程，命名 `app`。前端选 Vue + TypeScript。生成后将其内容**移到当前目录根**（保留 `docs/`、`prototype/`、`.git/`、`.gitignore`）。
  - 验收：`pnpm tauri dev` 启动并显示默认欢迎页
- [ ] **M0-2** 拆出 `src/views/{DrawerView,PreviewView,EditorView,SettingsView}.vue` 四个空视图，配 vue-router 路由 `/drawer`、`/preview/:id`、`/editor/:id?`、`/settings`
  - 验收：手动改 URL hash 能切换四个视图
- [ ] **M0-3** 在 `src-tauri/tauri.conf.json` 配置四个 Webview 窗口（label: `drawer/preview/editor/settings`），各自加载对应路由 hash
  - 验收：`tauri dev` 后能用 `WebviewWindow` API 各自打开
- [ ] **M0-4** 引入 Pinia 选项式语法骨架（`stores/{prompts,folders,tags,settings,ui}.ts`），仅写空 state 与 TODO 注释
- [ ] **M0-5** 写 `src/styles/{tokens,theme-light,theme-dark,reset,global}.css`，定义颜色/间距/字体的 CSS 变量；`useTheme.ts` 监听系统主题切换 `data-theme`
  - 验收：页面背景色能跟随系统深浅色切换
- [ ] **M0-6** 写 `src/types/*.ts` 与 `src-tauri/src/models/*.rs`，类型字段一一对应（serde rename_all = "snake_case"）
- [ ] **M0-7** Git commit：`[M0] project skeleton`

---

## M1 · 数据层（1 天）

- [ ] **M1-1** Rust 端引入 `rusqlite` + `r2d2_sqlite`，写 `db/mod.rs` 连接管理（数据库路径用 `app_data_dir`）
- [ ] **M1-2** 写 `db/schema.rs` 与 `db/migrations.rs`，建表 SQL 同 PLAN §5；`migrations` 用 `user_version` 简单版本化
  - 验收：首次运行自动建表；二次运行不重复
- [ ] **M1-3** 写 `db/prompts.rs`：`list / get / create / update / delete / toggle_favorite / toggle_pin / record_use`
  - 验收：单元测试覆盖每个 fn（用 `:memory:` 数据库）
- [ ] **M1-4** 写 `db/folders.rs`、`db/tags.rs`、`db/settings.rs` 同上
- [ ] **M1-5** 写 `commands/prompts.rs` 等，把 `db/*` 暴露为 IPC；统一 `Result<T, AppError>` 返回
- [ ] **M1-6** `src/api/*.ts` 包装 invoke，前端**只从此处**调后端
- [ ] **M1-7** `stores/prompts.ts` 实现 `loadAll / create / update / delete / toggleFavorite / togglePin`，供视图 dispatch
- [ ] **M1-8** `commands/data.rs` 实现 JSON `export` / `import`（merge / replace 两种模式）
- [ ] **M1-9** Git commit：`[M1] data layer`

---

## M2 · 抽屉 UI（2 天）

- [ ] **M2-1** `views/DrawerView.vue` 实现单列布局：标题栏 / 搜索 / 筛选 / 列表 / 底部 hint 条
- [ ] **M2-2** `components/drawer/SearchBar.vue`：搜索框 + pin 按钮 + 排序下拉 + 设置入口 + "新建"按钮（打开 EditorView 新建）
- [ ] **M2-3** `components/drawer/FilterChips.vue`：水平滚动的分类（"全部 / 收藏" + 文件夹列表 + 标签下拉）；多选时合取
- [ ] **M2-4** `components/drawer/PromptList.vue` + `PromptListItem.vue`：列表渲染、键盘 ↑↓ 选中、Enter 触发注入；置顶项独立 section 永远在顶部
- [ ] **M2-5** `composables/useFuzzySearch.ts`：Fuse.js 包装，搜索域 = title + content + tags
  - 验收：空搜索词返回原列表；输入"foo"返回评分排序结果
- [ ] **M2-6** `components/drawer/HintBar.vue`：底部快捷键提示条（↵注入 / ⌘C复制 / ⌘E编辑 / Space预览 / 计数）
- [ ] **M2-6b** `components/drawer/SiteLauncher.vue`：横向 favicon 列表，绑定 `sites_open`，末尾"+"按钮跳到设置网址快捷页
- [ ] **M2-6c** Tab / Shift+Tab 在 FilterChips 内循环切换；按住 Tab 不会跑到列表或搜索框
- [ ] **M2-7** `components/ui/BaseToast.vue` + `stores/ui.ts` toast 队列：注入失败时弹"已复制到剪贴板"
- [ ] **M2-8** 排序下拉：最近使用 / 创建时间 / 更新时间 / 标题 A-Z（持久化到 settings）
- [ ] **M2-9** 抽屉滑入/滑出动画（CSS `transform: translateX`），窗口尺寸 400×720
- [ ] **M2-10** Git commit：`[M2] drawer UI`

## M2.5 · 预览窗口（半天）

- [ ] **M2.5-1** `views/PreviewView.vue`：从抽屉按 Space 打开；通过 IPC `window_open_preview { id }` 创建窗口
- [ ] **M2.5-2** `components/preview/PreviewHeader.vue`：元数据条 + 标题 + 标签 + 收藏/置顶图标
- [ ] **M2.5-3** `components/preview/MarkdownView.vue`：复用 `utils/markdown.ts`，渲染只读 Markdown
- [ ] **M2.5-4** `components/preview/PreviewActions.vue`：底部 字数/token 统计 + "复制 / 注入"按钮
- [ ] **M2.5-5** 预览窗同样不抢焦点（与抽屉一致），按 Esc 关闭
- [ ] **M2.5-6** Git commit：`[M2.5] preview window`

---

## M2.8 · 网址数据层与抓取（半天，与 M3 并行可）

- [ ] **M2.8-1** `db/sites.rs` + `commands/sites.rs`：list/create/update/delete/reorder/refresh_favicon/open
- [ ] **M2.8-2** Rust 端用 `reqwest` 抓 favicon（先试 `/favicon.ico`，失败回退解析 HTML `<link rel="icon">`）
  - 验收：输入 `https://chat.openai.com` 能拿到正确 favicon
- [ ] **M2.8-3** `commands/sites.rs::sites_open` 调 `tauri-plugin-opener::open` 打开浏览器
- [ ] **M2.8-4** `stores/sites.ts` + `api/sites.ts` 前端封装；favicon 用 `data:` URI 渲染
- [ ] **M2.8-5** Git commit：`[M2.8] sites data layer`

## M3 · 编辑与设置窗口（1.5 天）

- [ ] **M3-1** `EditorView.vue`：标题 + Markdown textarea + 标签多选 + 文件夹下拉；右下"保存 / 取消"
  - 验收：未改动时关闭不提示；改动后关闭弹"放弃修改"确认
- [ ] **M3-2** `MarkdownField.vue`：纯 textarea + Tab 键插入两空格 + 实时字数；右上切换"编辑/预览"
- [ ] **M3-3** `SettingsView.vue` 标签页：常规 / 快捷键 / 主题 / 分类管理 / 网址快捷 / 数据 / 关于
- [ ] **M3-4** `HotkeyRecorder.vue`：捕获用户按键组合，验证非空并避免与系统冲突
- [ ] **M3-5** `DataPanel.vue`：导入 JSON（弹文件选择）、导出 JSON（弹保存对话框）、清空所有数据（二次确认）
- [ ] **M3-6** "权限诊断"区：检测辅助功能权限、检测全局快捷键注册状态；引导跳转到系统设置
- [ ] **M3-7** `components/settings/FoldersPanel.vue`：列表 + 重命名 + 删除（带迁移确认）+ 拖拽排序（`vue-draggable-plus`）
- [ ] **M3-8** `components/settings/SitesPanel.vue`：列表（favicon 缩略图 + 名称 + url）+ 增删改 + 拖拽排序 + "重新获取图标"按钮
- [ ] **M3-9** Git commit：`[M3] editor & settings windows`

---

## M4 · 全局快捷键 + 注入 + 焦点保持（2 天）

- [ ] **M4-1** 引入 `tauri-plugin-global-shortcut`，注册用户配置的快捷键，触发显示抽屉
  - 验收：从其他应用按快捷键能弹出抽屉
- [ ] **M4-2** 首次启动检测无快捷键 → 自动打开 `SettingsView` 引导设置
- [ ] **M4-3** 抽屉窗口改造为不抢焦点窗口
  - macOS：引入 `tauri-nspanel` 或写 `platform/window_macos.rs` 调 objc 改 NSWindow 为 NSPanel
  - Windows：`platform/window_windows.rs` 设 `WS_EX_NOACTIVATE`
  - 验收：从浏览器/记事本按快捷键唤起抽屉，原窗口光标仍在闪
- [ ] **M4-4** `platform/inject_macos.rs` + `inject_windows.rs`：用 `enigo` 模拟 Cmd/Ctrl+V
- [ ] **M4-5** `commands/inject.rs::inject_paste`：写剪贴板 → 隐藏窗口 → sleep 50ms → 模拟粘贴；任一步失败回退仅复制
  - 验收：在 ChatGPT、Cursor、微信、记事本里分别试一遍
- [ ] **M4-6** macOS 辅助功能权限检查：未授权时 toast 提示并打开"系统设置 > 辅助功能"
- [ ] **M4-7** 注入成功后调用 `prompts_record_use` 更新 use_count + last_used_at
- [ ] **M4-8** Git commit：`[M4] hotkey & injection`

---

## M5 · 主题打磨 + 毛玻璃 + 打包（1 天）

- [ ] **M5-1** 引入 `tauri-plugin-vibrancy`，抽屉与编辑/设置窗口启用毛玻璃（macOS HUD/sidebar，Windows Mica/Acrylic）
- [ ] **M5-2** 调色：依据 pencil 设计稿对照颜色、阴影、圆角、动画时长，更新 `theme-*.css`（**用 pencil MCP 读取设计稿，不靠目测**）
- [ ] **M5-3** 键盘可达性：所有可点击元素支持 Tab + Enter；列表 ↑↓、Esc 关闭、Cmd/Ctrl+E 编辑、Cmd/Ctrl+F 聚焦搜索框
- [ ] **M5-4** 应用图标（macOS `.icns` + Windows `.ico`），托盘图标分深浅色
- [ ] **M5-5** `tauri.conf.json` 完善 bundle 配置；`pnpm tauri build` 生成 `.dmg` 与 `.msi`
  - 验收：在干净的另一台 Mac / Win 安装能运行
- [ ] **M5-6** 写 `README.md`（用户视角：截图、安装、首次设置、常见问题）
- [ ] **M5-7** Git tag `v0.1.0` + commit：`[M5] release v0.1.0`

---

## v2（暂不进入 MVP）

- 变量占位符 `{{var}}`：解析 + 填表 UI + 注入前替换
- 云同步：自建后端 / Supabase / iCloud Documents
- 提示词模板市场：浏览公共集合并导入
- 命令面板（Cmd+K）：在抽屉内执行操作
- 多语言界面（zh / en）
- 自动更新：`tauri-plugin-updater`

---

## 当前进行中

- 写 PLAN/TODO 文档 ✅
- 用 pencil 画原型 ⏳（下一步）
- 等用户确认原型后进入 M0
