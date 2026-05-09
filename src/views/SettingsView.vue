<!--
  SettingsView.vue — 设置窗口（独立）。
-->
<script lang="ts">
import { defineComponent } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  Sliders, Keyboard, Palette, FolderTree, Globe, Database,
  ShieldCheck, Info, X, Settings as SettingsIcon, FileText,
} from "lucide-vue-next";

import HotkeyRecorder from "../components/settings/HotkeyRecorder.vue";
import FoldersPanel from "../components/settings/FoldersPanel.vue";
import SitesPanel from "../components/settings/SitesPanel.vue";
import DataPanel from "../components/settings/DataPanel.vue";
import BaseToast from "../components/ui/BaseToast.vue";

import { useSettingsStore } from "../stores/settings";
import {
  registerHotkey,
  unregisterHotkey,
} from "../api/window";
import {
  permissionsCheckAccessibility,
  permissionsRequestAccessibility,
} from "../api/inject";
import { logDir } from "../api";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { setThemeMode, applyPersistedTheme } from "../composables/useTheme";
import {
  listenAppEvent,
  EVT_THEME_CHANGED, EVT_SETTINGS_CHANGED,
  EVT_FOLDERS_CHANGED, EVT_TAGS_CHANGED, EVT_SITES_CHANGED,
} from "../composables/useAppEvents";
import { useFoldersStore } from "../stores/folders";
import { useTagsStore } from "../stores/tags";
import { useSitesStore } from "../stores/sites";
import { isMac } from "../utils/format";
import { log } from "../utils/logger";
import type { ThemeMode } from "../types/settings";

type TabKey =
  | "general" | "hotkey" | "theme" | "folders"
  | "sites" | "data" | "permissions" | "about";

interface NavItem {
  key: TabKey;
  label: string;
  icon: any;
}

export default defineComponent({
  name: "SettingsView",
  components: {
    HotkeyRecorder, FoldersPanel, SitesPanel, DataPanel, BaseToast,
    Sliders, Keyboard, Palette, FolderTree, Globe, Database,
    ShieldCheck, Info, X, SettingsIcon, FileText,
  },
  data() {
    return {
      tab: "general" as TabKey,
      hotkeyDraft: "",
      accessibilityOk: true,
      isMacOS: isMac(),
      logsPath: "",
      unlisteners: [] as UnlistenFn[],
    };
  },
  computed: {
    settings() { return useSettingsStore(); },
    appVersion(): string { return "0.1.0"; },
    navItems(): NavItem[] {
      return [
        { key: "general", label: "常规", icon: Sliders },
        { key: "hotkey", label: "快捷键", icon: Keyboard },
        { key: "theme", label: "主题", icon: Palette },
        { key: "folders", label: "分类管理", icon: FolderTree },
        { key: "sites", label: "网址快捷", icon: Globe },
        { key: "data", label: "数据", icon: Database },
        { key: "permissions", label: "权限诊断", icon: ShieldCheck },
        { key: "about", label: "关于", icon: Info },
      ];
    },
  },
  async mounted() {
    log.info("SettingsView mounted");
    if (!this.settings.loaded) await this.settings.loadAll();
    applyPersistedTheme(this.settings.data.theme);
    this.hotkeyDraft = this.settings.data.hotkey ?? "";
    this.accessibilityOk = await permissionsCheckAccessibility();
    try { this.logsPath = await logDir(); } catch { /* */ }

    this.unlisteners.push(
      await listenAppEvent<ThemeMode>(EVT_THEME_CHANGED, (m) => applyPersistedTheme(m)),
      await listenAppEvent(EVT_SETTINGS_CHANGED, () => this.settings.loadAll()),
      await listenAppEvent(EVT_FOLDERS_CHANGED, () => useFoldersStore().loadAll()),
      await listenAppEvent(EVT_TAGS_CHANGED, () => useTagsStore().loadAll()),
      await listenAppEvent(EVT_SITES_CHANGED, () => useSitesStore().loadAll()),
    );
  },
  beforeUnmount() {
    for (const u of this.unlisteners) u();
  },
  methods: {
    setTab(t: TabKey) { this.tab = t; },
    close() { getCurrentWebviewWindow().close(); },
    async setTheme(mode: "system" | "light" | "dark") {
      log.info(`theme switched to ${mode}`);
      await this.settings.set("theme", mode);
      setThemeMode(mode);
    },
    async setDefaultAction(v: "inject" | "copy_only") {
      await this.settings.set("default_action", v);
    },
    async toggleAutoStart(v: boolean) {
      await this.settings.set("auto_start", v);
    },
    async saveHotkey() {
      log.info(`saving hotkey: ${this.hotkeyDraft}`);
      await this.settings.set("hotkey", this.hotkeyDraft || null);
      try {
        if (this.hotkeyDraft) await registerHotkey(this.hotkeyDraft);
        else await unregisterHotkey();
      } catch (e) {
        log.error(`register hotkey failed: ${e}`);
      }
    },
    async clearHotkey() {
      this.hotkeyDraft = "";
      await this.saveHotkey();
    },
    async recheckAccess() {
      this.accessibilityOk = await permissionsCheckAccessibility();
    },
    async requestAccess() {
      this.accessibilityOk = await permissionsRequestAccessibility();
    },
    async openLogsDir() {
      if (!this.logsPath) return;
      try {
        await revealItemInDir(this.logsPath);
      } catch (e) {
        log.error(`open logs dir failed: ${e}`);
      }
    },
  },
});
</script>

