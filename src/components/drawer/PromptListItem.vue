<!-- PromptListItem.vue — 单条列表项。 -->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import type { Prompt } from "../../types/prompt";
import { snippet } from "../../utils/format";

export default defineComponent({
  name: "PromptListItem",
  props: {
    prompt: { type: Object as PropType<Prompt>, required: true },
    selected: { type: Boolean, default: false },
  },
  emits: {
    click: (_id: number) => true,
    "toggle-fav": (_id: number) => true,
  },
  computed: {
    preview(): string {
      return snippet(this.prompt.content, 90);
    },
  },
  methods: {
    onStarClick(e: Event) {
      e.stopPropagation();
      this.$emit("toggle-fav", this.prompt.id);
    },
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
        <span v-if="prompt.is_pinned" class="pin">📌</span>
        <span class="title">{{ prompt.title }}</span>
      </div>
      <div class="content">{{ preview }}</div>
    </div>
    <button class="star" :class="{ on: prompt.is_favorite }" @click="onStarClick">
      ★
    </button>
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
.pin { font-size: 9px; opacity: 0.7; }
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
.star {
  width: 22px; height: 22px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 4px;
  font-size: 14px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.star:hover { background: var(--bg-hover); }
.star.on { color: var(--star); }
</style>
