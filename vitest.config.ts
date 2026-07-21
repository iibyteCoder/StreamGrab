import { defineConfig } from "vitest/config";
import { resolve } from "path";

// 纯函数单元测试配置（node 环境，无 DOM 依赖）
export default defineConfig({
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  test: {
    environment: "node",
    include: ["src/**/*.{test,spec}.ts"],
  },
});
