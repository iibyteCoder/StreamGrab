<script setup lang="ts">
/**
 * TitleBar 标题栏组件
 * 自定义窗口标题栏（需要 Tauri 窗口配置 decorations: false）
 */

import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "@/composables";

interface Props {
  title?: string;
}

withDefaults(defineProps<Props>(), {
  title: "StreamGrab",
});

const { theme, setTheme } = useSettings();
const isMaximized = ref(false);
const appWindow = getCurrentWindow();

// 检查窗口是否已最大化
onMounted(async () => {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    // 非 Tauri 环境忽略错误
  }
});

// 窗口控制
const handleMinimize = async () => {
  try {
    await appWindow.minimize();
  } catch {
    // 非 Tauri 环境忽略错误
  }
};

const handleMaximize = async () => {
  try {
    await appWindow.toggleMaximize();
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    // 非 Tauri 环境忽略错误
  }
};

const handleClose = async () => {
  try {
    await appWindow.close();
  } catch {
    // 非 Tauri 环境忽略错误
  }
};

// 主题切换
const toggleTheme = () => {
  setTheme(theme.value === "dark" ? "light" : "dark");
};
</script>

<template>
  <div
    class="titlebar h-8 bg-bg-surface border-b border-border-default flex items-center justify-between px-3 select-none"
    data-tauri-drag-region
  >
    <!-- Logo 和标题 -->
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <svg
        class="w-4 h-4 text-accent-primary"
        viewBox="0 0 24 24"
        fill="currentColor"
      >
        <path
          d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"
        />
      </svg>
      <span class="text-sm font-medium text-text-primary">{{ title }}</span>
    </div>

    <!-- 中间区域（可拖拽） -->
    <div class="flex-1 h-full" data-tauri-drag-region />

    <!-- 右侧控制按钮 -->
    <div class="flex items-center gap-1">
      <!-- 主题切换 -->
      <button
        class="w-7 h-7 flex items-center justify-center rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
        @click="toggleTheme"
        title="切换主题"
      >
        <svg
          v-if="theme === 'dark'"
          class="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
          />
        </svg>
        <svg
          v-else
          class="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
          />
        </svg>
      </button>

      <!-- 设置 -->
      <button
        class="w-7 h-7 flex items-center justify-center rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
        title="设置"
      >
        <svg
          class="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
          />
        </svg>
      </button>

      <!-- 窗口控制（仅在桌面端显示） -->
      <div class="flex items-center ml-2 border-l border-border-default pl-2">
        <button
          class="w-7 h-7 flex items-center justify-center rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
          @click="handleMinimize"
          title="最小化"
        >
          <svg
            class="w-3.5 h-3.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M20 12H4"
            />
          </svg>
        </button>

        <button
          class="w-7 h-7 flex items-center justify-center rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
          @click="handleMaximize"
          title="最大化"
        >
          <svg
            v-if="isMaximized"
            class="w-3.5 h-3.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 4H6a2 2 0 00-2 2v2m0 8v2a2 2 0 002 2h2m8-16h2a2 2 0 012 2v2m0 8v2a2 2 0 01-2 2h-2"
            />
          </svg>
          <svg
            v-else
            class="w-3.5 h-3.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"
            />
          </svg>
        </button>

        <button
          class="w-7 h-7 flex items-center justify-center rounded hover:bg-accent-error/20 text-text-secondary hover:text-accent-error transition-colors"
          @click="handleClose"
          title="关闭"
        >
          <svg
            class="w-3.5 h-3.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  /* 确保 titlebar 在最上层 */
  z-index: 9999;
}
</style>
