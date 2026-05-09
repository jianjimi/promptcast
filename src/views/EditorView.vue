<!--
  EditorView.vue — 编辑窗口（600×560）。
  M3 实现：标题 / 文件夹 / 标签 / Markdown 编辑 / 保存。
-->
<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "EditorView",
  computed: {
    isNew(): boolean {
      return !this.$route.params.id;
    },
  },
});
</script>

<template>
  <div class="editor">
    <header class="editor-header">
      {{ isNew ? "新建提示词" : "编辑提示词" }}
    </header>
    <main class="editor-body">
      <p class="placeholder">M0 骨架 · 待 M3 实现编辑表单</p>
    </main>
  </div>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-surface);
  color: var(--text-primary);
}
.editor-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 13px;
}
.editor-body {
  flex: 1;
  padding: 24px;
}
.placeholder {
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