<template>
  <div class="settings">
    <header class="head">
      <div class="hl">
        <SettingsIcon :size="14" class="hl-ico" />
        <span class="brand">设置</span>
      </div>
      <button class="close" @click="close">
        <X :size="14" />
      </button>
    </header>
    <div class="body">
      <nav class="nav">
        <button
          v-for="n in navItems"
          :key="n.key"
          :class="{ on: tab === n.key }"
          @click="setTab(n.key)"
        >
          <component :is="n.icon" :size="14" />
          <span>{{ n.label }}</span>
        </button>
      </nav>
      <main class="main">
        <!-- 常规 -->
        <section v-if="tab === 'general'" class="panel">
          <h3>常规</h3>
          <div class="card">
            <div class="row">
              <div>
                <div class="title">默认动作</div>
                <div class="sub">在抽屉里按 Enter 时执行的操作。</div>
              </div>
            </div>
            <label class="radio-card" :class="{ on: settings.data.default_action === 'inject' }">
              <input type="radio" :checked="settings.data.default_action === 'inject'"
                @change="setDefaultAction('inject')" />
              <div>
                <div class="title">自动注入到上一个聚焦窗口</div>
                <div class="sub">隐藏抽屉 → 模拟粘贴；失败回退仅复制。</div>
              </div>
            </label>
            <label class="radio-card" :class="{ on: settings.data.default_action === 'copy_only' }">
              <input type="radio" :checked="settings.data.default_action === 'copy_only'"
                @change="setDefaultAction('copy_only')" />
              <div>
                <div class="title">仅复制到剪贴板</div>
                <div class="sub">不需要辅助功能权限；用户自行 ⌘V。</div>
              </div>
            </label>
          </div>
          <div class="card">
            <div class="row">
              <div>
                <div class="title">开机自启</div>
                <div class="sub">登录时自动启动并在后台监听快捷键。</div>
              </div>
              <span class="spacer" />
              <label class="switch">
                <input type="checkbox" :checked="settings.data.auto_start"
                  @change="(e) => toggleAutoStart((e.target as HTMLInputElement).checked)" />
                <span />
              </label>
            </div>
          </div>
        </section>

        <!-- 快捷键 -->
        <section v-if="tab === 'hotkey'" class="panel">
          <h3>全局唤起快捷键</h3>
          <p class="lead">
            按下快捷键即可从任意应用中呼出抽屉。建议用三键组合避免冲突（如 ⌘⇧P / Ctrl+Shift+P）。
          </p>
          <div class="card">
            <div class="row">
              <div class="title">打开抽屉</div>
              <span class="spacer" />
              <HotkeyRecorder v-model="hotkeyDraft" />
            </div>
            <div class="row">
              <span class="spacer" />
              <button class="ghost" @click="clearHotkey">清除</button>
              <button class="primary" @click="saveHotkey">应用</button>
            </div>
          </div>
        </section>

        <!-- 主题 -->
        <section v-if="tab === 'theme'" class="panel">
          <h3>主题</h3>
          <div class="theme-row">
            <button
              class="theme-btn"
              :class="{ on: settings.data.theme === 'system' }"
              @click="setTheme('system')"
            >
              <div class="card-prev sys">A·a</div>
              <span>跟随系统</span>
            </button>
            <button
              class="theme-btn"
              :class="{ on: settings.data.theme === 'light' }"
              @click="setTheme('light')"
            >
              <div class="card-prev light">A</div>
              <span>浅色</span>
            </button>
            <button
              class="theme-btn"
              :class="{ on: settings.data.theme === 'dark' }"
              @click="setTheme('dark')"
            >
              <div class="card-prev dark">A</div>
              <span>深色</span>
            </button>
          </div>
        </section>

        <FoldersPanel v-if="tab === 'folders'" />
        <SitesPanel v-if="tab === 'sites'" />
        <DataPanel v-if="tab === 'data'" />

        <!-- 权限诊断 -->
        <section v-if="tab === 'permissions'" class="panel">
          <h3>权限诊断</h3>
          <div class="card">
            <div class="row">
              <div>
                <div class="title">{{ isMacOS ? "macOS 辅助功能" : "键盘模拟权限" }}</div>
                <div class="sub">
                  注入功能需要此权限（AXIsProcessTrusted）。
                  未授权时按 Enter 会回退仅复制。
                </div>
              </div>
              <span class="spacer" />
              <span :class="['status', accessibilityOk ? 'ok' : 'bad']">
                {{ accessibilityOk ? "已授权" : "未授权" }}
              </span>
            </div>
            <div v-if="!accessibilityOk && isMacOS" class="hint">
              点下面按钮 → 系统会弹出引导窗 → 在「辅助功能」里把
              <b>prompt-manager</b> 加入并勾选 → 重启应用使其生效。
            </div>
            <div class="row">
              <span class="spacer" />
              <button class="ghost" @click="recheckAccess">重新检测</button>
              <button v-if="!accessibilityOk && isMacOS" class="primary" @click="requestAccess">
                请求授权
              </button>
            </div>
          </div>
          <div class="card">
            <div class="row">
              <div>
                <div class="title">日志</div>
                <div class="sub">所有操作和错误会写入此目录的 app.log。出问题时把这个文件给开发者。</div>
              </div>
              <span class="spacer" />
              <button class="ghost" @click="openLogsDir">
                <FileText :size="13" /> 打开日志目录
              </button>
            </div>
            <div class="path">{{ logsPath || "(加载中)" }}</div>
          </div>
        </section>

        <!-- 关于 -->
        <section v-if="tab === 'about'" class="panel">
          <h3>关于</h3>
          <div class="card">
            <div class="row">
              <div>
                <div class="title">Prompt Hub</div>
                <div class="sub">v{{ appVersion }} · MIT License</div>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
    <BaseToast />
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-surface);
  color: var(--text-primary);
}
.head {
  height: 48px;
  padding: 0 12px 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  background: var(--bg-titlebar);
  -webkit-app-region: drag;
}
.hl { display: flex; align-items: center; gap: 8px; }
.hl-ico { color: var(--accent); }
.brand { font-weight: 600; font-size: 13px; }
.close {
  width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  -webkit-app-region: no-drag;
}
.close:hover { background: var(--bg-hover); color: var(--text-primary); }

