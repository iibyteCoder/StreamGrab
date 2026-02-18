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

  prettierConfig,
];
