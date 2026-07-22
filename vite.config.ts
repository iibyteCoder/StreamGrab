import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version || "0.1.0"),
  },
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
  },
  // To make Tauri work in development mode
  clearScreen: false,
  build: {
    // 桌面应用从磁盘加载，chunk 大小不影响 UX；抑制 >500KB 警告
    chunkSizeWarningLimit: 800,
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-vue": ["vue", "vue-router", "pinia", "vue-i18n"],
          "vendor-ui": ["reka-ui"],
          "vendor-tauri": ["@tauri-apps/api"],
        },
      },
    },
  },
});
