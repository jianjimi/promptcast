// api/prompts.ts
import { invoke } from "@tauri-apps/api/core";
import type { Prompt, PromptDraft, SortMode } from "../types/prompt";

export const promptsList = (sort: SortMode) =>
  invoke<Prompt[]>("prompts_list", { sort });

export const promptsGet = (id: number) =>
  invoke<Prompt>("prompts_get", { id });

export const promptsCreate = (draft: PromptDraft) =>
  invoke<Prompt>("prompts_create", { draft });

export const promptsUpdate = (id: number, draft: PromptDraft) =>
  invoke<Prompt>("prompts_update", { id, draft });

export const promptsDelete = (id: number) =>
  invoke<void>("prompts_delete", { id });

export const promptsToggleFavorite = (id: number) =>
  invoke<Prompt>("prompts_toggle_favorite", { id });

export const promptsTogglePin = (id: number) =>
  invoke<Prompt>("prompts_toggle_pin", { id });

export const promptsRecordUse = (id: number) =>
  invoke<void>("prompts_record_use", { id });
