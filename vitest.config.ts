import { defineConfig } from "vitest/config";
import { resolve } from "path";
import vue from "@vitejs/plugin-vue";

// 单元测试配置
// - 纯函数测试默认 node 环境（无 DOM 依赖）
// - 组件测试在文件顶部以 /** @vitest-environment happy-dom */ 声明 DOM 环境
// - vue 别名指向 esm-bundler 构建：组件测试中的 Host 组件使用运行时模板字符串，
//   需要带编译器的构建（仅影响测试，不影响生产构建）
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
      vue: "vue/dist/vue.esm-bundler.js",
    },
  },
  test: {
    environment: "node",
    include: ["src/**/*.{test,spec}.ts"],
  },
});
