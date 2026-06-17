// types/sync.ts — 与 Rust commands/auth.rs::AuthStatus、sync::SyncStatus 对齐。
export interface AuthStatus {
  logged_in: boolean;
  email: string | null;
}

export type SyncState = "idle" | "syncing" | "error" | "offline" | "logged_out";

export interface SyncStatus {
  state: SyncState;
  logged_in: boolean;
  enabled: boolean;
  email: string | null;
  last_sync_at: number | null;
  pending: number;
  message: string | null;
}
