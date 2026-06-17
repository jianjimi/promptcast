// api/sync.ts
import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus } from "../types/sync";

export const syncStatus = () => invoke<SyncStatus>("sync_status");
export const syncSetEnabled = (enabled: boolean) =>
  invoke<void>("sync_set_enabled", { enabled });
export const syncNow = () => invoke<void>("sync_now");
export const syncGetServerUrl = () => invoke<string>("sync_get_server_url");
export const syncSetServerUrl = (url: string) =>
  invoke<void>("sync_set_server_url", { url });
