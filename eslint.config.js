import js from "@eslint/js";
import pluginVue from "eslint-plugin-vue";
import vueTsEslintConfig from "@vue/eslint-config-typescript";
import prettierConfig from "@vue/eslint-config-prettier";

export default [
  {
    name: "app/files-to-lint",
    files: ["**/*.{ts,mts,tsx,vue}"],
  },

  {
    name: "app/files-to-ignore",
    ignores: [
      "**/dist/**",
      "**/dist-ssr/**",
      "**/coverage/**",
      "**/node_modules/**",
      "**/src-tauri/**",
      "scripts/**",
      "*.config.js",
      "*.config.ts",
    ],
  },

  js.configs.recommended,
  ...pluginVue.configs["flat/essential"],
  ...vueTsEslintConfig(),

  {
    name: "app/rules",
    rules: {
      "vue/multi-word-component-names": "off",
      "vue/no-unused-vars": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
    },
  },

  {
    name: "e2e/scripts",
    files: ["e2e/**/*.mjs", "e2e/support/tauri-mock.js"],
    languageOptions: {
      globals: {
        // Node
        process: "readonly",
        Buffer: "readonly",
        console: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        fetch: "readonly",
        URL: "readonly",
        URLSearchParams: "readonly",
        // Browser（tauri-mock.js 运行在页面内）
        window: "readonly",
        document: "readonly",
        location: "readonly",
        navigator: "readonly",
        sessionStorage: "readonly",
        localStorage: "readonly",
        atob: "readonly",
        TextDecoder: "readonly",
        DataTransfer: "readonly",
        DragEvent: "readonly",
        PointerEvent: "readonly",
        MouseEvent: "readonly",
        CustomEvent: "readonly",
        Event: "readonly",
        HTMLInputElement: "readonly",
        HTMLTextAreaElement: "readonly",
        MutationObserver: "readonly",
      },
    },
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },

  prettierConfig,
];
