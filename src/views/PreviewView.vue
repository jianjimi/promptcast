<!--
  PreviewView.vue — 预览窗口（540×640）。
  M2.5 实现：通过 IPC 拿 prompt → 渲染 Markdown → 注入/复制操作条。
-->
<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "PreviewView",
  computed: {
    promptId(): string {
      return (this.$route.params.id as string) ?? "";
    },
  },
});
</script>

<template>
  <div class="preview">
    <header class="preview-header">预览 · {{ promptId || "(empty)" }}</header>
    <main class="preview-body">
      <p class="placeholder">M0 骨架 · 待 M2.5 实现 Markdown 渲染与操作条</p>
    </main>
  </div>
</template>

<style scoped>
.preview {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-surface);
  color: var(--text-primary);
}
.preview-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 13px;
}
.preview-body {
  flex: 1;
  padding: 20px;
}
.placeholder {
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