.body { flex: 1; display: flex; min-height: 0; }
.nav {
  width: 160px;
  padding: 12px 8px;
  background: var(--bg-base);
  border-right: 1px solid var(--border);
  display: flex; flex-direction: column; gap: 2px;
}
.nav button {
  height: 30px; padding: 0 10px;
  border-radius: 6px;
  display: flex; align-items: center; gap: 8px;
  text-align: left;
  font-size: 12px;
  color: var(--text-secondary);
}
.nav button:hover { background: var(--bg-hover); }
.nav button.on {
  background: var(--bg-selected);
  color: var(--text-primary);
  font-weight: 600;
  box-shadow: inset 2.5px 0 0 var(--accent);
}

.main { flex: 1; padding: 24px; overflow-y: auto; }

.panel { display: flex; flex-direction: column; gap: 12px; }
.panel h3 { font-size: 14px; font-weight: 600; }
.lead { font-size: 12px; color: var(--text-secondary); line-height: 1.5; }

.card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  background: var(--bg-base);
  display: flex; flex-direction: column; gap: 10px;
}
.row { display: flex; align-items: center; gap: 8px; }
.title { font-size: 13px; font-weight: 500; }
.sub { font-size: 11px; color: var(--text-secondary); margin-top: 2px; line-height: 1.5; }
.spacer { flex: 1; }
.path {
  font-family: var(--font-mono);
  font-size: 10.5px;
  color: var(--text-tertiary);
  background: var(--bg-input);
  padding: 6px 8px;
  border-radius: 6px;
  word-break: break-all;
}

