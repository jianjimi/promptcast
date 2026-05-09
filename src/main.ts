// main.ts — Vue 入口：注册 Pinia、Router、初始化主题。
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { initTheme } from "./composables/useTheme";

import "./styles/reset.css";
import "./styles/tokens.css";
import "./styles/theme-light.css";
import "./styles/theme-dark.css";
import "./styles/global.css";

initTheme();

createApp(App).use(createPinia()).use(router).mount("#app");
