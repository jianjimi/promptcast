<!--
  DrawerView.vue — 主抽屉窗口（400×720 窄高条）。
  M0 阶段仅占位骨架。M2 实现真实 UI（搜索 / 筛选 / 列表 / 网址栏 / hint 条）。
-->
<script lang="ts">
import { defineComponent } from "vue";
import { ping } from "../api";

export default defineComponent({
  name: "DrawerView",
  data() {
    return {
      version: "0.1.0",
      backendStatus: "(checking…)",
    };
  },
  async mounted() {
    try {
      this.backendStatus = await ping();
    } catch (e) {
      this.backendStatus = `backend error: ${String(e)}`;
    }
  },
});
</script>

<template>
  <div class="drawer">
    <header class="drawer-header">
      <span class="brand">Prompt Hub</span>
      <span class="version">v{{ version }}</span>
    </header>
    <main class="drawer-body">
      <p class="placeholder">M0 骨架就绪 · 待 M2 实现搜索 / 筛选 / 列表</p>
      <p class="status">{{ backendStatus }}</p>
    </main>
  </div>
</template>

<style scoped>
.drawer {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-base);
  color: var(--text-primary);
  border-radius: 12px;
  overflow: hidden;
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 52px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-titlebar);
}

.brand {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.version {
  font-size: 11px;
  color: var(--text-tertiary);
}

.drawer-body {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.placeholder {
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}
.status {
  margin-top: 8px;
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-tertiary);
  text-align: center;
}
</style>
