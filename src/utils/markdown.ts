// utils/markdown.ts — marked + DOMPurify 安全渲染。
import { marked } from "marked";
import DOMPurify from "dompurify";

marked.setOptions({
  gfm: true,
  breaks: true,
});

export function renderMarkdown(src: string): string {
  const raw = marked.parse(src, { async: false }) as string;
  return DOMPurify.sanitize(raw, {
    USE_PROFILES: { html: true },
    ALLOWED_ATTR: [
      "href", "title", "target", "rel",
      "class", "id",
      "src", "alt", "width", "height",
    ],
  });
}
