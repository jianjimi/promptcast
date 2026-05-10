<!--
  PromptListItem.vue — 列表项；hover 显示「注入 / 编辑 / 收藏」。
  单击行 = 选中（不注入）；hover 上的紫色注入图标 = 直接注入这条。
-->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import { Star, ArrowRight, Pencil, Pin } from "lucide-vue-next";
import type { Prompt } from "../../types/prompt";
import { snippet } from "../../utils/format";

export default defineComponent({
  name: "PromptListItem",
  components: { Star, ArrowRight, Pencil, Pin },
  props: {
    prompt: { type: Object as PropType<Prompt>, required: true },
    selected: { type: Boolean, default: false },
  },
  emits: {
    click: (_id: number) => true,
    "toggle-fav": (_id: number) => true,
    inject: (_id: number) => true,
    edit: (_id: number) => true,
  },
  computed: {
    preview(): string {
      return snippet(this.prompt.content, 90);
    },
  },
  methods: {
    onStar(e: Event) {
      e.stopPropagation();
      console.log("[ListItem] star click id=", this.prompt.id);
      this.$emit("toggle-fav", this.prompt.id);
    },
    onInject(e: Event) {
      e.stopPropagation();
      console.log("[ListItem] inject click id=", this.prompt.id);
      this.$emit("inject", this.prompt.id);
    },
    onEdit(e: Event) {
      e.stopPropagation();
      console.log("[ListItem] edit click id=", this.prompt.id);
      this.$emit("edit", this.prompt.id);
    },
  },
});
</script>

<template>
  <div class="item" :class="{ selected }" @click="$emit('click', prompt.id)">
    <div class="main">
      <div class="title-row">
        <Pin v-if="prompt.is_pinned" :size="11" class="pin" />
        <span class="title">{{ prompt.title }}</span>
      </div>
      <div class="content">{{ preview }}</div>
    </div>

    <div class="actions">
      <button
        class="ico-btn star-btn"
        :class="{ on: prompt.is_favorite }"
        @click="onStar"
        title="收藏"
      >
        <Star :size="14" :fill="prompt.is_favorite ? 'currentColor' : 'none'" />
      </button>
      <button class="ico-btn hover-only" @click="onEdit" title="编辑 (⌘E)">
        <Pencil :size="14" />
      </button>
      <button
        class="ico-btn inject-btn hover-only"
        @click="onInject"
        title="注入到上一个输入框 (Enter)"
      >
        <ArrowRight :size="14" />
      </button>
    </div>
  </div>
</template>

<style scoped>
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
  box-shadow: inset 2.5px 0 0 var(--accent);
}
.main { flex: 1; min-width: 0; }
.title-row {
  display: flex; gap: 4px; align-items: center;
  margin-bottom: 2px;
}
.pin { color: var(--text-tertiary); }
.title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.content {
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.actions {
  display: flex;
  gap: 2px;
  align-items: flex-start;
  flex-shrink: 0;
}
.ico-btn {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 5px;
  color: var(--text-tertiary);
  transition: opacity var(--dur-fast) var(--ease-out),
              background var(--dur-fast) var(--ease-out),
              color var(--dur-fast) var(--ease-out);
}
.ico-btn:hover { background: var(--bg-surface); color: var(--text-primary); }
.star-btn.on { color: var(--star); }
.inject-btn:hover {
  background: var(--accent);
  color: var(--accent-fg);
}
.hover-only { opacity: 0; }
.item:hover .hover-only,
.item.selected .hover-only { opacity: 1; }
</style>