.radio-card {
  display: flex; align-items: center; gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
}
.radio-card.on { border-color: var(--accent); border-width: 1.5px; }
.radio-card input { accent-color: var(--accent); }

.switch {
  position: relative;
  width: 34px; height: 20px;
  display: inline-block;
}
.switch input { position: absolute; opacity: 0; pointer-events: none; }
.switch span {
  position: absolute; inset: 0;
  background: var(--border-strong);
  border-radius: 999px;
  cursor: pointer;
  transition: background var(--dur-fast);
}
.switch span::after {
  content: "";
  position: absolute;
  width: 16px; height: 16px;
  background: #ffffff;
  border-radius: 50%;
  top: 2px; left: 2px;
  transition: left var(--dur-fast) var(--ease-out);
  box-shadow: 0 1px 2px rgba(0,0,0,0.2);
}
.switch input:checked + span { background: var(--accent); }
.switch input:checked + span::after { left: 16px; }

.theme-row { display: flex; gap: 12px; }
.theme-btn {
  width: 120px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-base);
  display: flex; flex-direction: column; gap: 8px; align-items: center;
  font-size: 11px;
  color: var(--text-secondary);
}
.theme-btn.on { border-color: var(--accent); border-width: 1.5px; }
.card-prev {
  width: 80px; height: 50px;
  border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
  font-weight: 700;
}
.card-prev.light { background: #ffffff; color: #18181b; border: 1px solid #ececee; }
.card-prev.dark { background: #17171a; color: #fafafa; border: 1px solid #2a2a2f; }
.card-prev.sys {
  background: linear-gradient(135deg, #ffffff 50%, #17171a 50%);
  color: #888;
  border: 1px solid var(--border);
}

.ghost {
  height: 30px; padding: 0 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  font-size: 12px;
  color: var(--text-secondary);
  display: inline-flex; align-items: center; gap: 6px;
}
.ghost:hover { background: var(--bg-hover); color: var(--text-primary); }
.primary {
  height: 30px; padding: 0 14px;
  border-radius: 6px;
  background: var(--accent);
  color: var(--accent-fg);
  font-size: 12px;
  font-weight: 600;
}

.status { font-size: 11px; padding: 2px 8px; border-radius: 4px; font-weight: 500; }
.status.ok { background: var(--accent-soft); color: var(--success); }
.status.bad { background: var(--accent-soft); color: var(--danger); }
.hint {
  font-size: 11px;
  padding: 8px 10px;
  background: var(--bg-hover);
  border-radius: 6px;
  color: var(--text-secondary);
}
</style>
