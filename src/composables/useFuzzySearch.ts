// composables/useFuzzySearch.ts — Fuse.js 包装。
import Fuse, { type IFuseOptions } from "fuse.js";
import type { Prompt } from "../types/prompt";
import type { Tag } from "../types/tag";

export interface SearchableContext {
  tagsById: Map<number, string>;
}

export function buildSearchable(
  prompts: Prompt[],
  tags: Tag[],
): { items: PromptSearchItem[]; ctx: SearchableContext } {
  const tagsById = new Map(tags.map((t) => [t.id, t.name]));
  const items = prompts.map((p) => ({
    id: p.id,
    title: p.title,
    content: p.content,
    tags: p.tag_ids.map((id) => tagsById.get(id) ?? ""),
    raw: p,
  }));
  return { items, ctx: { tagsById } };
}

export interface PromptSearchItem {
  id: number;
  title: string;
  content: string;
  tags: string[];
  raw: Prompt;
}

const FUSE_OPTS: IFuseOptions<PromptSearchItem> = {
  includeScore: true,
  threshold: 0.4,
  ignoreLocation: true,
  keys: [
    { name: "title", weight: 0.6 },
    { name: "tags", weight: 0.25 },
    { name: "content", weight: 0.15 },
  ],
};

export function searchPrompts(
  items: PromptSearchItem[],
  query: string,
): PromptSearchItem[] {
  const q = query.trim();
  if (!q) return items;
  const fuse = new Fuse(items, FUSE_OPTS);
  return fuse.search(q).map((r) => r.item);
}
