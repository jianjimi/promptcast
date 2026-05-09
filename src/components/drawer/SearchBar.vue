<!-- SearchBar.vue — 顶栏。整体可拖拽（除按钮 / 搜索框）。 -->
<script lang="ts">
import { defineComponent } from "vue";
import {
  Sparkle, Pin, ArrowUpDown, Settings, Plus, Search,
} from "lucide-vue-next";
import { useUIStore } from "../../stores/ui";
import { usePromptsStore } from "../../stores/prompts";
import {
  windowOpenEditor,
  windowOpenSettings,
  windowSetPin,
} from "../../api/window";
import type { SortMode } from "../../types/prompt";
import { modKey } from "../../utils/format";

export default defineComponent({
  name: "SearchBar",
  components: { Sparkle, Pin, ArrowUpDown, Settings, Plus, Search },
  props: {
    modelValue: { type: String, required: true },
  },
  emits: {
    "update:modelValue": (_v: string) => true,
  },
  data() {
    return {
      sortMenuOpen: false,
      modSymbol: modKey(),
    };
  },
  computed: {
    ui() { return useUIStore(); },
    prompts() { return usePromptsStore(); },
  },
  mounted() {
    document.addEventListener("click", this.onDocClick);
  },
  beforeUnmount() {
    document.removeEventListener("click", this.onDocClick);
  },
  methods: {
    onInput(e: Event) {
      this.$emit("update:modelValue", (e.target as HTMLInputElement).value);
    },
    async togglePin() {
      this.ui.drawerPinned = !this.ui.drawerPinned;
      await windowSetPin(this.ui.drawerPinned);
    },
    async openSettings() { await windowOpenSettings(); },
    async openNew() { await windowOpenEditor(null); },
    async setSort(mode: SortMode) {
      this.sortMenuOpen = false;
      await this.prompts.setSort(mode);
    },
    onDocClick(e: MouseEvent) {
      if (!this.sortMenuOpen) return;
      const wrap = (this.$el as HTMLElement)?.querySelector(".sort-wrap");
      if (wrap && !wrap.contains(e.target as Node)) this.sortMenuOpen = false;
    },
  },
});
</script>

<template>
  <header class="search-bar">
    <div class="title-row">
      <div class="brand">
        <Sparkle :size="14" class="brand-ico" />
        <span class="name">Prompt Hub</span>
      </div>
      <div class="actions">
        <button
          class="icon-btn"
          :class="{ active: ui.drawerPinned }"
          @click="togglePin"
          title="钉住（不自动隐藏）"
        >
          <Pin :size="14" />
        </button>
        <div class="sort-wrap">
          <button class="icon-btn" @click="sortMenuOpen = !sortMenuOpen" title="排序">
            <ArrowUpDown :size="14" />
          </button>
          <div v-if="sortMenuOpen" class="sort-menu">
            <button @click="setSort('recent_used')">最近使用</button>
            <button @click="setSort('created')">创建时间</button>
            <button @click="setSort('updated')">更新时间</button>
            <button @click="setSort('title')">标题 A-Z</button>
          </div>
        </div>
        <button class="icon-btn" @click="openSettings" title="设置">
          <Settings :size="14" />
        </button>
        <span class="divider" />
        <button class="primary-btn" @click="openNew" title="新建">
          <Plus :size="13" />
          <span>新建</span>
        </button>
      </div>
    </div>
    <div class="search-row">
      <Search :size="14" class="search-icon" />
      <input
        :value="modelValue"
        @input="onInput"
        placeholder="搜索提示词…"
      />
      <span class="hint">{{ modSymbol }}F</span>
    </div>
  </header>
</template>

<style scoped>
.search-bar {
  display: flex;
  flex-direction: column;
  background: var(--bg-titlebar);
  border-bottom: 1px solid var(--border);
}
.title-row {
  height: 44px;
  padding: 0 8px 0 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  -webkit-app-region: drag;
}
.brand { display: flex; align-items: center; gap: 8px; }
.brand-ico { color: var(--accent); }
.brand .name { font-weight: 700; font-size: 13px; letter-spacing: 0.2px; }

.actions {
  display: flex; align-items: center; gap: 2px;
  -webkit-app-region: no-drag;
}
.icon-btn {
  width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: background var(--dur-fast) var(--ease-out);
}
.icon-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.icon-btn.active { background: var(--accent-soft); color: var(--accent); }
.divider { width: 1px; height: 16px; background: var(--border); margin: 0 4px; }

.primary-btn {
  height: 28px;
  padding: 0 12px;
  display: flex; align-items: center; gap: 4px;
  background: var(--accent);
  color: var(--accent-fg);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}
.primary-btn:hover { opacity: 0.92; }

.search-row {
  height: 50px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}
.search-row input {
  flex: 1;
  height: 32px;
  padding: 0 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 8px;
  outline: none;
  color: var(--text-primary);
  box-shadow: var(--shadow-inner-input);
  font-size: 13px;
}
.search-row input:focus { border-color: var(--text-secondary); }
.search-icon { color: var(--text-tertiary); flex-shrink: 0; }
.hint {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.sort-wrap { position: relative; }
.sort-menu {
  position: absolute;
  top: 32px; right: 0;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--shadow-md);
  padding: 4px;
  display: flex; flex-direction: column;
  min-width: 120px;
  z-index: 10;
}
.sort-menu button {
  text-align: left;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-primary);
}
.sort-menu button:hover { background: var(--bg-hover); }
</style>
