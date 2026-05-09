<!--
  DrawerView.vue — 主抽屉窗口（400×720）。
  组合 SearchBar / FilterChips / PromptList / SiteLauncher / HintBar。
-->
<script lang="ts">
import { defineComponent } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";

import SearchBar from "../components/drawer/SearchBar.vue";
import FilterChips from "../components/drawer/FilterChips.vue";
import PromptList from "../components/drawer/PromptList.vue";
import SiteLauncher from "../components/drawer/SiteLauncher.vue";
import HintBar from "../components/drawer/HintBar.vue";
import BaseToast from "../components/ui/BaseToast.vue";

import { usePromptsStore } from "../stores/prompts";
import { useFoldersStore } from "../stores/folders";
import { useTagsStore } from "../stores/tags";
import { useSitesStore } from "../stores/sites";
import { useSettingsStore } from "../stores/settings";
import { useUIStore } from "../stores/ui";

import { buildSearchable, searchPrompts } from "../composables/useFuzzySearch";
import { applyPersistedTheme } from "../composables/useTheme";
import {
  listenAppEvent,
  EVT_PROMPTS_CHANGED,
  EVT_FOLDERS_CHANGED,
  EVT_TAGS_CHANGED,
  EVT_SITES_CHANGED,
  EVT_SETTINGS_CHANGED,
  EVT_THEME_CHANGED,
} from "../composables/useAppEvents";
import {
  injectPaste,
  injectCopyOnly,
} from "../api/inject";
import {
  windowOpenEditor,
  windowOpenPreview,
  windowHideDrawer,
} from "../api/window";
import { log } from "../utils/logger";

import type { Prompt } from "../types/prompt";
import type { ThemeMode } from "../types/settings";

