// stores/folders.ts
import { defineStore } from "pinia";
import type { Folder } from "../types/folder";
import {
  foldersList,
  foldersCreate,
  foldersRename,
  foldersDelete,
  foldersReorder,
} from "../api/folders";

interface State {
  list: Folder[];
  loaded: boolean;
}

export const useFoldersStore = defineStore("folders", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      this.list = await foldersList();
      this.loaded = true;
    },
    async create(name: string): Promise<Folder> {
      const f = await foldersCreate(name);
      await this.loadAll();
      return f;
    },
    async rename(id: number, name: string): Promise<void> {
      await foldersRename(id, name);
      await this.loadAll();
    },
    async remove(id: number): Promise<void> {
      await foldersDelete(id);
      await this.loadAll();
    },
    async reorder(orderedIds: number[]): Promise<void> {
      await foldersReorder(orderedIds);
      await this.loadAll();
    },
  },
});
