// stores/prompts.ts
import { defineStore } from "pinia";
import type { Prompt, PromptDraft, SortMode } from "../types/prompt";
import {
  promptsList,
  promptsCreate,
  promptsUpdate,
  promptsDelete,
  promptsToggleFavorite,
  promptsTogglePin,
  promptsRecordUse,
} from "../api/prompts";

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
      return state.list.find((p) => p.id === state.selectedId) ?? null;
    },
  },
  actions: {
    async loadAll(): Promise<void> {
      this.list = await promptsList(this.sortMode);
      this.loaded = true;
      if (
        this.selectedId !== null &&
        !this.list.find((p) => p.id === this.selectedId)
      ) {
        this.selectedId = this.list[0]?.id ?? null;
      } else if (this.selectedId === null) {
        this.selectedId = this.list[0]?.id ?? null;
      }
    },
    async setSort(mode: SortMode): Promise<void> {
      this.sortMode = mode;
      await this.loadAll();
    },
    async create(draft: PromptDraft): Promise<Prompt> {
      const p = await promptsCreate(draft);
      await this.loadAll();
      this.selectedId = p.id;
      return p;
    },
    async update(id: number, draft: PromptDraft): Promise<Prompt> {
      const p = await promptsUpdate(id, draft);
      await this.loadAll();
      return p;
    },
    async remove(id: number): Promise<void> {
      await promptsDelete(id);
      await this.loadAll();
    },
    async toggleFavorite(id: number): Promise<void> {
      await promptsToggleFavorite(id);
      await this.loadAll();
    },
    async togglePin(id: number): Promise<void> {
      await promptsTogglePin(id);
      await this.loadAll();
    },
    async recordUse(id: number): Promise<void> {
      await promptsRecordUse(id);
    },
    select(id: number | null): void {
      this.selectedId = id;
    },
  },
});