export default defineComponent({
  name: "DrawerView",
  components: {
    SearchBar, FilterChips, PromptList, SiteLauncher, HintBar, BaseToast,
  },
  data() {
    return {
      query: "",
      unlisteners: [] as UnlistenFn[],
    };
  },
  computed: {
    prompts() { return usePromptsStore(); },
    settings() { return useSettingsStore(); },
    ui() { return useUIStore(); },
    folders() { return useFoldersStore(); },
    tags() { return useTagsStore(); },
    chipFiltered(): Prompt[] {
      const list = this.prompts.list;
      const k = this.ui.activeChipKey;
      if (k === "all") return list;
      if (k === "favorites") return list.filter((p) => p.is_favorite);
      if (k.startsWith("folder:")) {
        const id = Number(k.slice(7));
        return list.filter((p) => p.folder_id === id);
      }
      if (k.startsWith("tag:")) {
        const id = Number(k.slice(4));
        return list.filter((p) => p.tag_ids.includes(id));
      }
      return list;
    },
    searched(): Prompt[] {
      if (!this.query.trim()) return this.chipFiltered;
      const { items } = buildSearchable(this.chipFiltered, this.tags.list);
      return searchPrompts(items, this.query).map((it) => it.raw);
    },
    counts(): Record<string, number> {
      const list = this.prompts.list;
      const out: Record<string, number> = {
        all: list.length,
        favorites: list.filter((p) => p.is_favorite).length,
      };
      for (const f of this.folders.list) {
        out[`folder:${f.id}`] = list.filter((p) => p.folder_id === f.id).length;
      }
      return out;
    },
    selectedPrompt(): Prompt | null {
      const id = this.prompts.selectedId;
      if (id == null) return null;
      return this.searched.find((p) => p.id === id)
        ?? this.prompts.list.find((p) => p.id === id)
        ?? null;
    },
  },
  watch: {
    searched(list: Prompt[]) {
      if (!list.find((p) => p.id === this.prompts.selectedId)) {
        this.prompts.select(list[0]?.id ?? null);
      }
    },
  },
  async mounted() {
    log.info("DrawerView mounted");
    await Promise.all([
      this.settings.loadAll(),
      this.folders.loadAll(),
      this.tags.loadAll(),
      this.sites().loadAll(),
      this.prompts.loadAll(),
    ]);
    applyPersistedTheme(this.settings.data.theme);
    document.addEventListener("keydown", this.onKey);
    await this.subscribeEvents();
  },
  beforeUnmount() {
    document.removeEventListener("keydown", this.onKey);
    for (const u of this.unlisteners) u();
  },
  methods: {
    sites() { return useSitesStore(); },
    async subscribeEvents() {
      this.unlisteners.push(
        await listenAppEvent(EVT_PROMPTS_CHANGED, () => this.prompts.loadAll()),
        await listenAppEvent(EVT_FOLDERS_CHANGED, () => this.folders.loadAll()),
        await listenAppEvent(EVT_TAGS_CHANGED, () => this.tags.loadAll()),
        await listenAppEvent(EVT_SITES_CHANGED, () => this.sites().loadAll()),
        await listenAppEvent(EVT_SETTINGS_CHANGED, () => this.settings.loadAll()),
        await listenAppEvent<ThemeMode>(EVT_THEME_CHANGED, (m) => applyPersistedTheme(m)),
      );
    },
    focusSearch() {
      const inp = document.querySelector<HTMLInputElement>(".search-row input");
      inp?.focus(); inp?.select();
    },
    moveSel(dir: 1 | -1) {
      const list = this.searched;
      if (!list.length) return;
      const cur = list.findIndex((p) => p.id === this.prompts.selectedId);
      const next = (cur + dir + list.length) % list.length;
      this.prompts.select(list[next].id);
    },
    async injectSelected() {
      const p = this.selectedPrompt;
      if (!p) return;
      log.info(`inject prompt id=${p.id}`);
      if (this.settings.data.default_action === "copy_only") {
        await this.copySelected();
        return;
      }
      try {
        const r = await injectPaste(p.content);
        await this.prompts.recordUse(p.id);
        if (!r.ok) this.ui.pushToast("注入失败 · 已复制到剪贴板", "warning");
      } catch (e) {
        log.error(`inject failed: ${e}`);
        this.ui.pushToast(`注入失败: ${e}`, "danger");
      }
    },
    async copySelected() {
      const p = this.selectedPrompt;
      if (!p) return;
      log.info(`copy prompt id=${p.id}`);
      await injectCopyOnly(p.content);
      await this.prompts.recordUse(p.id);
      this.ui.pushToast("已复制到剪贴板", "success");
    },
    async editSelected() {
      const p = this.selectedPrompt;
      if (p) await windowOpenEditor(p.id);
    },
    async previewSelected() {
      const p = this.selectedPrompt;
      if (p) await windowOpenPreview(p.id);
    },
    async newPrompt() {
      log.info("new prompt window opening");
      await windowOpenEditor(null);
    },
    async toggleFav(id: number) { await this.prompts.toggleFavorite(id); },
    async copyById(id: number) {
      this.prompts.select(id);
      await this.copySelected();
    },
    async editById(id: number) {
      await windowOpenEditor(id);
    },
    onKey(e: KeyboardEvent) {
      const cmd = e.metaKey || e.ctrlKey;
      const target = e.target as HTMLElement | null;
      const inEditable =
        target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA");
      if (e.key === "ArrowDown") { e.preventDefault(); this.moveSel(1); return; }
      if (e.key === "ArrowUp") { e.preventDefault(); this.moveSel(-1); return; }
      if (e.key === "Enter" && !inEditable) {
        e.preventDefault(); this.injectSelected(); return;
      }
      if (e.key === "Enter" && inEditable && target?.tagName === "INPUT") {
        e.preventDefault(); this.injectSelected(); return;
      }
      if (cmd && (e.key === "c" || e.key === "C") && !inEditable) {
        e.preventDefault(); this.copySelected(); return;
      }
      if (cmd && (e.key === "e" || e.key === "E")) {
        e.preventDefault(); this.editSelected(); return;
      }
      if (cmd && (e.key === "n" || e.key === "N")) {
        e.preventDefault(); this.newPrompt(); return;
      }
      if (cmd && (e.key === "f" || e.key === "F")) {
        e.preventDefault(); this.focusSearch(); return;
      }
      if (e.key === " " && !inEditable) {
        e.preventDefault(); this.previewSelected(); return;
      }
      if (e.key === "Escape") {
        if (!this.ui.drawerPinned) windowHideDrawer();
        return;
      }
    },
  },
});
</script>

<template>
  <div class="drawer">
    <SearchBar v-model="query" />
    <FilterChips :counts="counts" />
    <PromptList
      :prompts="searched"
      :selected-id="prompts.selectedId"
      @select="(id: number) => prompts.select(id)"
      @toggle-fav="toggleFav"
      @copy="copyById"
      @edit="editById"
      @new-prompt="newPrompt"
    />
    <SiteLauncher />
    <HintBar :count="searched.length" />
    <BaseToast />
  </div>
</template>

<style scoped>
.drawer {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-base);
  color: var(--text-primary);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border);
}
</style>
