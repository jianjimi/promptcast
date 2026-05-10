"""Platform-layer factory."""
from __future__ import annotations

import sys

from app.platform.base import ForegroundRef, Platform


_instance: Platform | None = None


def get_platform() -> Platform:
    global _instance
    if _instance is None:
        if sys.platform == "win32":
            from app.platform.windows import WindowsPlatform
            _instance = WindowsPlatform()
        elif sys.platform == "darwin":
            from app.platform.macos import MacOSPlatform
            _instance = MacOSPlatform()
        else:
            raise NotImplementedError(f"unsupported platform: {sys.platform}")
    return _instance


__all__ = ["ForegroundRef", "Platform", "get_platform"]
