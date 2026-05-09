// utils/debounce.ts — 经典 debounce；搜索框节流用。
export function debounce<T extends (...args: any[]) => void>(
  fn: T,
  waitMs: number,
): (...args: Parameters<T>) => void {
  let timer: number | null = null;
  return (...args: Parameters<T>) => {
    if (timer !== null) window.clearTimeout(timer);
    timer = window.setTimeout(() => fn(...args), waitMs);
  };
}
