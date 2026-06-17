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
  // 调试用的全局点击/键盘日志仅在开发构建启用：
  // 生产里它会把每次按键（含可打印字符）经 IPC 明文写进日志文件，属隐私泄露。
  if (import.meta.env.DEV) {
    window.addEventListener("click", (e) => {
      const t = e.target as HTMLElement | null;
      if (!t) return;
      const tag = t.tagName;
      const cls = t.className && typeof t.className === "string" ? t.className.slice(0, 80) : "";
      log.debug(`click ${tag} class="${cls}"`);
    }, true);
    window.addEventListener("keydown", (e) => {
      // 只记修饰键状态，不记可打印字符本身。
      log.debug(`keydown key=${e.key.length === 1 ? "·" : e.key} cmd=${e.metaKey} ctrl=${e.ctrlKey} shift=${e.shiftKey}`);
    }, true);
  }
}
