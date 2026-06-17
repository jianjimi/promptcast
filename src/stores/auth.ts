// stores/auth.ts — 账户登录态。
import { defineStore } from "pinia";
import { authStatus, authLogin, authRegister, authLogout } from "../api/auth";

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
  },
});
