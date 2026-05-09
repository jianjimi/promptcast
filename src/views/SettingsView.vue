<!--
  SettingsView.vue — 设置窗口（680×560）。
  M3 实现：常规 / 快捷键 / 主题 / 分类管理 / 网址快捷 / 数据 / 权限诊断 / 关于。
-->
<script lang="ts">
import { defineComponent } from "vue";

type TabKey =
  | "general"
  | "hotkey"
  | "theme"
  | "folders"
  | "sites"
  | "data"
  | "permissions"
  | "about";

export default defineComponent({
  name: "SettingsView",
  data() {
    return {
      activeTab: "hotkey" as TabKey,
    };
  },
});
</script>

<template>
  <div class="settings">
    <header class="settings-header">设置 · {{ activeTab }}</header>
    <main class="settings-body">
      <p class="placeholder">M0 骨架 · 待 M3 实现各设置面板</p>
    </main>
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
.settings-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 13px;
}
.settings-body {
  flex: 1;
  padding: 24px;
}
.placeholder {
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
