<!-- PromptList.vue — 列表容器（置顶 / 全部）。 -->
<script lang="ts">
import { defineComponent, type PropType } from "vue";
import PromptListItem from "./PromptListItem.vue";
import type { Prompt } from "../../types/prompt";

export default defineComponent({
  name: "PromptList",
  components: { PromptListItem },
  props: {
    prompts: { type: Array as PropType<Prompt[]>, required: true },
    selectedId: { type: Number as PropType<number | null>, default: null },
  },
  emits: {
    select: (_id: number) => true,
    "toggle-fav": (_id: number) => true,
    copy: (_id: number) => true,
    edit: (_id: number) => true,
    "new-prompt": () => true,
  },
  computed: {
    pinned(): Prompt[] {
      return this.prompts.filter((p) => p.is_pinned);
    },
    rest(): Prompt[] {
      return this.prompts.filter((p) => !p.is_pinned);
    },
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
});
</script>

<template>
  <div class="list">
    <template v-if="pinned.length">
      <div class="section-label">置顶</div>
      <div v-for="p in pinned" :key="p.id" :data-id="p.id">
        <PromptListItem
          :prompt="p"
          :selected="p.id === selectedId"
          @click="$emit('select', p.id)"
          @toggle-fav="$emit('toggle-fav', p.id)"
          @copy="$emit('copy', p.id)"
          @edit="$emit('edit', p.id)"
        />
      </div>
    </template>
    <div v-if="rest.length" class="section-label">全部</div>
    <div v-for="p in rest" :key="p.id" :data-id="p.id">
      <PromptListItem
        :prompt="p"
        :selected="p.id === selectedId"
        @click="$emit('select', p.id)"
        @toggle-fav="$emit('toggle-fav', p.id)"
      />
    </div>
    <div v-if="!prompts.length" class="empty">
      <p>没有匹配的提示词</p>
      <button class="empty-cta" @click="$emit('new-prompt')">+ 新建提示词</button>
      <p class="hint">或按 ⌘N</p>
    </div>
  </div>
</template>

<style scoped>
.list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px 8px;
}
.section-label {
  padding: 8px 4px 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.6px;
  color: var(--text-tertiary);
  text-transform: uppercase;
}
.empty {
  padding: 60px 16px;
  text-align: center;
  color: var(--text-tertiary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
.empty .hint {
  font-family: var(--font-mono);
  font-size: 10px;
}
.empty-cta {
  height: 32px;
  padding: 0 14px;
  border-radius: 6px;
  background: var(--accent);
  color: var(--accent-fg);
  font-size: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}
.empty-cta:hover { opacity: 0.92; }
</style>
