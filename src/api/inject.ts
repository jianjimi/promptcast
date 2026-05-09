// api/inject.ts
import { invoke } from "@tauri-apps/api/core";

export interface InjectResult {
  ok: boolean;
  fallback: "clipboard" | null;
  message: string | null;
}

export const injectPaste = (content: string) =>
  invoke<InjectResult>("inject_paste", { content });

export const injectCopyOnly = (content: string) =>
  invoke<void>("inject_copy_only", { content });

export const permissionsCheckAccessibility = () =>
  invoke<boolean>("permissions_check_accessibility");

export const permissionsRequestAccessibility = () =>
  invoke<boolean>("permissions_request_accessibility");
