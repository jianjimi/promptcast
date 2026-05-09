// stores/tags.ts
import { defineStore } from "pinia";
import type { Tag } from "../types/tag";
import { tagsList, tagsCreate, tagsRename, tagsDelete } from "../api/tags";

interface State {
  list: Tag[];
  loaded: boolean;
}

export const useTagsStore = defineStore("tags", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      this.list = await tagsList();
      this.loaded = true;
    },
    async create(name: string, color: string | null = null): Promise<Tag> {
      const t = await tagsCreate(name, color);
      await this.loadAll();
      return t;
    },
    async rename(id: number, name: string): Promise<void> {
      await tagsRename(id, name);
      await this.loadAll();
    },
    async remove(id: number): Promise<void> {
      await tagsDelete(id);
      await this.loadAll();
    },
  },
});
