import { describe, it, expect } from "vitest";
import { snippet, relativeTime } from "./format";

describe("snippet", () => {
  it("折叠空白", () => {
    expect(snippet("hello   world", 80)).toBe("hello world");
  });
  it("超长截断加省略号", () => {
    expect(snippet("a".repeat(100), 10)).toBe("a".repeat(10) + "…");
  });
  it("不超长原样返回", () => {
    expect(snippet("short", 80)).toBe("short");
  });
});

describe("relativeTime", () => {
  it("null → 未使用", () => {
    expect(relativeTime(null)).toBe("未使用");
  });
  it("数秒内 → 刚刚", () => {
    expect(relativeTime(Date.now() - 2000)).toBe("刚刚");
  });
  it("几分钟前", () => {
    expect(relativeTime(Date.now() - 5 * 60_000)).toBe("5 分钟前");
  });
});
