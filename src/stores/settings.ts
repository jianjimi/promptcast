// stores/settings.ts — 应用设置（持久化在 settings 表）。
import { defineStore } from "pinia";
import { DEFAULT_SETTINGS, type Settings } from "../types/settings";

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
      // TODO: M1（settings_get_all）
    },
    async set<K extends keyof Settings>(
      _key: K,
      _value: Settings[K],
    ): Promise<void> {
      // TODO: M1（settings_set）
    },
  },
});
