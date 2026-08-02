/**
 * 任务右键菜单项定义与显隐规则
 *
 * 显隐矩阵见 spec §4：四个常驻项 + 「复制文件路径」条件项
 * （completed 且 outputPath 非空）。纯函数，无 DOM 依赖，可独立单测。
 */

import * as Icons from "lucide-vue-next";
import type { DownloadTask } from "@/domain";

type IconName = keyof typeof Icons;

export type ContextMenuItemKey =
  | "redownload"
  | "copyUrl"
  | "copyFileName"
  | "copyFilePath"
  | "openDetail";

export interface ContextMenuItemDef {
  key: ContextMenuItemKey;
  icon: IconName;
  /** i18n key（task.contextMenu.* 命名空间） */
  labelKey: string;
  /** i18n 缺失时的兜底文案（zh-CN） */
  fallback: string;
  /** 本项之后渲染分隔线 */
  separatorAfter?: boolean;
}

export function buildContextMenuItems(
  task: DownloadTask,
): ContextMenuItemDef[] {
  const items: ContextMenuItemDef[] = [
    {
      key: "redownload",
      icon: "RotateCw",
      labelKey: "task.contextMenu.redownload",
      fallback: "以此链接重新下载",
      separatorAfter: true,
    },
    {
      key: "copyUrl",
      icon: "Link2",
      labelKey: "task.contextMenu.copyUrl",
      fallback: "复制下载链接",
    },
    {
      key: "copyFileName",
      icon: "FileText",
      labelKey: "task.contextMenu.copyFileName",
      fallback: "复制文件名",
    },
  ];

  if (task.status === "completed" && task.outputPath) {
    items.push({
      key: "copyFilePath",
      icon: "Folder",
      labelKey: "task.contextMenu.copyFilePath",
      fallback: "复制文件路径",
    });
  }

  // 最后一个复制项之后补分隔线（items 此处恒有 ≥3 项，guard 仅为满足
  // noUncheckedIndexedAccess）
  const lastCopyItem = items[items.length - 1];
  if (lastCopyItem) {
    lastCopyItem.separatorAfter = true;
  }

  items.push({
    key: "openDetail",
    icon: "PanelRightOpen",
    labelKey: "task.contextMenu.openDetail",
    fallback: "打开详情",
  });

  return items;
}
