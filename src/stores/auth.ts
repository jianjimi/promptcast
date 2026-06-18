// stores/auth.ts — 账户登录态。
import { defineStore } from "pinia";
import {
  authStatus, authLogin, authRegister, authLogout,
  authChangePassword, authDeleteAccount, authForgotPassword, authResetPassword,
} from "../api/auth";

interface State {
  loggedIn: boolean;
  email: string | null;
  loaded: boolean;
}

export const useAuthStore = defineStore("auth", {
  state: (): State => ({ loggedIn: false, email: null, loaded: false }),
  actions: {
    async load(): Promise<void> {
      const s = await authStatus();
      this.loggedIn = s.logged_in;
      this.email = s.email;
      this.loaded = true;
    },
    async login(email: string, password: string): Promise<void> {
      const s = await authLogin(email, password);
      this.loggedIn = s.logged_in;
      this.email = s.email;
    },
    async register(email: string, password: string): Promise<void> {
      const s = await authRegister(email, password);
      this.loggedIn = s.logged_in;
      this.email = s.email;
    },
    async logout(): Promise<void> {
      await authLogout();
      this.loggedIn = false;
      this.email = null;
    },
    async changePassword(oldPassword: string, newPassword: string): Promise<void> {
      await authChangePassword(oldPassword, newPassword);
    },
    async deleteAccount(password: string): Promise<void> {
      await authDeleteAccount(password);
      this.loggedIn = false;
      this.email = null;
    },
    forgotPassword(email: string): Promise<string | null> {
      return authForgotPassword(email);
    },
    resetPassword(token: string, newPassword: string): Promise<void> {
      return authResetPassword(token, newPassword);
    },
  },
});
