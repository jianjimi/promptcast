"""Design tokens — shared spacing/radius/font and per-theme color palette."""
from __future__ import annotations

from types import MappingProxyType

# Geometry & typography (theme-independent).
BASE = MappingProxyType({
    "space_1": 2, "space_2": 4, "space_3": 6, "space_4": 8,
    "space_5": 10, "space_6": 12, "space_8": 16, "space_10": 20,
    "space_12": 24, "space_16": 32,

    "radius_sm": 4, "radius_md": 6, "radius_lg": 8, "radius_xl": 12,
    "radius_pill": 999,

    "fs_10": 10, "fs_11": 11, "fs_12": 12, "fs_13": 13,
    "fs_14": 14, "fs_16": 16, "fs_18": 18,

    "fw_regular": 400, "fw_medium": 500, "fw_semibold": 600, "fw_bold": 700,

    "font_mono": '"JetBrains Mono", "SF Mono", Menlo, Consolas, "Courier New", monospace',
    "font_sans": '-apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif',
})

LIGHT = MappingProxyType({
    "bg_base": "#fafafb",
    "bg_surface": "#ffffff",
    "bg_titlebar": "rgba(255, 255, 255, 0.9)",
    "bg_hover": "#f4f4f5",
    "bg_selected": "#f4f4f5",
    "bg_input": "#ffffff",
    "bg_input_disabled": "#fafafa",
    "bg_glass": "rgba(255, 255, 255, 0.95)",

    "border": "#ececee",
    "border_strong": "#d4d4d8",

    "text_primary": "#18181b",
    "text_secondary": "#52525b",
    "text_tertiary": "#a1a1aa",

    "accent": "#18181b",
    "accent_fg": "#ffffff",
    "accent_soft": "#f4f4f5",

    "star": "#f59e0b",
    "danger": "#dc2626",
    "success": "#16a34a",
    "warning": "#d97706",

    "shadow_alpha": "rgba(0, 0, 0, 0.10)",
    "scrollbar_thumb": "#d4d4d8",
    "scrollbar_thumb_hover": "#a1a1aa",
})

DARK = MappingProxyType({
    "bg_base": "#0e0e10",
    "bg_surface": "#17171a",
    "bg_titlebar": "rgba(23, 23, 26, 0.9)",
    "bg_hover": "#1f1f23",
    "bg_selected": "#232328",
    "bg_input": "#1c1c20",
    "bg_input_disabled": "#161618",
    "bg_glass": "rgba(23, 23, 26, 0.92)",

    "border": "#2a2a2f",
    "border_strong": "#3a3a40",

    "text_primary": "#fafafa",
    "text_secondary": "#a1a1aa",
    "text_tertiary": "#71717a",

    "accent": "#fafafa",
    "accent_fg": "#0e0e10",
    "accent_soft": "#232328",

    "star": "#fbbf24",
    "danger": "#f87171",
    "success": "#4ade80",
    "warning": "#fbbf24",

    "shadow_alpha": "rgba(0, 0, 0, 0.55)",
    "scrollbar_thumb": "#3a3a40",
    "scrollbar_thumb_hover": "#52525b",
})


def palette(theme: str) -> dict[str, object]:
    """Return a flat dict combining BASE + theme-specific colors."""
    colors = DARK if theme == "dark" else LIGHT
    return {**BASE, **colors}
