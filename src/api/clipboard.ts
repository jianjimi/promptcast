// api/clipboard.ts — 剪贴板历史 IPC。写入由后端监听线程负责，前端只读和删。
import { invoke } from "@tauri-apps/api/core";
import type { ClipEntry } from "../types/clipboard";

export const clipboardList = (limit = 500) =>
  invoke<ClipEntry[]>("clipboard_list", { limit });

export const clipboardDelete = (id: number) =>
  invoke<void>("clipboard_delete", { id });

export const clipboardClear = () => invoke<void>("clipboard_clear");
