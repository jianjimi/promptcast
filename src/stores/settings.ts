// stores/settings.ts
import { defineStore } from "pinia";
import { DEFAULT_SETTINGS, type Settings } from "../types/settings";
import { settingsGetAll, settingsSet } from "../api/settings";

interface State {
  data: Settings;
  loaded: boolean;
}

export const useSettingsStore = defineStore("settings", {
  state: (): State => ({
    data: { ...DEFAULT_SETTINGS },
    loaded: false,
  }),
  actions: {
    async loadAll(): Promise<void> {
      this.data = await settingsGetAll();
      this.loaded = true;
    },
    async set<K extends keyof Settings>(key: K, value: Settings[K]): Promise<void> {
      await settingsSet(key, value);
      this.data[key] = value;
    },
  },
});
