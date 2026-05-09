// api/ — 前端唯一调用 Tauri 的入口。Vue 组件不要直接 import { invoke }。
// 各领域子模块在 M1 起逐步填充。
import { invoke } from "@tauri-apps/api/core";

export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

// export * from "./prompts";   // M1
// export * from "./folders";   // M1
// export * from "./tags";      // M1
// export * from "./settings";  // M1
// export * from "./sites";     // M2.8
// export * from "./inject";    // M4
// export * from "./window";    // M4
