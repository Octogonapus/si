import { createApp } from "vue";
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { createHead } from "@vueuse/head";
import VueSafeTeleport from "vue-safe-teleport";
import Toast, { PluginOptions } from "vue-toastification";
import "vue-toastification/dist/index.css";

import "@si/vue-lib/tailwind/main.css";
import "@si/vue-lib/tailwind/tailwind.css";

import App from "@/App.vue";
import "./utils/posthog";
import router from "./router";
import store from "./store";

const app = createApp(App);

app.use(createHead());
app.use(router);
app.use(store);

// we attach to the #app-layout div (in AppLayout.vue) to stay within an overflow hidden div and not mess with page scrollbars
app.use(FloatingVue, {
  container: "#app-layout",
  themes: {
    "user-info": {
      $extend: "tooltip",
      delay: { show: 10, hide: 100 },
      instantMove: true,
      html: true,
    },
  },
});

const toastOptions: PluginOptions = {
  // TODO(Wendy) - any options we want to configure for vue-toastification go here
};

app.use(Toast, toastOptions);

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.use(VueSafeTeleport);

app.mount("#app");
