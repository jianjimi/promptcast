// api/auth.ts
import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "../types/sync";

export const authRegister = (email: string, password: string) =>
  invoke<AuthStatus>("auth_register", { email, password });
export const authLogin = (email: string, password: string) =>
  invoke<AuthStatus>("auth_login", { email, password });
export const authLogout = () => invoke<void>("auth_logout");
export const authStatus = () => invoke<AuthStatus>("auth_status");
