<!--
  EditorView.vue — 编辑/新建窗口（独立）。
  含标题、文件夹下拉、标签多选、Markdown textarea、保存/取消/删除。
-->
<script lang="ts">
import { defineComponent } from "vue";
import { promptsGet, promptsCreate, promptsUpdate, promptsDelete } from "../api/prompts";
import { foldersList } from "../api/folders";
import { tagsList, tagsCreate } from "../api/tags";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { Folder } from "../types/folder";
import type { Tag } from "../types/tag";
import BaseToast from "../components/ui/BaseToast.vue";
import { useUIStore } from "../stores/ui";

export default defineComponent({
  name: "EditorView",
  components: { BaseToast },
  data() {
    return {
      isNew: true,
      promptId: null as number | null,
      title: "",
      content: "",
      folderId: null as number | null,
      selectedTagIds: [] as number[],
      newTagInput: "",
      folders: [] as Folder[],
      tags: [] as Tag[],
      loaded: false,
      previewMode: false,
      dirty: false,
    };
  },
  computed: {
    canSave(): boolean { return this.title.trim().length > 0; },
    wordCount(): number { return this.content.length; },
    selectedTags(): Tag[] {
      return this.tags.filter((t) => this.selectedTagIds.includes(t.id));
    },
    unselectedTags(): Tag[] {
      return this.tags.filter((t) => !this.selectedTagIds.includes(t.id));
    },
  },
  async mounted() {
    const id = this.$route.params.id;
    if (id) {
      this.isNew = false;
      this.promptId = Number(id);
    }
    await Promise.all([this.loadMeta(), this.loadPrompt()]);
    this.loaded = true;
    this.$nextTick(() => {
      (this.$refs.titleInput as HTMLInputElement | undefined)?.focus();
    });
    document.addEventListener("keydown", this.onKey);
  },
  beforeUnmount() {
    document.removeEventListener("keydown", this.onKey);
  },
  watch: {
    title() { this.dirty = true; },
    content() { this.dirty = true; },
    folderId() { this.dirty = true; },
    selectedTagIds: { deep: true, handler() { this.dirty = true; } },
  },
  methods: {
    async loadMeta() {
      [this.folders, this.tags] = await Promise.all([foldersList(), tagsList()]);
    },
    async loadPrompt() {
      if (!this.promptId) return;
      const p = await promptsGet(this.promptId);
      this.title = p.title;
      this.content = p.content;
      this.folderId = p.folder_id;
      this.selectedTagIds = [...p.tag_ids];
      this.dirty = false;
    },
    addTag(id: number) {
      if (!this.selectedTagIds.includes(id)) this.selectedTagIds.push(id);
    },
    removeTag(id: number) {
      this.selectedTagIds = this.selectedTagIds.filter((x) => x !== id);
    },
    async createNewTag() {
      const name = this.newTagInput.trim().replace(/^#/, "");
      if (!name) return;
      const t = await tagsCreate(name);
      this.tags.push(t);
      this.addTag(t.id);
      this.newTagInput = "";
    },
    async save() {
      if (!this.canSave) return;
      const draft = {
        title: this.title.trim(),
        content: this.content,
        folder_id: this.folderId,
        tag_ids: this.selectedTagIds,
      };
      try {
        if (this.isNew) {
          await promptsCreate(draft);
        } else if (this.promptId) {
          await promptsUpdate(this.promptId, draft);
        }
        this.dirty = false;
        useUIStore().pushToast("已保存", "success");
        getCurrentWebviewWindow().close();
      } catch (e) {
        useUIStore().pushToast(`保存失败: ${e}`, "danger");
      }
    },
    async cancel() {
      if (this.dirty && !confirm("放弃未保存的修改？")) return;
      getCurrentWebviewWindow().close();
    },
    async remove() {
      if (!this.promptId) return;
      if (!confirm(`确认删除「${this.title}」？此操作不可撤销。`)) return;
      await promptsDelete(this.promptId);
      getCurrentWebviewWindow().close();
    },
    onKey(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        this.save();
      }
    },
  },
});
</script>

