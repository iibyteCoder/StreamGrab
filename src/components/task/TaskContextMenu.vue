<script setup lang="ts">
/**
 * TaskContextMenu - 任务卡片右键菜单
 * 纯展示组件：收纳悬停按钮放不下的次要操作
 * （重新下载 / 复制链接 / 复制文件名 / 复制文件路径 / 打开详情）。
 * 显隐规则委托 buildContextMenuItems；本组件只做渲染 + 事件转发。
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";
import { AppIcon } from "@/components/common";
import {
  buildContextMenuItems,
  type ContextMenuItemKey,
} from "./contextMenuItems";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
  /** 预留：当前显隐矩阵不依赖此项（见 spec §4 注 ①） */
  fileExists?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  fileExists: false,
});

const emit = defineEmits<{
  (e: "redownload"): void;
  (e: "copyUrl"): void;
  (e: "copyFileName"): void;
  (e: "copyFilePath"): void;
  (e: "openDetail"): void;
}>();

const { t } = useI18n();

const items = computed(() => buildContextMenuItems(props.task));

const handlers: Record<ContextMenuItemKey, () => void> = {
  redownload: () => emit("redownload"),
  copyUrl: () => emit("copyUrl"),
  copyFileName: () => emit("copyFileName"),
  copyFilePath: () => emit("copyFilePath"),
  openDetail: () => emit("openDetail"),
};
</script>

<template>
  <ContextMenuContent class="w-56">
    <template v-for="item in items" :key="item.key">
      <ContextMenuItem @select="handlers[item.key]()">
        <AppIcon :name="item.icon" :size="14" class="mr-2" />
        {{ t(item.labelKey, item.fallback) }}
      </ContextMenuItem>
      <ContextMenuSeparator v-if="item.separatorAfter" />
    </template>
  </ContextMenuContent>
</template>
