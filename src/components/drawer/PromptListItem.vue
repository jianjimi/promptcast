<!-- PromptListItem.vue — 单条列表项；hover 显示 复制 / 编辑 / 收藏 按钮。 -->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import { Star, Copy, Pencil, Pin } from "lucide-vue-next";
import type { Prompt } from "../../types/prompt";
import { snippet } from "../../utils/format";

export default defineComponent({
  name: "PromptListItem",
  components: { Star, Copy, Pencil, Pin },
  props: {
    prompt: { type: Object as PropType<Prompt>, required: true },
    selected: { type: Boolean, default: false },
  },
  emits: {
    click: (_id: number) => true,
    "toggle-fav": (_id: number) => true,
    copy: (_id: number) => true,
    edit: (_id: number) => true,
  },
  computed: {
    preview(): string {
      return snippet(this.prompt.content, 90);
    },
  },
  methods: {
    stop(e: Event) { e.stopPropagation(); },
    onStar(e: Event) { e.stopPropagation(); this.$emit("toggle-fav", this.prompt.id); },
    onCopy(e: Event) { e.stopPropagation(); this.$emit("copy", this.prompt.id); },
    onEdit(e: Event) { e.stopPropagation(); this.$emit("edit", this.prompt.id); },
  },
});
</script>

<template>
  <div
    class="item"
    :class="{ selected }"
    @click="$emit('click', prompt.id)"
  >
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
      <button class="ico-btn hover-only" @click="onCopy" title="复制">
        <Copy :size="14" />
      </button>
      <button class="ico-btn hover-only" @click="onEdit" title="编辑">
        <Pencil :size="14" />
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
  width: 24px; height: 24px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 5px;
  color: var(--text-tertiary);
  transition: opacity var(--dur-fast) var(--ease-out),
              background var(--dur-fast) var(--ease-out),
              color var(--dur-fast) var(--ease-out);
}
.ico-btn:hover { background: var(--bg-surface); color: var(--text-primary); }
.star-btn.on { color: var(--star); }
.hover-only { opacity: 0; }
.item:hover .hover-only,
.item.selected .hover-only { opacity: 1; }
</style>
