// api/data.ts — JSON 导入/导出。
import { invoke } from "@tauri-apps/api/core";

export const dataExportJson = () => invoke<string>("data_export_json");

// 导出到指定文件路径（路径由前端 dialog.save 选取）。
export const dataExportToFile = (path: string) =>
  invoke<void>("data_export_to_file", { path });

export const dataImportJson = (json: string, mode: "merge" | "replace") =>
  invoke<{ inserted: number; updated: number }>("data_import_json", {
    args: { json, mode },
  });
