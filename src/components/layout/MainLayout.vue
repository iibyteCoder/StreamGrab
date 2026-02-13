<script setup lang="ts">
/**
 * MainLayout 主布局组件
 * 应用的主要布局结构
 */

import { computed } from 'vue';
import { useUiStore } from '@/stores';
import TitleBar from './TitleBar.vue';

interface Props {
  showTitleBar?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showTitleBar: true,
});

const uiStore = useUiStore();

const sidebarClasses = computed(() => {
  return [
    'flex-shrink-0 h-full bg-bg-surface border-r border-border-default transition-all duration-300',
    uiStore.isSidebarCollapsed ? 'w-0 overflow-hidden' : 'w-48',
  ];
});

const toggleSidebar = () => {
  uiStore.toggleSidebar();
};
</script>

<template>
  <div class="h-screen flex flex-col bg-bg-base text-text-primary overflow-hidden">
    <!-- 标题栏 -->
    <TitleBar v-if="showTitleBar" />

    <!-- 主体区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 侧边栏（可选） -->
      <aside v-if="$slots.sidebar" :class="sidebarClasses">
        <div class="w-48 h-full flex flex-col py-4">
          <!-- 侧边栏内容 -->
          <slot name="sidebar" />
        </div>
      </aside>

      <!-- 主内容区 -->
      <main class="flex-1 flex flex-col overflow-hidden">
        <!-- 顶部工具栏 -->
        <header
          v-if="$slots.toolbar"
          class="flex-shrink-0 border-b border-border-default bg-bg-surface"
        >
          <div class="px-6 py-3 flex items-center justify-between">
            <slot name="toolbar" />
          </div>
        </header>

        <!-- 内容区域 -->
        <div class="flex-1 overflow-auto">
          <div class="p-6">
            <slot />
          </div>
        </div>

        <!-- 底部状态栏（可选） -->
        <footer
          v-if="$slots.footer"
          class="flex-shrink-0 border-t border-border-default bg-bg-surface"
        >
          <div class="px-6 py-2">
            <slot name="footer" />
          </div>
        </footer>
      </main>
    </div>

    <!-- 全局 Toast 容器 -->
    <div class="fixed bottom-6 right-6 z-50 flex flex-col gap-2">
      <slot name="toasts" />
    </div>
  </div>
</template>
