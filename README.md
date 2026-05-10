# PromptCast

Keyboard-driven prompt drawer. Press a global hotkey, pick a prompt, hit Enter — it gets pasted straight into the app you were just using.

Pure Python + PyQt6. Windows + macOS.

> Mirror repo: https://cnb.cool/liuxiaogang/promptcast

## Features

- 🪟 Frameless **400×720 drawer**, summoned by a global hotkey, dismissed on Esc / focus loss
- 🔍 Fuzzy search, folder & tag filter chips, ★ favorites, pinning
- ✏️ Markdown editor + preview window (markdown-it-py + Pygments)
- 📋 Smart **paste injection** — restores foreground app and synthesizes Ctrl/⌘+V; copy-only fallback when blocked
- 🌓 Light / Dark / System theme (designed against a single token palette)
- 🔗 Site launcher row with auto-fetched favicons
- 💾 SQLite persistence + JSON import/export (merge or replace)
- 🚀 Optional auto-start (Windows registry / macOS LaunchAgent)
- 📌 System tray for quick access

## Requirements

- Python **3.11+**
- Windows 10+ or macOS 11+

## Install & run

```bash
# create a virtualenv (recommended)
python -m venv .venv
. .venv/Scripts/activate          # Windows
# source .venv/bin/activate       # macOS

pip install -r requirements.txt
python -m app
```

Default global hotkey: `Ctrl + Shift + Space` (Win) / `⌃ ⇧ Space` (mac). Change it in **Settings → 热键**.

## Project layout

```
app/
├── main.py               # entry: ensure dirs, init theme, install tray, start controller
├── controller.py         # wires drawer + companion windows + services
├── bootstrap.py          # first-run sample data
├── config.py             # APPDATA paths
├── logging_setup.py
├── tray.py               # QSystemTrayIcon
├── db/
│   ├── connection.py     # sqlite3 + WAL + foreign keys
│   ├── migrations.py     # PRAGMA user_version schema
│   └── repositories/     # prompts/folders/tags/sites/settings
├── models/               # @dataclass entities
├── services/
│   ├── theme.py          # QSS rendering + system-scheme tracking
│   ├── hotkey.py         # pynput → Qt signal bridge
│   ├── inject.py         # clipboard + activate + paste pipeline
│   ├── favicon.py        # async favicon fetch
│   ├── importer.py       # JSON import/export
│   └── autostart.py      # Win registry / macOS LaunchAgent
├── platform/             # foreground capture + activation
│   ├── base.py
│   ├── windows.py        # SetForegroundWindow + AttachThreadInput
│   └── macos.py          # NSWorkspace + NSRunningApplication + AXIsProcessTrusted
└── ui/
    ├── windows/          # drawer / editor / preview / settings
    ├── widgets/          # search bar, chips, prompt list, hint bar, ...
    └── styles/           # tokens.py + base.qss template
docs/
├── PYTHON_REWRITE_PLAN.md
└── PYTHON_REWRITE_TODO.md
```

## Tests

```bash
pip install pytest
python -m pytest -q
```

## Packaging

```bash
pip install pyinstaller
pyinstaller --noconsole --name PromptCast --onedir -p . app/__main__.py
```

## Hotkey shortcuts (drawer focused)

| Key | Action |
|---|---|
| `↑ / ↓` | move selection |
| `Enter` | inject into target app |
| `Ctrl/⌘ + C` | copy only |
| `Ctrl/⌘ + E` | edit selected |
| `Ctrl/⌘ + N` | new prompt |
| `Ctrl/⌘ + F` | focus search |
| `Tab / Shift+Tab` | cycle filter chips |
| `Space` | preview |
| `Esc` | clear search → hide drawer |

## License

MIT
