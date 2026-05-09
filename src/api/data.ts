// api/data.ts — JSON 导入/导出。
import { invoke } from "@tauri-apps/api/core";

export const dataExportJson = () => invoke<string>("data_export_json");

export const dataImportJson = (json: string, mode: "merge" | "replace") =>
  invoke<{ inserted: number; updated: number }>("data_import_json", {
    args: { json, mode },
  });
