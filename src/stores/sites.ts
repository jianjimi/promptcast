// stores/sites.ts — 底部网址快捷管理（M2.8 实装 + M3 设置页）。
import { defineStore } from "pinia";
import type { Site } from "../types/site";

interface State {
  list: Site[];
  loaded: boolean;
}

export const useSitesStore = defineStore("sites", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      // TODO: M2.8
    },
  },
});
