<!--
  App.vue — 极薄 router outlet。
  实际界面在 src/views/*View.vue 里，每个对应一个 Tauri 窗口。
-->
<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "App",
});
</script>

<template>
  <router-view />
</template>
