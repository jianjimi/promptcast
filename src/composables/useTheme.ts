// useTheme.ts — 监听系统外观，给 <html> 设 data-theme。
// M3 之后会改成读取 settings.theme（system/light/dark），现在先 system。
type Mode = "light" | "dark";

function apply(mode: Mode): void {
  document.documentElement.setAttribute("data-theme", mode);
}

function resolveSystem(): Mode {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

let mediaQuery: MediaQueryList | null = null;

export function initTheme(): void {
  apply(resolveSystem());
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", (e) => {
    apply(e.matches ? "dark" : "light");
  });
}

// 后续 settings 模式切换调用：
export function setThemeMode(mode: Mode | "system"): void {
  if (mode === "system") apply(resolveSystem());
  else apply(mode);
}
