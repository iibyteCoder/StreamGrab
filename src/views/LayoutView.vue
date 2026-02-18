<script setup lang="ts">
/**
 * LayoutView - 主布局框架
 * 嵌套路由的父组件，提供统一的外层结构
 */

import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AppIcon } from '@/components/common';
import { useDownloader, useSettings } from '@/composables';

// Tauri 2.0 缩放方向类型
type ResizeDirection = 'North' | 'South' | 'East' | 'West' | 'NorthEast' | 'NorthWest' | 'SouthEast' | 'SouthWest';

const route = useRoute();
const router = useRouter();

// 窗口控制
const appWindow = getCurrentWindow();
const isMaximized = ref(false);

// 下载器版本
const downloaderVersion = ref('');
const { checkDownloaderAvailable, getDownloaderVersion } = useDownloader();
const { theme, setTheme } = useSettings();

// 当前是否为首页
const isHome = computed(() => route.name === 'home');

// 初始化
onMounted(async () => {
  const available = await checkDownloaderAvailable();
  if (available) {
    downloaderVersion.value = await getDownloaderVersion();
  }
  // 检查窗口最大化状态
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    // 非 Tauri 环境忽略
  }
});

// 返回首页
const goBack = () => router.push('/');

// 跳转设置
const goSettings = () => router.push('/settings');

// 窗口控制
const handleMinimize = async () => {
  try {
    await appWindow.minimize();
  } catch {
    // 非 Tauri 环境忽略
  }
};

const handleMaximize = async () => {
  try {
    await appWindow.toggleMaximize();
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    // 非 Tauri 环境忽略
  }
};

const handleClose = async () => {
  try {
    await appWindow.close();
  } catch {
    // 非 Tauri 环境忽略
  }
};

// 方向映射：简写 -> Tauri API 格式
const directionMap: Record<string, ResizeDirection> = {
  'n': 'North',
  's': 'South',
  'e': 'East',
  'w': 'West',
  'ne': 'NorthEast',
  'nw': 'NorthWest',
  'se': 'SouthEast',
  'sw': 'SouthWest',
};

// 启动窗口缩放
const startResize = (direction: string) => async (e: MouseEvent) => {
  e.preventDefault();
  const dir = directionMap[direction];
  if (!dir) return;
  try {
    await appWindow.startResizeDragging(dir);
  } catch {
    // 非 Tauri 环境忽略
  }
};

// 主题切换
const toggleTheme = () => {
  setTheme(theme.value === 'dark' ? 'light' : 'dark');
};
</script>

<template>
  <div class="app-container flex h-full flex-col bg-background">
    <!-- 窗口缩放边框 -->
    <div class="resize-handles">
      <div class="resize-edge top" @mousedown="startResize('n')" />
      <div class="resize-edge bottom" @mousedown="startResize('s')" />
      <div class="resize-edge left" @mousedown="startResize('w')" />
      <div class="resize-edge right" @mousedown="startResize('e')" />
      <div class="resize-corner top-left" @mousedown="startResize('nw')" />
      <div class="resize-corner top-right" @mousedown="startResize('ne')" />
      <div class="resize-corner bottom-left" @mousedown="startResize('sw')" />
      <div class="resize-corner bottom-right" @mousedown="startResize('se')" />
    </div>

    <!-- 自定义标题栏 -->
    <header
      class="titlebar shrink-0 bg-card border-b flex items-center h-9 px-3 select-none"
      data-tauri-drag-region
    >
      <!-- 左侧：Logo + 标题 -->
      <div class="flex items-center gap-2" data-tauri-drag-region>
        <div class="flex h-6 w-6 items-center justify-center rounded-md bg-primary/10">
          <AppIcon name="Download" :size="14" class="text-primary" />
        </div>
        <span class="text-sm font-medium">StreamGrab</span>
      </div>

      <!-- 中间可拖拽区域 -->
      <div class="flex-1 h-full" data-tauri-drag-region />

      <!-- 右侧控制按钮 -->
      <div class="flex items-center gap-0.5">
        <!-- 主题切换 -->
        <button
          class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          @click="toggleTheme"
          title="切换主题"
        >
          <svg v-if="theme === 'dark'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
          </svg>
        </button>

        <!-- 设置按钮（仅首页） -->
        <button
          v-if="isHome"
          class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          @click="goSettings"
          title="设置"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>

        <!-- 返回按钮（非首页） -->
        <button
          v-else
          class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          @click="goBack"
          title="返回"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
        </button>

        <!-- 窗口控制按钮 -->
        <div class="flex items-center ml-1 pl-1 border-l border-border">
          <button
            class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
            @click="handleMinimize"
            title="最小化"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
          </button>

          <button
            class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
            @click="handleMaximize"
            title="最大化"
          >
            <svg v-if="isMaximized" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 4H6a2 2 0 00-2 2v2m0 8v2a2 2 0 002 2h2m8-16h2a2 2 0 012 2v2m0 8v2a2 2 0 01-2 2h-2" />
            </svg>
            <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
            </svg>
          </button>

          <button
            class="window-btn h-7 w-7 flex items-center justify-center rounded hover:bg-destructive/20 text-muted-foreground hover:text-destructive transition-colors"
            @click="handleClose"
            title="关闭"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- 主内容区：子路由出口 -->
    <main class="flex-1 min-h-0 overflow-hidden">
      <router-view />
    </main>

    <!-- 底部状态栏（仅首页显示） -->
    <footer
      v-if="isHome"
      class="border-t px-6 py-2 shrink-0 text-xs text-muted-foreground bg-card/80 text-right"
    >
      {{ downloaderVersion || 'N_m3u8DL-RE' }}
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  /* 窗口阴影 */
  box-shadow:
    0 0 0 1px rgba(255, 255, 255, 0.05),
    0 8px 32px rgba(0, 0, 0, 0.4),
    0 2px 8px rgba(0, 0, 0, 0.2);
}

.titlebar {
  z-index: 9999;
}

/* 窗口缩放边框 */
.resize-handles {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9998;
}

.resize-edge,
.resize-corner {
  position: absolute;
  pointer-events: auto;
}

.resize-edge.top {
  top: 0;
  left: 0;
  right: 0;
  height: 6px;
  cursor: n-resize;
}

.resize-edge.bottom {
  bottom: 0;
  left: 0;
  right: 0;
  height: 6px;
  cursor: s-resize;
}

.resize-edge.left {
  top: 0;
  bottom: 0;
  left: 0;
  width: 6px;
  cursor: w-resize;
}

.resize-edge.right {
  top: 0;
  bottom: 0;
  right: 0;
  width: 6px;
  cursor: e-resize;
}

.resize-corner.top-left {
  top: 0;
  left: 0;
  width: 12px;
  height: 12px;
  cursor: nw-resize;
}

.resize-corner.top-right {
  top: 0;
  right: 0;
  width: 12px;
  height: 12px;
  cursor: ne-resize;
}

.resize-corner.bottom-left {
  bottom: 0;
  left: 0;
  width: 12px;
  height: 12px;
  cursor: sw-resize;
}

.resize-corner.bottom-right {
  bottom: 0;
  right: 0;
  width: 12px;
  height: 12px;
  cursor: se-resize;
}
</style>
