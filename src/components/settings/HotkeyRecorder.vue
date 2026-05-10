<!-- HotkeyRecorder.vue — 录制全局快捷键。 -->
<script lang="ts">
import { defineComponent } from "vue";
import { isMac } from "../../utils/format";

export default defineComponent({
  name: "HotkeyRecorder",
  props: {
    modelValue: { type: String, default: "" },
  },
  emits: {
    "update:modelValue": (_v: string) => true,
  },
  data() {
    return { recording: false };
  },
  computed: {
    display(): string {
      if (!this.modelValue) return this.recording ? "按下组合键…" : "（未设置）";
      return this.modelValue
        .replace(/CmdOrCtrl/g, isMac() ? "⌘" : "Ctrl")
        .replace(/Shift/g, isMac() ? "⇧" : "Shift")
        .replace(/Alt/g, isMac() ? "⌥" : "Alt")
        .replace(/Super/g, isMac() ? "⌘" : "Win")
        .replace(/Meta/g, isMac() ? "⌘" : "Win")
        .replace(/\+/g, " ");
    },
  },
  methods: {
    start() {
      this.recording = true;
      document.addEventListener("keydown", this.onKey, true);
    },
    cancel() {
      this.recording = false;
      document.removeEventListener("keydown", this.onKey, true);
    },
    onKey(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") { this.cancel(); return; }
      // 区分 Win/Cmd 与 Ctrl：
      //   - macOS: metaKey = ⌘ → 用作 CmdOrCtrl（与 Win 上 Ctrl 互通）
      //   - Windows: metaKey = Win 键 → 单独标 Super；ctrlKey 才是 CmdOrCtrl
      const mac = isMac();
      const mods: string[] = [];
      if (mac) {
        if (e.metaKey || e.ctrlKey) mods.push("CmdOrCtrl");
      } else {
        if (e.ctrlKey) mods.push("CmdOrCtrl");
        if (e.metaKey) mods.push("Super");
      }
      if (e.shiftKey) mods.push("Shift");
      if (e.altKey) mods.push("Alt");
      if (["Control", "Meta", "Shift", "Alt"].includes(e.key)) return;
      // 必须至少一个 modifier，否则单键当快捷键太容易冲突。
      if (mods.length === 0) return;
      const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
      const combo = [...mods, key].join("+");
      this.$emit("update:modelValue", combo);
      this.cancel();
    },
  },
});
</script>

<template>
  <div class="hk">
    <div class="combo">{{ display }}</div>
    <button v-if="!recording" class="rec" @click="start">录制</button>
    <button v-else class="rec stop" @click="cancel">取消</button>
  </div>
</template>

<style scoped>
.hk { display: flex; align-items: center; gap: 8px; }
.combo {
  min-width: 120px;
  height: 28px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}
.rec {
  height: 28px; padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-surface);
  font-size: 11px; font-weight: 500;
  color: var(--text-secondary);
}
.rec:hover { background: var(--bg-hover); }
.rec.stop { color: var(--danger); border-color: var(--danger); }
</style>
