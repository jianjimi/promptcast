// utils/format.ts
export function relativeTime(ms: number | null | undefined): string {
  if (!ms) return "未使用";
  const diff = Date.now() - ms;
  if (diff < 60_000) return "刚刚";
  const min = Math.floor(diff / 60_000);
  if (min < 60) return `${min} 分钟前`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} 小时前`;
  const day = Math.floor(hr / 24);
  if (day < 30) return `${day} 天前`;
  const mo = Math.floor(day / 30);
  if (mo < 12) return `${mo} 个月前`;
  return `${Math.floor(mo / 12)} 年前`;
}

export function snippet(s: string, maxLen = 80): string {
  const oneLine = s.replace(/\s+/g, " ").trim();
  if (oneLine.length <= maxLen) return oneLine;
  return oneLine.slice(0, maxLen) + "…";
}

export function isMac(): boolean {
  return navigator.platform.toLowerCase().includes("mac");
}

export function modKey(): string {
  return isMac() ? "⌘" : "Ctrl";
}
