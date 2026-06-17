<!--
  ClipboardList.vue — 剪贴板历史列表（「剪贴板」分类 chip 选中时显示）。
  click 选中；Enter / 箭头按钮 = 注入；垃圾桶 = 删除。复用与 PromptList 一致的视觉。
-->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import { Trash2, ArrowRight } from "lucide-vue-next";
import type { ClipEntry } from "../../types/clipboard";
import { snippet, relativeTime } from "../../utils/format";

export default defineComponent({
  name: "ClipboardList",
  components: { Trash2, ArrowRight },
  props: {
    entries: { type: Array as PropType<ClipEntry[]>, required: true },
    selectedId: { type: Number as PropType<number | null>, default: null },
  },
  emits: {
    select: (_id: number) => true,
    inject: (_id: number) => true,
    delete: (_id: number) => true,
  },
  watch: {
    selectedId(id: number | null) {
      if (id == null) return;
      this.$nextTick(() => {
        const el = this.$el?.querySelector?.(`[data-id="${id}"]`) as HTMLElement | null;
        el?.scrollIntoView({ block: "nearest" });
      });
    },
  },
  methods: {
    preview(c: ClipEntry): string { return snippet(c.content, 100); },
    time(c: ClipEntry): string { return relativeTime(c.created_at); },
  },
});
</script>

<template>
  <div class="list">
    <div v-for="c in entries" :key="c.id" :data-id="c.id">
      <div
        class="item"
        :class="{ selected: c.id === selectedId }"
        @click="$emit('select', c.id)"
      >
        <div class="main">
          <div class="text">{{ preview(c) }}</div>
          <div class="meta">{{ time(c) }} · {{ c.char_count }} 字</div>
        </div>
        <div class="actions">
          <button
            type="button"
            class="ico-btn hover-only"
            @click.stop="$emit('inject', c.id)"
            title="注入到上一个输入框 (Enter)"
          >
            <ArrowRight :size="14" />
          </button>
          <button
            type="button"
            class="ico-btn hover-only del"
            @click.stop="$emit('delete', c.id)"
            title="删除"
          >
            <Trash2 :size="14" />
          </button>
        </div>
      </div>
    </div>
    <div v-if="!entries.length" class="empty">
      <p>暂无剪贴板记录</p>
      <p class="hint">复制任意文本后会自动出现在这里</p>
    </div>
  </div>
</template>

<style scoped>
.list { flex: 1; overflow-y: auto; padding: 4px 8px 8px; }
.item {
  display: flex;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  position: relative;
  transition: background var(--dur-fast) var(--ease-out);
}
.item:hover { background: var(--bg-hover); }
.item.selected {
  background: var(--bg-selected);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}
.main { flex: 1; min-width: 0; }
.text {
  font-size: 12.5px;
  color: var(--text-primary);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}
.meta { font-size: 10px; color: var(--text-tertiary); margin-top: 3px; }
.actions { display: flex; gap: 2px; align-items: flex-start; flex-shrink: 0; }
.ico-btn {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 5px;
  color: var(--text-tertiary);
  transition: opacity var(--dur-fast) var(--ease-out),
              background var(--dur-fast) var(--ease-out),
              color var(--dur-fast) var(--ease-out);
}
.ico-btn svg, .ico-btn svg * { pointer-events: none; }
.ico-btn:hover { background: var(--bg-surface); color: var(--text-primary); }
.ico-btn.del:hover { color: var(--danger); }
.hover-only { opacity: 0; }
.item:hover .hover-only,
.item.selected .hover-only { opacity: 1; }
.empty {
  padding: 50px 16px;
  text-align: center;
  color: var(--text-tertiary);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.empty .hint { font-size: 11px; }
</style>
