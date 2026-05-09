// api/folders.ts
import { invoke } from "@tauri-apps/api/core";
import type { Folder } from "../types/folder";

export const foldersList = () => invoke<Folder[]>("folders_list");
export const foldersCreate = (name: string) =>
  invoke<Folder>("folders_create", { name });
export const foldersRename = (id: number, name: string) =>
  invoke<void>("folders_rename", { id, name });
export const foldersDelete = (id: number) =>
  invoke<void>("folders_delete", { id });
export const foldersReorder = (orderedIds: number[]) =>
  invoke<void>("folders_reorder", { orderedIds });
