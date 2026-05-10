"""Win32 platform layer — foreground tracking + activation via AttachThreadInput."""
from __future__ import annotations

import logging
import os
import time

from app.platform.base import ForegroundRef, Platform

log = logging.getLogger(__name__)

_OUR_PID = os.getpid()


def _import_win32():
    import ctypes
    import win32api  # type: ignore
    import win32con  # type: ignore
    import win32gui  # type: ignore
    import win32process  # type: ignore
    return ctypes, win32api, win32con, win32gui, win32process


class WindowsPlatform(Platform):
    def __init__(self) -> None:
        ctypes, win32api, win32con, win32gui, win32process = _import_win32()
        self._ctypes = ctypes
        self._user32 = ctypes.windll.user32
        self._kernel32 = ctypes.windll.kernel32
        self._gui = win32gui
        self._proc = win32process
        self._con = win32con
        self._api = win32api

    # ---- foreground tracking -------------------------------------------------------
    def capture_foreground(self) -> ForegroundRef:
        hwnd = self._gui.GetForegroundWindow()
        if not hwnd:
            return ForegroundRef()
        _tid, pid = self._proc.GetWindowThreadProcessId(hwnd)
        if pid == _OUR_PID:
            return ForegroundRef()  # our own window — ignore
        log.info("capture foreground: hwnd=%s pid=%s", hwnd, pid)
        return ForegroundRef(pid=pid, hwnd=int(hwnd))

    def activate(self, ref: ForegroundRef) -> bool:
        if ref.hwnd and self._activate_hwnd(ref.hwnd):
            return True
        if ref.pid:
            return self._activate_pid(ref.pid)
        return False

    # ---- activation primitives -----------------------------------------------------
    def _activate_hwnd(self, hwnd: int) -> bool:
        if not hwnd or not self._user32.IsWindow(hwnd):
            return False
        if not self._user32.IsWindowVisible(hwnd):
            log.warning("activate_hwnd: target hwnd %s not visible", hwnd)
            return False

        target_tid, _pid = self._proc.GetWindowThreadProcessId(hwnd)
        fg_hwnd = self._gui.GetForegroundWindow()
        fg_tid, _ = self._proc.GetWindowThreadProcessId(fg_hwnd) if fg_hwnd else (0, 0)
        our_tid = self._kernel32.GetCurrentThreadId()

        if self._user32.IsIconic(hwnd):
            self._user32.ShowWindow(hwnd, self._con.SW_RESTORE)

        attached_fg = bool(fg_tid and fg_tid != our_tid and self._user32.AttachThreadInput(our_tid, fg_tid, True))
        attached_tg = bool(
            target_tid and target_tid != our_tid and target_tid != fg_tid
            and self._user32.AttachThreadInput(our_tid, target_tid, True)
        )

        ok = bool(self._user32.SetForegroundWindow(hwnd))
        self._user32.BringWindowToTop(hwnd)

        if attached_tg:
            self._user32.AttachThreadInput(our_tid, target_tid, False)
        if attached_fg:
            self._user32.AttachThreadInput(our_tid, fg_tid, False)

        log.info("activate_hwnd hwnd=%s ok=%s target_tid=%s fg_tid=%s", hwnd, ok, target_tid, fg_tid)
        return ok

    def _activate_pid(self, pid: int) -> bool:
        target_hwnd = 0

        def _cb(hwnd: int, _lparam: int) -> bool:
            nonlocal target_hwnd
            if not self._user32.IsWindowVisible(hwnd):
                return True
            _tid, hpid = self._proc.GetWindowThreadProcessId(hwnd)
            if hpid == pid and not self._gui.GetWindow(hwnd, 4):  # GW_OWNER == 4
                target_hwnd = hwnd
                return False
            return True

        self._gui.EnumWindows(_cb, 0)
        if not target_hwnd:
            log.warning("activate_pid: no visible top-level window for pid %s", pid)
            return False
        return self._activate_hwnd(target_hwnd)

    # ---- input simulation ----------------------------------------------------------
    def simulate_paste(self) -> None:
        # Use pynput to send Ctrl+V — works regardless of keyboard layout.
        from pynput.keyboard import Controller, Key
        kbd = Controller()
        kbd.press(Key.ctrl)
        time.sleep(0.01)
        kbd.press("v")
        time.sleep(0.02)
        kbd.release("v")
        kbd.release(Key.ctrl)

    # ---- permissions ---------------------------------------------------------------
    def check_accessibility(self) -> bool:
        return True  # Windows has no accessibility-permission gate
