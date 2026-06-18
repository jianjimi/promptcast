// stores/update.ts — 应用更新状态机。
// 启动静默查（drawer）/ 手动查（设置）→ 有新版弹 UpdateModal → 下载（监听 update-progress）
// → 后端拉起安装器。下载是阻塞后端命令，进度靠事件推。
import { defineStore } from "pinia";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  updateCheck,
  updateDownloadInstall,
  updateGetManifestUrl,
  updateSetManifestUrl,
} from "../api/update";
import { EVT_UPDATE_PROGRESS } from "../composables/useAppEvents";
import type { UpdateInfo, UpdateProgress } from "../types/update";

interface State {
  info: UpdateInfo | null;
  visible: boolean;
  checking: boolean;
  downloading: boolean;
  launched: boolean;
  progress: UpdateProgress;
  error: string | null;
  manifestUrl: string;
  unlisten: UnlistenFn | null;
}

export const useUpdateStore = defineStore("update", {
  state: (): State => ({
    info: null,
    visible: false,
    checking: false,
    downloading: false,
    launched: false,
    progress: { downloaded: 0, total: null },
    error: null,
    manifestUrl: "",
    unlisten: null,
  }),
  getters: {
    percent(s): number {
      if (!s.progress.total || s.progress.total <= 0) return 0;
      return Math.min(
        100,
        Math.round((s.progress.downloaded / s.progress.total) * 100),
      );
    },
  },
  actions: {
    async loadManifestUrl(): Promise<void> {
      try {
        this.manifestUrl = await updateGetManifestUrl();
      } catch {
        /* ignore */
      }
    },
    async setManifestUrl(url: string): Promise<void> {
      await updateSetManifestUrl(url);
      this.manifestUrl = url.trim();
    },
    /**
     * 查更新。manual=false 为启动静默查（找不到/无新版都不打扰）；
     * manual=true 返回布尔让调用方决定提示（“已是最新” / “未配置”）。
     * 返回 true = 有新版且已弹窗。
     */
    async check(manual = false): Promise<boolean> {
      if (this.checking) return false;
      this.checking = true;
      this.error = null;
      try {
        const info = await updateCheck();
        if (info) {
          this.info = info;
          // 复位上一轮下载残留，否则二次弹窗会带着 launched=true / 旧进度条出现。
          this.launched = false;
          this.progress = { downloaded: 0, total: null };
          this.visible = true;
          return true;
        }
        return false;
      } catch (e) {
        // 静默查不打扰用户，只有手动查才暴露错误。
        if (manual) this.error = String(e);
        return false;
      } finally {
        this.checking = false;
      }
    },
    async startInstall(): Promise<void> {
      if (this.downloading) return;
      this.downloading = true;
      this.launched = false;
      this.error = null;
      this.progress = { downloaded: 0, total: null };
      // 监听进度（一次性绑定，下载结束/出错后解绑）。
      if (!this.unlisten) {
        this.unlisten = await listen<UpdateProgress>(
          EVT_UPDATE_PROGRESS,
          (e) => {
            this.progress = e.payload;
          },
        );
      }
      try {
        await updateDownloadInstall();
        // 命令返回 = 安装器已拉起（Windows setup/msi、macOS dmg 已打开）。
        this.launched = true;
      } catch (e) {
        this.error = String(e);
      } finally {
        this.downloading = false;
        if (this.unlisten) {
          this.unlisten();
          this.unlisten = null;
        }
      }
    },
    dismiss(): void {
      this.visible = false;
      this.error = null;
      this.launched = false;
      this.progress = { downloaded: 0, total: null };
      if (this.unlisten) {
        this.unlisten();
        this.unlisten = null;
      }
    },
  },
});
