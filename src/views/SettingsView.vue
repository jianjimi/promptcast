<!--
  SettingsView.vue — 设置窗口（独立）。
  左侧 nav，右侧主区。各面板独立组件。
-->
<script lang="ts">
import { defineComponent } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

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
import { permissionsCheckAccessibility } from "../api/inject";
import { setThemeMode, applyPersistedTheme } from "../composables/useTheme";
import { isMac } from "../utils/format";

type TabKey =
  | "general" | "hotkey" | "theme" | "folders"
  | "sites" | "data" | "permissions" | "about";

export default defineComponent({
  name: "SettingsView",
  components: { HotkeyRecorder, FoldersPanel, SitesPanel, DataPanel, BaseToast },
  data() {
    return {
      tab: "hotkey" as TabKey,
      hotkeyDraft: "",
      accessibilityOk: true,
      isMacOS: isMac(),
    };
  },
  computed: {
    settings() { return useSettingsStore(); },
    appVersion(): string { return "0.1.0"; },
  },
  async mounted() {
    if (!this.settings.loaded) await this.settings.loadAll();
    applyPersistedTheme(this.settings.data.theme);
    this.hotkeyDraft = this.settings.data.hotkey ?? "";
    this.accessibilityOk = await permissionsCheckAccessibility();
  },
  methods: {
    setTab(t: TabKey) { this.tab = t; },
    close() { getCurrentWebviewWindow().close(); },
    async setTheme(mode: "system" | "light" | "dark") {
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
      await this.settings.set("hotkey", this.hotkeyDraft || null);
      if (this.hotkeyDraft) await registerHotkey(this.hotkeyDraft);
      else await unregisterHotkey();
    },
    async clearHotkey() {
      this.hotkeyDraft = "";
      await this.saveHotkey();
    },
  },
});
</script>

<template>
  <div class="settings">
    <header class="head">
      <div class="hl">
        <span class="brand">⚙ 设置</span>
      </div>
      <button class="close" @click="close">×</button>
    </header>
    <div class="body">
      <nav class="nav">
        <button :class="{ on: tab === 'general' }" @click="setTab('general')">
          ⚖ 常规
        </button>
        <button :class="{ on: tab === 'hotkey' }" @click="setTab('hotkey')">
          ⌨ 快捷键
        </button>
        <button :class="{ on: tab === 'theme' }" @click="setTab('theme')">
          🎨 主题
        </button>
        <button :class="{ on: tab === 'folders' }" @click="setTab('folders')">
          📁 分类管理
        </button>
        <button :class="{ on: tab === 'sites' }" @click="setTab('sites')">
          🌐 网址快捷
        </button>
        <button :class="{ on: tab === 'data' }" @click="setTab('data')">
          💾 数据
        </button>
        <button :class="{ on: tab === 'permissions' }" @click="setTab('permissions')">
          🛡 权限诊断
        </button>
        <button :class="{ on: tab === 'about' }" @click="setTab('about')">
          ℹ 关于
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

        <!-- 分类 / 网址 / 数据 -->
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
                  注入功能需要此权限。未授权时按 Enter 会回退仅复制。
                </div>
              </div>
              <span class="spacer" />
              <span :class="['status', accessibilityOk ? 'ok' : 'bad']">
                {{ accessibilityOk ? "✓ 已授权" : "✗ 未授权" }}
              </span>
            </div>
            <div v-if="!accessibilityOk && isMacOS" class="hint">
              打开 系统设置 → 隐私与安全 → 辅助功能，把 Prompt Hub 加入并勾选。
            </div>
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
.brand { font-weight: 600; font-size: 13px; }
.close {
  width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 6px;
  font-size: 16px;
  color: var(--text-secondary);
  -webkit-app-region: no-drag;
}
.close:hover { background: var(--bg-hover); }

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
.sub { font-size: 11px; color: var(--text-secondary); margin-top: 2px; }
.spacer { flex: 1; }

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
.switch input {
  position: absolute; opacity: 0; pointer-events: none;
}
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
}
.primary {
  height: 30px; padding: 0 14px;
  border-radius: 6px;
  background: var(--accent);
  color: var(--accent-fg);
  font-size: 12px;
  font-weight: 600;
}

.status { font-size: 11px; padding: 2px 8px; border-radius: 4px; }
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
