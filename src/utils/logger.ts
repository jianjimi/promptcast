// utils/logger.ts — 把 JS 日志转发到后端，落到同一个 app.log。
//
// 同时拦截全局 onerror / unhandledrejection，所有异常都进文件。
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
  } catch {
    // 防止日志失败导致循环
  }
}

export const log = {
  trace: (m: string, d?: unknown) => { console.debug(m, d ?? ""); void send("trace", m, d); },
  debug: (m: string, d?: unknown) => { console.debug(m, d ?? ""); void send("debug", m, d); },
  info: (m: string, d?: unknown) => { console.info(m, d ?? ""); void send("info", m, d); },
  warn: (m: string, d?: unknown) => { console.warn(m, d ?? ""); void send("warn", m, d); },
  error: (m: string, d?: unknown) => { console.error(m, d ?? ""); void send("error", m, d); },
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
}
