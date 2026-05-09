// stores/ui.ts — UI 状态：抽屉显隐、当前 chip 选中、toast 队列。
import { defineStore } from "pinia";

export interface Toast {
  id: number;
  text: string;
  kind: "info" | "success" | "warning" | "danger";
  ttlMs: number;
}

interface State {
  drawerOpen: boolean;
  drawerPinned: boolean;
  activeChipKey: string; // "all" | "favorites" | `folder:${id}` | `tag:${id}`
  toasts: Toast[];
}

let toastId = 0;

export const useUIStore = defineStore("ui", {
  state: (): State => ({
    drawerOpen: true,
    drawerPinned: false,
    activeChipKey: "all",
    toasts: [],
  }),
  actions: {
    pushToast(
      text: string,
      kind: Toast["kind"] = "info",
      ttlMs = 2400,
    ): void {
      const id = ++toastId;
      this.toasts.push({ id, text, kind, ttlMs });
      window.setTimeout(() => this.dismissToast(id), ttlMs);
    },
    dismissToast(id: number): void {
      this.toasts = this.toasts.filter((t) => t.id !== id);
    },
  },
});
