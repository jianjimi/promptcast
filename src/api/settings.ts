// api/settings.ts
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types/settings";

export const settingsGetAll = () => invoke<Settings>("settings_get_all");

export const settingsSet = <K extends keyof Settings>(
  key: K,
  value: Settings[K],
) => invoke<void>("settings_set", { key, value });
