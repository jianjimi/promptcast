// stores/clipboard.ts — 剪贴板历史列表。
import { defineStore } from "pinia";
import type { ClipEntry } from "../types/clipboard";
import { clipboardList, clipboardDelete, clipboardClear } from "../api/clipboard";

interface State {
  list: ClipEntry[];
  loaded: boolean;
}

export const useClipboardStore = defineStore("clipboard", {
  state: (): State => ({
    list: [],
    loaded: false,
  }),
  actions: {
    async loadAll(limit = 500): Promise<void> {
      this.list = await clipboardList(limit);
      this.loaded = true;
    },
    async remove(id: number): Promise<void> {
      await clipboardDelete(id);
      await this.loadAll();
    },
    async removeMany(ids: number[]): Promise<void> {
      for (const id of ids) await clipboardDelete(id);
      await this.loadAll();
    },
    async clear(): Promise<void> {
      await clipboardClear();
      this.list = [];
    },
  },
});
