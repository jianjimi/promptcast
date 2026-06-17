import { describe, it, expect } from "vitest";
import { buildSearchable, searchPrompts } from "./useFuzzySearch";
import type { Prompt } from "../types/prompt";

const P = (o: Partial<Prompt>): Prompt => ({
  id: 1, title: "", content: "", folder_id: null, tag_ids: [],
  is_favorite: false, is_pinned: false, use_count: 0, last_used_at: null,
  created_at: 0, updated_at: 0, ...o,
});

describe("searchPrompts", () => {
  it("空查询返回全部（兑现 M2-5 验收）", () => {
    const { items } = buildSearchable([P({ id: 1, title: "foo" }), P({ id: 2, title: "bar" })], []);
    expect(searchPrompts(items, "  ")).toHaveLength(2);
  });

  it("按标题匹配", () => {
    const { items } = buildSearchable(
      [P({ id: 1, title: "Daily Standup" }), P({ id: 2, title: "Code Review" })],
      [],
    );
    expect(searchPrompts(items, "standup")[0].raw.id).toBe(1);
  });

  it("按标签名匹配（搜索域含 tags）", () => {
    const { items } = buildSearchable(
      [P({ id: 1, title: "x", tag_ids: [10] })],
      [{ id: 10, name: "marketing", color: null }],
    );
    expect(searchPrompts(items, "marketing")).toHaveLength(1);
  });

  it("无匹配则过滤掉", () => {
    const { items } = buildSearchable([P({ id: 1, title: "hello" })], []);
    expect(searchPrompts(items, "zzzzzqqqq")).toHaveLength(0);
  });
});
