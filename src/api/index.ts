// api — 前端唯一调 Tauri 的入口。Vue 组件不要直接 import { invoke }。
import { invoke } from "@tauri-apps/api/core";

export const ping = () => invoke<string>("ping");

export * from "./prompts";
export * from "./folders";
export * from "./tags";
export * from "./settings";
export * from "./sites";
export * from "./inject";
export * from "./window";
export * from "./data";
