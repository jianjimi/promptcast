// api — 前端唯一调 Tauri 的入口。Vue 组件不要直接 import { invoke }。
import { invoke } from "@tauri-apps/api/core";

export const ping = () => invoke<string>("ping");

export const logDir = () => invoke<string>("log_dir");

// 后端 DbState 是否已就绪（避免与 setup() 的 manage 抢跑导致 "state not managed for db"）。
export const backendReady = () => invoke<boolean>("backend_ready");

// 启动门闸：轮询 backend_ready 直到就绪再加载数据。各窗口 mounted 第一步调用。
// 用 memoize 的 Promise，同一窗口内只真正轮询一次；backend_ready 不依赖 DbState，永不报错。
let _readyPromise: Promise<void> | null = null;
export function ensureBackendReady(): Promise<void> {
  if (!_readyPromise) {
    _readyPromise = (async () => {
      for (let i = 0; i < 300; i++) {
        try {
          if (await backendReady()) return;
        } catch {
          // IPC 偶发未就绪 —— 忽略后重试
        }
        await new Promise((r) => setTimeout(r, 30));
      }
      // 兜底：~9s 还没就绪也放行，让后续加载自己暴露真实错误而非永远卡住。
      console.warn("ensureBackendReady: backend 未在 ~9s 内就绪，继续加载（可能随后报错）");
    })();
  }
  return _readyPromise;
}

export * from "./prompts";
export * from "./folders";
export * from "./tags";
export * from "./settings";
export * from "./sites";
export * from "./inject";
export * from "./window";
export * from "./data";
export * from "./clipboard";
export * from "./auth";
export * from "./sync";
