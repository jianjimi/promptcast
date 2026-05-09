// types/site.ts — 注意 favicon 在 Rust 端是 BLOB，IPC 返回 base64 dataURI。
export interface Site {
  id: number;
  name: string;
  url: string;
  favicon_data_uri: string | null;
  favicon_fetched_at: number | null;
  sort_order: number;
  created_at: number;
}
