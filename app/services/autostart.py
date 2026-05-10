"""Cross-platform autostart toggle.

Windows: HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run registry value.
macOS:   ~/Library/LaunchAgents/dev.promptcast.app.plist (LaunchAgent).
"""
from __future__ import annotations

import sys
from pathlib import Path

_RUN_KEY_NAME = "PromptCast"
_LAUNCH_AGENT_LABEL = "dev.promptcast.app"


def _launch_command() -> list[str]:
    return [sys.executable, "-m", "app"]


def set_enabled(enabled: bool) -> None:
    if sys.platform == "win32":
        _set_windows(enabled)
    elif sys.platform == "darwin":
        _set_macos(enabled)
    else:
        raise NotImplementedError("autostart is only implemented on Windows and macOS")


def _set_windows(enabled: bool) -> None:
    import winreg

    key_path = r"Software\Microsoft\Windows\CurrentVersion\Run"
    with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path, 0, winreg.KEY_SET_VALUE) as key:
        if enabled:
            cmd = " ".join(f'"{p}"' for p in _launch_command())
            winreg.SetValueEx(key, _RUN_KEY_NAME, 0, winreg.REG_SZ, cmd)
        else:
            try:
                winreg.DeleteValue(key, _RUN_KEY_NAME)
            except FileNotFoundError:
                pass


def _set_macos(enabled: bool) -> None:
    plist_dir = Path.home() / "Library" / "LaunchAgents"
    plist_dir.mkdir(parents=True, exist_ok=True)
    path = plist_dir / f"{_LAUNCH_AGENT_LABEL}.plist"
    if not enabled:
        path.unlink(missing_ok=True)
        return
    args_xml = "\n        ".join(f"<string>{a}</string>" for a in _launch_command())
    path.write_text(
        f"""<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\"><dict>
    <key>Label</key><string>{_LAUNCH_AGENT_LABEL}</string>
    <key>RunAtLoad</key><true/>
    <key>ProgramArguments</key><array>
        {args_xml}
    </array>
</dict></plist>
""",
        encoding="utf-8",
    )
