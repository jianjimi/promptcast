// stores/folders.ts — 文件夹（分类）管理（M1 + M3 拖拽排序）。
import { defineStore } from "pinia";
import type { Folder } from "../types/folder";

interface State {
  list: Folder[];
  loaded: boolean;
}

export const useFoldersStore = defineStore("folders", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      // TODO: M1
    },
  },
});