<template>
  <div class="editor">
    <header class="head">
      <div class="hl">
        <span class="brand">{{ isNew ? "新建提示词" : "编辑提示词" }}</span>
        <span v-if="dirty" class="dirty">· 未保存</span>
      </div>
      <div class="hr">
        <button class="ghost" @click="previewMode = !previewMode">
          {{ previewMode ? "返回编辑" : "预览" }}
        </button>
      </div>
    </header>

    <div v-if="loaded" class="body">
      <label class="field">
        <span class="label">标题</span>
        <input
          ref="titleInput"
          v-model="title"
          placeholder="给提示词起个名字"
          class="title-input"
        />
      </label>

      <div class="row">
        <label class="field">
          <span class="label">文件夹</span>
          <select v-model="folderId" class="select">
            <option :value="null">(未分类)</option>
            <option v-for="f in folders" :key="f.id" :value="f.id">{{ f.name }}</option>
          </select>
        </label>

        <label class="field">
          <span class="label">标签</span>
          <div class="tag-input">
            <span v-for="t in selectedTags" :key="t.id" class="chip">
              #{{ t.name }}
              <button @click="removeTag(t.id)">×</button>
            </span>
            <select
              v-if="unselectedTags.length"
              class="add-tag"
              @change="(e) => { addTag(Number((e.target as HTMLSelectElement).value)); (e.target as HTMLSelectElement).value = ''; }"
            >
              <option value="">+ 选择</option>
              <option v-for="t in unselectedTags" :key="t.id" :value="t.id">
                {{ t.name }}
              </option>
            </select>
            <input
              v-model="newTagInput"
              placeholder="新建…"
              @keydown.enter.prevent="createNewTag"
              class="new-tag"
            />
          </div>
        </label>
      </div>

      <label class="field grow">
        <div class="content-label-row">
          <span class="label">内容（Markdown）</span>
          <span class="word">{{ wordCount }} 字 · Tab 插入两空格</span>
        </div>
        <textarea
          v-model="content"
          @keydown.tab.prevent="content = content + '  '"
          class="content-input"
          spellcheck="false"
        />
      </label>
    </div>
    <div v-else class="loading">加载中…</div>

    <footer class="footer">
      <button v-if="!isNew" class="ghost danger" @click="remove">删除</button>
      <span class="spacer" />
      <button class="ghost" @click="cancel">取消</button>
      <button class="primary" :disabled="!canSave" @click="save">保存 ⌘S</button>
    </footer>
    <BaseToast />
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
.head {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  background: var(--bg-titlebar);
  -webkit-app-region: drag;
}
.hl { display: flex; align-items: center; gap: 8px; }
.brand { font-weight: 600; font-size: 13px; }
.dirty { font-size: 11px; color: var(--warning); }
.hr { -webkit-app-region: no-drag; }
.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
  overflow-y: auto;
}
.field { display: flex; flex-direction: column; gap: 6px; }
.field.grow { flex: 1; min-height: 0; }
.label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.4px;
}
.title-input {
  height: 40px;
  padding: 0 12px;
  border: 1.5px solid var(--accent);
  border-radius: 8px;
  background: var(--bg-input);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  box-shadow: var(--shadow-inner-input);
  outline: none;
}
.row { display: flex; gap: 12px; }
.row .field { flex: 1; min-width: 0; }
.select {
  height: 36px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-surface);
  color: var(--text-primary);
  font-size: 13px;
}
.tag-input {
  min-height: 36px;
  padding: 4px 6px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-surface);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: var(--accent-soft);
  color: var(--text-primary);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
}
.chip button { font-size: 11px; padding: 0 2px; color: var(--text-secondary); }
.add-tag, .new-tag {
  border: 0;
  background: transparent;
  font-size: 11px;
  color: var(--text-tertiary);
  outline: none;
}
.new-tag { width: 80px; }

.content-label-row {
  display: flex; align-items: center; justify-content: space-between;
}
.word { font-size: 10px; color: var(--text-tertiary); }
.content-input {
  flex: 1;
  min-height: 200px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-input-disabled);
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  resize: none;
  outline: none;
}
.loading { padding: 40px; color: var(--text-tertiary); text-align: center; }
.footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  background: var(--bg-base);
}
.spacer { flex: 1; }
.ghost {
  height: 32px; padding: 0 14px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  font-size: 12px;
  color: var(--text-secondary);
}
.ghost:hover { background: var(--bg-hover); }
.ghost.danger { border-color: transparent; color: var(--danger); }
.primary {
  height: 32px; padding: 0 16px;
  border-radius: 6px;
  background: var(--accent);
  color: var(--accent-fg);
  font-size: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}
.primary:disabled { opacity: 0.4; cursor: not-allowed; }
.primary:hover:not(:disabled) { opacity: 0.92; }
</style>
