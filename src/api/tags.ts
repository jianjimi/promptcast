// api/tags.ts
import { invoke } from "@tauri-apps/api/core";
import type { Tag } from "../types/tag";

export const tagsList = () => invoke<Tag[]>("tags_list");
export const tagsCreate = (name: string, color: string | null = null) =>
  invoke<Tag>("tags_create", { name, color });
export const tagsRename = (id: number, name: string) =>
  invoke<void>("tags_rename", { id, name });
export const tagsDelete = (id: number) =>
  invoke<void>("tags_delete", { id });
