// utils/logger.ts — JS 侧日志，转发到 Rust 端落盘到同一个 app.log。
//
// 即使 IPC 失败也至少 console 看得到（dev 时打开 Web Inspector 即可）。
import { invoke } from "@tauri-apps/api/core";

type Level = "trace" | "debug" | "info" | "warn" | "error";

let SOURCE = "frontend";

export function configureLogger(source: string): void {
  SOURCE = source;
}

async function send(level: Level, message: string, data?: unknown): Promise<void> {
  try {
    await invoke("log_record", {
      entry: {
        level,
        source: SOURCE,
        message,
        data: data === undefined ? null : (data as object | null),
      },
    });
  } catch (e) {
    // IPC 失败也不能吞 — console 至少看得见
    console.warn("log_record IPC failed:", e, "msg=", message);
  }
}

function fmt(level: Level, m: string, d?: unknown) {
  const tag = `[${SOURCE}/${level}]`;
  if (d !== undefined) {
    if (level === "error" || level === "warn") console.warn(tag, m, d);
    else console.log(tag, m, d);
  } else {
    if (level === "error" || level === "warn") console.warn(tag, m);
    else console.log(tag, m);
  }
}

export const log = {
  trace: (m: string, d?: unknown) => { fmt("trace", m, d); void send("trace", m, d); },
  debug: (m: string, d?: unknown) => { fmt("debug", m, d); void send("debug", m, d); },
  info: (m: string, d?: unknown) => { fmt("info", m, d); void send("info", m, d); },
  warn: (m: string, d?: unknown) => { fmt("warn", m, d); void send("warn", m, d); },
  error: (m: string, d?: unknown) => { fmt("error", m, d); void send("error", m, d); },
};

export function installGlobalErrorHandlers(): void {
  window.addEventListener("error", (ev) => {
    log.error(`window.onerror: ${ev.message}`, {
      filename: ev.filename, lineno: ev.lineno, colno: ev.colno,
      stack: ev.error?.stack,
    });
  });
  window.addEventListener("unhandledrejection", (ev) => {
    const r: any = ev.reason;
    log.error(`unhandledrejection: ${r?.message ?? String(r)}`, {
      stack: r?.stack,
    });
  });
  // 全局点击日志 — 帮助排查"按钮没反应"
  window.addEventListener("click", (e) => {
    const t = e.target as HTMLElement | null;
    if (!t) return;
    const tag = t.tagName;
    const cls = t.className && typeof t.className === "string" ? t.className.slice(0, 80) : "";
    log.debug(`click ${tag} class="${cls}"`);
  }, true);
  // 键盘事件日志（仅记 keydown 的 key 名，调试焦点用）
  window.addEventListener("keydown", (e) => {
    log.debug(`keydown key=${e.key} cmd=${e.metaKey} ctrl=${e.ctrlKey} shift=${e.shiftKey}`);
  }, true);
}
