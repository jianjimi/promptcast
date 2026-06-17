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
  <!-- :key 强制按路由（含 /editor/:id、/preview/:id）重挂载：
       单例窗口被复用切到另一条 prompt 时，mounted() 会重跑、重新加载数据，
       避免「编辑 B 实际覆盖 A」的脏数据 bug。 -->
  <router-view :key="$route.fullPath" />
</template>
