"""Abstract platform façade — captures previous foreground app and re-activates it."""
from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass(slots=True)
class ForegroundRef:
    pid: int | None = None
    hwnd: int | None = None  # platform-specific opaque handle (Windows only)


class Platform(ABC):
    @abstractmethod
    def capture_foreground(self) -> ForegroundRef: ...

    @abstractmethod
    def activate(self, ref: ForegroundRef) -> bool: ...

    @abstractmethod
    def simulate_paste(self) -> None: ...

    @abstractmethod
    def check_accessibility(self) -> bool: ...

    def request_accessibility(self) -> None:  # default no-op
        return None
