<!-- FilterChips.vue — 横向 chip 筛选；Tab/Shift+Tab 循环。 -->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import { useUIStore } from "../../stores/ui";
import { useFoldersStore } from "../../stores/folders";
import { useTagsStore } from "../../stores/tags";
import type { Folder } from "../../types/folder";
import type { Tag } from "../../types/tag";

interface Chip {
  key: string;
  label: string;
  icon?: string;
}

export default defineComponent({
  name: "FilterChips",
  props: {
    counts: {
      type: Object as PropType<Record<string, number>>,
      default: () => ({}),
    },
  },
  computed: {
    ui() { return useUIStore(); },
    chips(): Chip[] {
      const folders = useFoldersStore().list as Folder[];
      const tags = useTagsStore().list as Tag[];
      return [
        { key: "all", label: "全部" },
        { key: "favorites", label: "收藏", icon: "★" },
        { key: "clipboard", label: "剪贴板", icon: "📋" },
        ...folders.map((f) => ({ key: `folder:${f.id}`, label: f.name })),
        ...tags.map((t) => ({ key: `tag:${t.id}`, label: `#${t.name}` })),
      ];
    },
    activeIdx(): number {
      return Math.max(0, this.chips.findIndex((c) => c.key === this.ui.activeChipKey));
    },
  },
  watch: {
    // 删除当前筛选的文件夹/标签后，对应 chip 消失：把选中回退到「全部」，
    // 否则列表会基于已删 id 过滤成空、高亮也对不上。
    chips(list: Chip[]) {
      if (!list.find((c) => c.key === this.ui.activeChipKey)) {
        this.ui.activeChipKey = "all";
      }
    },
  },
  mounted() {
    document.addEventListener("keydown", this.onKey, true);
  },
  beforeUnmount() {
    document.removeEventListener("keydown", this.onKey, true);
  },
  methods: {
    onKey(e: KeyboardEvent) {
      if (e.key !== "Tab") return;
      // Tab 仅在 chip 间循环；如果焦点在输入框 / textarea 内仍允许
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      if (tag === "TEXTAREA" || tag === "INPUT") return; // 输入框里 Tab 不拦截
      e.preventDefault();
      const len = this.chips.length;
      if (len === 0) return;
      const dir = e.shiftKey ? -1 : 1;
      const next = (this.activeIdx + dir + len) % len;
      this.ui.activeChipKey = this.chips[next].key;
    },
    select(key: string) {
      this.ui.activeChipKey = key;
    },
    countOf(key: string): number | null {
      const v = this.counts[key];
      return Number.isFinite(v) ? v : null;
    },
  },
});
</script>

<template>
  <div class="chips">
    <button
      v-for="c in chips"
      :key="c.key"
      class="chip"
      :class="{ active: c.key === ui.activeChipKey }"
      @click="select(c.key)"
    >
      <span v-if="c.icon" class="ico">{{ c.icon }}</span>
      <span>{{ c.label }}</span>
      <span v-if="countOf(c.key) !== null" class="count">{{ countOf(c.key) }}</span>
    </button>
  </div>
</template>

<style scoped>
.chips {
  display: flex;
  gap: 6px;
  padding: 0 12px 8px;
  overflow-x: auto;
  scrollbar-width: none;
  border-bottom: 1px solid var(--border);
  background: var(--bg-titlebar);
}
.chips::-webkit-scrollbar { display: none; }

.chip {
  flex-shrink: 0;
  height: 24px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border-radius: 999px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 500;
  transition: all var(--dur-fast) var(--ease-out);
}
.chip:hover { background: var(--bg-hover); }
.chip.active {
  background: var(--accent);
  color: var(--accent-fg);
  border-color: var(--accent);
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}
.chip .ico { font-size: 10px; }
.chip .count { color: var(--text-tertiary); font-size: 10px; }
.chip.active .count { color: var(--accent-fg); opacity: 0.7; }
</style>
