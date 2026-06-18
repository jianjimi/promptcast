// utils/dialog.ts — 统一的原生确认对话框封装。
// 为什么不用 window.confirm：Tauri webview 里 window.confirm 不可靠（部分窗口直接返回 false，
// 导致「删除没反应」）。这里走 @tauri-apps/plugin-dialog 的原生对话框，行为一致可靠。
import { confirm as tauriConfirm } from "@tauri-apps/plugin-dialog";

/** 危险操作确认。返回用户是否点了「确定」。出错（如插件不可用）按取消处理，绝不误删。 */
export async function confirmDanger(message: string, title = "请确认"): Promise<boolean> {
  try {
    return await tauriConfirm(message, { title, kind: "warning" });
  } catch {
    return false;
  }
}
