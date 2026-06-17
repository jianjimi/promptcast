// stores/sync.ts — 同步状态（监听 sync-status-changed 实时更新）。
import { defineStore } from "pinia";
import {
  syncStatus,
  syncSetEnabled,
  syncNow,
  syncGetServerUrl,
  syncSetServerUrl,
} from "../api/sync";
import type { SyncStatus } from "../types/sync";

interface State {
  status: SyncStatus;
  serverUrl: string;
  loaded: boolean;
}

const EMPTY: SyncStatus = {
  state: "logged_out",
  logged_in: false,
  enabled: false,
  email: null,
  last_sync_at: null,
  pending: 0,
  message: null,
};

export const useSyncStore = defineStore("sync", {
  state: (): State => ({ status: { ...EMPTY }, serverUrl: "", loaded: false }),
  actions: {
    async load(): Promise<void> {
      this.status = await syncStatus();
      this.serverUrl = await syncGetServerUrl();
      this.loaded = true;
    },
    apply(s: SyncStatus): void {
      this.status = s;
    },
    async setEnabled(enabled: boolean): Promise<void> {
      await syncSetEnabled(enabled);
    },
    async now(): Promise<void> {
      await syncNow();
    },
    async setServerUrl(url: string): Promise<void> {
      await syncSetServerUrl(url);
      this.serverUrl = url;
    },
  },
});
