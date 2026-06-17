// useAppEvents.ts — 跨窗口事件常量 + listen 包装。
//
// 用法：在窗口 mounted 里调用 listenAppEvent('prompts-changed', () => store.loadAll())
// 返回的 unlisten 在 beforeUnmount 调用。
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const EVT_PROMPTS_CHANGED = "prompts-changed";
export const EVT_FOLDERS_CHANGED = "folders-changed";
export const EVT_TAGS_CHANGED = "tags-changed";
export const EVT_SITES_CHANGED = "sites-changed";
export const EVT_SETTINGS_CHANGED = "settings-changed";
export const EVT_THEME_CHANGED = "theme-changed";
export const EVT_CLIPBOARD_CHANGED = "clipboard-changed";

export async function listenAppEvent<T = unknown>(
  name: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return await listen<T>(name, (e) => handler(e.payload));
}
