// useTheme.ts — 主题管理。
//
// 三种模式：system（默认）/ light / dark。
// system 模式下挂监听跟系统切换；light/dark 显式时不响应系统变化。
type Mode = "light" | "dark";
export type ThemeMode = "system" | Mode;

function apply(mode: Mode): void {
  document.documentElement.setAttribute("data-theme", mode);
}

function resolveSystem(): Mode {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
let currentMode: ThemeMode = "system";

function onSystemChange(e: MediaQueryListEvent) {
  if (currentMode !== "system") return;
  apply(e.matches ? "dark" : "light");
}

export function initTheme(): void {
  apply(resolveSystem());
  mediaQuery.addEventListener("change", onSystemChange);
}

export function setThemeMode(mode: ThemeMode): void {
  currentMode = mode;
  if (mode === "system") apply(resolveSystem());
  else apply(mode);
}

/** 启动后从 settings 同步一次。 */
export function applyPersistedTheme(mode: ThemeMode): void {
  setThemeMode(mode);
}
