// api/auth.ts
import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "../types/sync";

export const authRegister = (email: string, password: string) =>
  invoke<AuthStatus>("auth_register", { email, password });
export const authLogin = (email: string, password: string) =>
  invoke<AuthStatus>("auth_login", { email, password });
export const authLogout = () => invoke<void>("auth_logout");
export const authStatus = () => invoke<AuthStatus>("auth_status");

export const authChangePassword = (oldPassword: string, newPassword: string) =>
  invoke<void>("auth_change_password", { oldPassword, newPassword });
export const authDeleteAccount = (password: string) =>
  invoke<void>("auth_delete_account", { password });
// 找回密码：本地开发返回 devToken（便于测试）；生产为 null（应走邮件）。
export const authForgotPassword = (email: string) =>
  invoke<string | null>("auth_forgot_password", { email });
export const authResetPassword = (token: string, newPassword: string) =>
  invoke<void>("auth_reset_password", { token, newPassword });
