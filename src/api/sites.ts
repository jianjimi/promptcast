// api/sites.ts
import { invoke } from "@tauri-apps/api/core";
import type { Site } from "../types/site";

export const sitesList = () => invoke<Site[]>("sites_list");
export const sitesCreate = (name: string, url: string) =>
  invoke<Site>("sites_create", { name, url });
export const sitesUpdate = (id: number, name: string, url: string) =>
  invoke<Site>("sites_update", { id, name, url });
export const sitesDelete = (id: number) =>
  invoke<void>("sites_delete", { id });
export const sitesReorder = (orderedIds: number[]) =>
  invoke<void>("sites_reorder", { orderedIds });
export const sitesRefreshFavicon = (id: number) =>
  invoke<Site>("sites_refresh_favicon", { id });
export const sitesOpen = (id: number) =>
  invoke<void>("sites_open", { id });
