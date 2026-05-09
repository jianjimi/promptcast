<!-- HintBar.vue — 底部快捷键提示条。 -->
<script lang="ts">
import { defineComponent } from "vue";
import { modKey } from "../../utils/format";

export default defineComponent({
  name: "HintBar",
  props: {
    count: { type: Number, default: 0 },
  },
  data() {
    return { mod: modKey() };
  },
});
</script>

<template>
  <div class="hint-bar">
    <span><b>↵</b> 注入</span>
    <span class="dot">·</span>
    <span>{{ mod }}C 复制</span>
    <span class="dot">·</span>
    <span>{{ mod }}E 编辑</span>
    <span class="dot">·</span>
    <span>Space 预览</span>
    <span class="spacer" />
    <span class="count">{{ count }} 条</span>
  </div>
</template>

<style scoped>
.hint-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 12px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border);
  font-size: 10px;
  color: var(--text-secondary);
}
.hint-bar b { font-weight: 700; }
.dot { color: var(--border-strong); }
.spacer { flex: 1; }
.count { color: var(--text-tertiary); }
</style>
