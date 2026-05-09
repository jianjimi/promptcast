// stores/prompts.ts — 提示词列表与 CRUD（M1 实装）。
import { defineStore } from "pinia";
import type { Prompt, SortMode } from "../types/prompt";

interface State {
  list: Prompt[];
  loaded: boolean;
  selectedId: number | null;
  sortMode: SortMode;
}

export const usePromptsStore = defineStore("prompts", {
  state: (): State => ({
    list: [],
    loaded: false,
    selectedId: null,
    sortMode: "recent_used",
  }),
  getters: {
    pinned(state): Prompt[] {
      return state.list.filter((p) => p.is_pinned);
    },
    rest(state): Prompt[] {
      return state.list.filter((p) => !p.is_pinned);
    },
    selected(state): Prompt | null {
      return (
        state.list.find((p) => p.id === state.selectedId) ?? null
      );
    },
  },
  actions: {
    // M1 实装：调 api/prompts 拉取并填充 list
    async loadAll(): Promise<void> {
      // TODO: M1
    },
  },
});
