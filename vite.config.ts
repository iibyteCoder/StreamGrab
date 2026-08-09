import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
import { readFileSync } from "fs";
import type { Plugin } from "vite";

/**
 * e2e 专用开发插件（仅 VITE_E2E_MOCK=1 时启用）
 *
 * 在 index.html 头部注入 Tauri bridge mock（e2e/support/tauri-mock.js），
 * 让前端在真实浏览器中以真实交互跑完整流程；seed 通过 URL ?e2e_seed=<base64> 传入。
 * 正常 `npm run dev` / 打包完全不受影响。
 */
function e2eMockPlugin(): Plugin {
  const enabled = process.env.VITE_E2E_MOCK === "1";
  if (!enabled) {
    return { name: "streamgrab-e2e-mock" };
  }
  const mockSource = readFileSync(
    resolve(process.cwd(), "e2e/support/tauri-mock.js"),
    "utf8",
  );
  return {
    name: "streamgrab-e2e-mock",
    transformIndexHtml(html: string) {
      return {
        html,
        tags: [
          {
            tag: "script",
            attrs: { type: "text/javascript" },
            children: mockSource,
            injectTo: "head-prepend",
          },
        ],
      };
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), e2eMockPlugin()],
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
