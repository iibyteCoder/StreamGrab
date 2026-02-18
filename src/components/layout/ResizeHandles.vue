<script setup lang="ts">
/**
 * ResizeHandles 窗口缩放边框组件
 * 为无装饰窗口提供边角和边缘缩放功能
 */

import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

// 缩放方向映射
type ResizeDirection = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

// 启动缩放
const startResize = (direction: ResizeDirection) => async (e: MouseEvent) => {
  e.preventDefault();
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    await (appWindow as any).startResizing(direction);
  } catch {
    // 非 Tauri 环境忽略
  }
};

// 边框尺寸
const EDGE_SIZE = 6;
const CORNER_SIZE = 12;
</script>

<template>
  <div class="resize-handles">
    <!-- 顶部边框 -->
    <div
      class="resize-edge top"
      :style="{ height: `${EDGE_SIZE}px` }"
      @mousedown="startResize('n')"
    />

    <!-- 底部边框 -->
    <div
      class="resize-edge bottom"
      :style="{ height: `${EDGE_SIZE}px` }"
      @mousedown="startResize('s')"
    />

    <!-- 左侧边框 -->
    <div
      class="resize-edge left"
      :style="{ width: `${EDGE_SIZE}px` }"
      @mousedown="startResize('w')"
    />

    <!-- 右侧边框 -->
    <div
      class="resize-edge right"
      :style="{ width: `${EDGE_SIZE}px` }"
      @mousedown="startResize('e')"
    />

    <!-- 四个角 -->
    <div
      class="resize-corner top-left"
      :style="{ width: `${CORNER_SIZE}px`, height: `${CORNER_SIZE}px` }"
      @mousedown="startResize('nw')"
    />
    <div
      class="resize-corner top-right"
      :style="{ width: `${CORNER_SIZE}px`, height: `${CORNER_SIZE}px` }"
      @mousedown="startResize('ne')"
    />
    <div
      class="resize-corner bottom-left"
      :style="{ width: `${CORNER_SIZE}px`, height: `${CORNER_SIZE}px` }"
      @mousedown="startResize('sw')"
    />
    <div
      class="resize-corner bottom-right"
      :style="{ width: `${CORNER_SIZE}px`, height: `${CORNER_SIZE}px` }"
      @mousedown="startResize('se')"
    />
  </div>
</template>

<style scoped>
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
  cursor: n-resize;
}

.resize-edge.bottom {
  bottom: 0;
  left: 0;
  right: 0;
  cursor: s-resize;
}

.resize-edge.left {
  top: 0;
  bottom: 0;
  left: 0;
  cursor: w-resize;
}

.resize-edge.right {
  top: 0;
  bottom: 0;
  right: 0;
  cursor: e-resize;
}

.resize-corner.top-left {
  top: 0;
  left: 0;
  cursor: nw-resize;
}

.resize-corner.top-right {
  top: 0;
  right: 0;
  cursor: ne-resize;
}

.resize-corner.bottom-left {
  bottom: 0;
  left: 0;
  cursor: sw-resize;
}

.resize-corner.bottom-right {
  bottom: 0;
  right: 0;
  cursor: se-resize;
}
</style>
