// stores/sites.ts
import { defineStore } from "pinia";
import type { Site } from "../types/site";
import {
  sitesList,
  sitesCreate,
  sitesUpdate,
  sitesDelete,
  sitesReorder,
  sitesRefreshFavicon,
  sitesOpen,
} from "../api/sites";

interface State {
  list: Site[];
  loaded: boolean;
}

export const useSitesStore = defineStore("sites", {
  state: (): State => ({ list: [], loaded: false }),
  actions: {
    async loadAll(): Promise<void> {
      this.list = await sitesList();
      this.loaded = true;
    },
    async create(name: string, url: string): Promise<Site> {
      const s = await sitesCreate(name, url);
      await this.loadAll();
      return s;
    },
    async update(id: number, name: string, url: string): Promise<Site> {
      const s = await sitesUpdate(id, name, url);
      await this.loadAll();
      return s;
    },
    async remove(id: number): Promise<void> {
      await sitesDelete(id);
      await this.loadAll();
    },
    async reorder(orderedIds: number[]): Promise<void> {
      await sitesReorder(orderedIds);
      await this.loadAll();
    },
    async refreshFavicon(id: number): Promise<void> {
      await sitesRefreshFavicon(id);
      await this.loadAll();
    },
    async open(id: number): Promise<void> {
      await sitesOpen(id);
    },
  },
});
