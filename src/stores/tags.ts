// stores/tags.ts — 标签管理（M1 实装）。
import { defineStore } from "pinia";
import type { Tag } from "../types/tag";

interface State {
  list: Tag[];
  loaded: boolean;
}

export const useTagsStore = defineStore("tags", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      // TODO: M1
    },
  },
});
