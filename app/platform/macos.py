"""macOS platform layer using PyObjC."""
from __future__ import annotations

import logging
import time

from app.platform.base import ForegroundRef, Platform

log = logging.getLogger(__name__)


class MacOSPlatform(Platform):
    def __init__(self) -> None:
        from AppKit import NSApp, NSApplication, NSApplicationActivationPolicyAccessory  # noqa
        try:
            app = NSApplication.sharedApplication()
            app.setActivationPolicy_(NSApplicationActivationPolicyAccessory)
        except Exception as exc:
            log.warning("setActivationPolicy failed: %s", exc)

    def capture_foreground(self) -> ForegroundRef:
        from AppKit import NSWorkspace
        front = NSWorkspace.sharedWorkspace().frontmostApplication()
        if front is None:
            return ForegroundRef()
        pid = int(front.processIdentifier())
        log.info("capture foreground: pid=%s bundle=%s", pid, front.bundleIdentifier())
        return ForegroundRef(pid=pid)

    def activate(self, ref: ForegroundRef) -> bool:
        from AppKit import NSRunningApplication, NSApplicationActivateIgnoringOtherApps
        if not ref.pid:
            return False
        target = NSRunningApplication.runningApplicationWithProcessIdentifier_(ref.pid)
        if target is None:
            log.warning("activate: no NSRunningApplication for pid=%s", ref.pid)
            return False
        ok = bool(target.activateWithOptions_(NSApplicationActivateIgnoringOtherApps))
        log.info("activate pid=%s ok=%s", ref.pid, ok)
        return ok

    def simulate_paste(self) -> None:
        from pynput.keyboard import Controller, Key
        kbd = Controller()
        kbd.press(Key.cmd)
        time.sleep(0.01)
        kbd.press("v")
        time.sleep(0.02)
        kbd.release("v")
        kbd.release(Key.cmd)

    def check_accessibility(self) -> bool:
        try:
            from ApplicationServices import AXIsProcessTrusted
            return bool(AXIsProcessTrusted())
        except Exception as exc:
            log.warning("AXIsProcessTrusted failed: %s", exc)
            return False

    def request_accessibility(self) -> None:
        try:
            from ApplicationServices import AXIsProcessTrustedWithOptions
            from CoreFoundation import CFDictionaryCreate, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks
            key = "AXTrustedCheckOptionPrompt"
            options = CFDictionaryCreate(
                None, [key], [True], 1,
                kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
            )
            AXIsProcessTrustedWithOptions(options)
        except Exception as exc:
            log.warning("request accessibility failed: %s", exc)
