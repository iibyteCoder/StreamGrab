# 任务卡片右键上下文菜单 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为任务卡片添加右键上下文菜单，收纳 5 项次要操作（以此链接重新下载 / 复制下载链接 / 复制文件名 / 复制文件路径 / 打开详情），不重复悬停按钮已有的状态操作；纯前端、零后端改动。

**Architecture:** shadcn-vue ContextMenu（reka-ui，与现有 `ui/dropdown-menu` 同构）包裹 `TaskCard` 根 div；新增纯展示组件 `TaskContextMenu`（显隐矩阵委托给纯函数 `buildContextMenuItems`，可独立单测）；三项复制在 TaskCard 内闭环（`clipboardService.writeText` + toast）；「重新下载」沿 TaskCard → TaskList → HomeView 事件链冒泡，预填 `AddTaskDialog`（新增 `initialUrl` prop）并复用 `submitPaste` 自动推进到配置步。

**Tech Stack:** Vue 3 `<script setup>` + TypeScript、vue-i18n、shadcn-vue ContextMenu（reka-ui）、lucide 图标（经 `AppIcon`）、`@tauri-apps/plugin-clipboard-manager`、vitest + @vue/test-utils + happy-dom。

**Spec:** `docs/superpowers/specs/2026-08-03-task-context-menu-design.md`（已批准）
**Branch:** `feat/task-context-menu`（已创建；spec 已提交 `2997504`）

## Global Constraints

- 新增用户可见文案必须三语（zh-CN / zh-TW / en-US）同步（CLAUDE.md）
- 组件不直接调用 Tauri API，统一经 `src/services/`（CLAUDE.md 架构规则）
- 禁止 `any` 类型（CLAUDE.md）
- 提交信息格式 `feat:` / `fix:` / `docs:` / `chore:` / `test:`，每条末尾附 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer
- 验证基准：`npm run type-check`、`npm run lint`、`npm test` 三项全绿
- **并行计划提示**：`docs/superpowers/plans/2026-08-03-detail-panel-copy-url.md` 可能先行落地 `clipboardService.writeText` 与剪贴板权限——Task 2 为防御式实现，已存在则跳过
- 不改 `TaskQuickActions` / `TaskDetailPanel` 既有逻辑；不做旧硬编码文案的 i18n 清理

---

## Task 1: 三语 i18n 文案（5 菜单项 + 3 toast）

**Files:**
- Modify: `src/locales/zh-CN.ts`（`task` 块 actions 之后；`messages` 块 clipboardUrlsDetected 之后）
- Modify: `src/locales/zh-TW.ts`（同上）
- Modify: `src/locales/en-US.ts`（同上）

**Interfaces:**
- Produces: i18n keys `task.contextMenu.{redownload,copyUrl,copyFileName,copyFilePath,openDetail}`（Task 5 菜单渲染依赖）、`messages.{copiedUrl,copiedFileName,copiedFilePath}`（Task 6 toast 依赖）

- [ ] **Step 1: zh-CN —— 菜单项 keys**

`src/locales/zh-CN.ts` 中找到：

```typescript
      viewLog: "查看日志",
    },
    unnamed: "未命名文件",
```

改为：

```typescript
      viewLog: "查看日志",
    },
    contextMenu: {
      redownload: "以此链接重新下载",
      copyUrl: "复制下载链接",
      copyFileName: "复制文件名",
      copyFilePath: "复制文件路径",
      openDetail: "打开详情",
    },
    unnamed: "未命名文件",
```

- [ ] **Step 2: zh-CN —— toast keys**

`src/locales/zh-CN.ts` 中找到：

```typescript
    clipboardUrlsDetected: "已添加 {count} 个下载链接",
```

在其后插入三行：

```typescript
    copiedUrl: "已复制下载链接",
    copiedFileName: "已复制文件名",
    copiedFilePath: "已复制文件路径",
```

- [ ] **Step 3: zh-TW —— 菜单项 keys**

`src/locales/zh-TW.ts` 中找到：

```typescript
      viewLog: "查看日誌",
    },
    unnamed: "未命名檔案",
```

改为：

```typescript
      viewLog: "查看日誌",
    },
    contextMenu: {
      redownload: "以此連結重新下載",
      copyUrl: "複製下載連結",
      copyFileName: "複製檔案名稱",
      copyFilePath: "複製檔案路徑",
      openDetail: "開啟詳情",
    },
    unnamed: "未命名檔案",
```

- [ ] **Step 4: zh-TW —— toast keys**

`src/locales/zh-TW.ts` 中找到：

```typescript
    clipboardUrlsDetected: "已新增 {count} 個下載連結",
```

在其后插入三行：

```typescript
    copiedUrl: "已複製下載連結",
    copiedFileName: "已複製檔案名稱",
    copiedFilePath: "已複製檔案路徑",
```

- [ ] **Step 5: en-US —— 菜单项 keys**

`src/locales/en-US.ts` 中找到：

```typescript
      viewLog: "View Log",
    },
    unnamed: "Unnamed File",
```

改为：

```typescript
      viewLog: "View Log",
    },
    contextMenu: {
      redownload: "Redownload from this link",
      copyUrl: "Copy download link",
      copyFileName: "Copy file name",
      copyFilePath: "Copy file path",
      openDetail: "Open details",
    },
    unnamed: "Unnamed File",
```

- [ ] **Step 6: en-US —— toast keys**

`src/locales/en-US.ts` 中找到：

```typescript
    clipboardUrlsDetected: "{count} download links added",
```

在其后插入三行：

```typescript
    copiedUrl: "Download link copied",
    copiedFileName: "File name copied",
    copiedFilePath: "File path copied",
```

- [ ] **Step 7: 验证**

Run: `npm run type-check && npm run lint`
Expected: 无错误、无新告警。

- [ ] **Step 8: 提交**

```bash
git add src/locales/zh-CN.ts src/locales/zh-TW.ts src/locales/en-US.ts
git commit -m "feat: 任务右键菜单与复制 toast 三语文案

task.contextMenu.*（5 个菜单项）+ messages.copied*（3 个复制成功 toast）

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 2: 剪贴板写入能力（防御式，可能已被并行计划落地）

**Files:**
- Modify（可能跳过）: `src/services/clipboardService.ts`（新增 `writeText`）
- Modify（可能跳过）: `src-tauri/capabilities/default.json`（新增写权限）

**Interfaces:**
- Produces: `clipboardService.writeText(text: string): Promise<void>` —— Task 6 的复制 handler 依赖此签名

- [ ] **Step 1: 检查 writeText 是否已存在**

Run: `grep -n "writeText" src/services/clipboardService.ts`

若输出含 `writeText` 方法定义 → 说明并行计划已落地，**跳过 Step 2**，直接进入 Step 3。否则继续。

- [ ] **Step 2: 为 clipboardService 增加 writeText**

`src/services/clipboardService.ts` 完整目标内容（改 import 行 + 加一个方法 + 文档注释，其余不动）：

```typescript
/**
 * 剪贴板服务
 *
 * 封装 @tauri-apps/plugin-clipboard-manager 的读取/写入 + 焦点事件订阅。
 * composable 层不直接 import @tauri-apps 原始 API。
 */

import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { subscribeToEvent, type UnlistenFn } from "./tauri";

class ClipboardService {
  /** 读取剪贴板文本 */
  async readText(): Promise<string> {
    return readText();
  }

  /** 写入文本到剪贴板 */
  async writeText(text: string): Promise<void> {
    return writeText(text);
  }

  /** 订阅窗口焦点事件 */
  async onFocus(handler: () => void): Promise<UnlistenFn> {
    return subscribeToEvent<null>("tauri://focus", () => handler());
  }
}

export const clipboardService = new ClipboardService();
```

- [ ] **Step 3: 检查并补齐写权限**

Run: `grep -n "clipboard-manager:allow-write-text" src-tauri/capabilities/default.json`

若无输出，将 `src-tauri/capabilities/default.json` 中：

```json
    "dialog:allow-open",
    "dialog:allow-save"
  ]
```

改为：

```json
    "dialog:allow-open",
    "dialog:allow-save",
    "clipboard-manager:allow-write-text"
  ]
```

（`clipboard-manager:allow-read-text` 若缺失不在本计划补——归并行计划 detail-panel-copy-url 负责；本功能只需写权限。）

- [ ] **Step 4: 确认后端已注册 clipboard 插件**

Run: `grep -n "clipboard" src-tauri/src/lib.rs`
Expected: 含 `tauri_plugin_clipboard_manager` 的 `.plugin(...)` 注册行（已注册则无需改动；若无输出则停下向用户确认，不要自行添加后端代码）。

- [ ] **Step 5: 验证**

Run: `npm run type-check && npm test`
Expected: type-check 无错误；现有测试套件全绿。

- [ ] **Step 6: 提交（若本任务有文件改动）**

```bash
git add src/services/clipboardService.ts src-tauri/capabilities/default.json
git commit -m "feat: clipboardService 新增 writeText 并补齐写权限

供任务右键菜单复制操作使用；若并行计划已落地则本提交为空跳过

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

若 Step 1 与 Step 3 均跳过（无任何改动），则不提交，直接进入 Task 3。

---

## Task 3: 菜单项显隐纯函数（TDD）

**Files:**
- Create: `src/components/task/contextMenuItems.ts`
- Create: `src/components/task/contextMenuItems.test.ts`

**Interfaces:**
- Produces: `buildContextMenuItems(task: DownloadTask): ContextMenuItemDef[]` 与类型 `ContextMenuItemKey`、`ContextMenuItemDef` —— Task 5 的 `TaskContextMenu` 消费

**说明：** 显隐矩阵（spec §4）抽为纯函数，node 环境即可单测，无需 DOM——与现有 `resolveLinkToTask.test.ts` 的纯函数测试风格一致。

- [ ] **Step 1: 先写失败测试**

创建 `src/components/task/contextMenuItems.test.ts`：

```typescript
import { describe, it, expect } from "vitest";
import { buildContextMenuItems } from "./contextMenuItems";
import type { DownloadTask } from "@/domain";

function mkTask(overrides: Partial<DownloadTask> = {}): DownloadTask {
  return {
    id: "t1",
    url: "https://example.com/v.m3u8",
    fileName: "v.mp4",
    saveDir: "/downloads",
    status: "pending",
    wasInterrupted: false,
    createdAt: "2026-08-03T00:00:00Z",
    updatedAt: "2026-08-03T00:00:00Z",
    progress: {
      percent: 0,
      overallPercent: 0,
      speed: 0,
      downloadedSize: 0,
      totalSize: 0,
      downloadedSegments: 0,
      totalSegments: 0,
      eta: 0,
      currentAction: "",
    },
    ...overrides,
  };
}

const ALL_STATUSES: DownloadTask["status"][] = [
  "pending",
  "analyzing",
  "downloading",
  "merging",
  "muxing",
  "paused",
  "completed",
  "failed",
  "cancelled",
];

describe("buildContextMenuItems", () => {
  it("四个常驻项在任何状态下均存在", () => {
    for (const status of ALL_STATUSES) {
      const keys = buildContextMenuItems(mkTask({ status })).map((i) => i.key);
      expect(keys).toContain("redownload");
      expect(keys).toContain("copyUrl");
      expect(keys).toContain("copyFileName");
      expect(keys).toContain("openDetail");
    }
  });

  it("复制文件路径仅在 completed + outputPath 时出现", () => {
    for (const status of ALL_STATUSES) {
      const withPath = buildContextMenuItems(
        mkTask({ status, outputPath: "/downloads/v.mp4" }),
      ).map((i) => i.key);
      const noPath = buildContextMenuItems(mkTask({ status })).map(
        (i) => i.key,
      );
      if (status === "completed") {
        expect(withPath).toContain("copyFilePath");
        expect(noPath).not.toContain("copyFilePath");
      } else {
        expect(withPath).not.toContain("copyFilePath");
      }
    }
  });

  it("顺序与分隔线：重新下载居首且其后有分隔线，最后一个复制项后有分隔线，打开详情居末", () => {
    const items = buildContextMenuItems(
      mkTask({ status: "completed", outputPath: "/downloads/v.mp4" }),
    );
    expect(items.map((i) => i.key)).toEqual([
      "redownload",
      "copyUrl",
      "copyFileName",
      "copyFilePath",
      "openDetail",
    ]);
    expect(items[0].separatorAfter).toBe(true);
    expect(items[3].separatorAfter).toBe(true);
    expect(items[4].separatorAfter).toBeFalsy();
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/contextMenuItems.test.ts`
Expected: FAIL —— `Cannot find module './contextMenuItems'`

- [ ] **Step 3: 实现纯函数**

创建 `src/components/task/contextMenuItems.ts`：

```typescript
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

  // 最后一个复制项之后补分隔线
  items[items.length - 1].separatorAfter = true;

  items.push({
    key: "openDetail",
    icon: "PanelRightOpen",
    labelKey: "task.contextMenu.openDetail",
    fallback: "打开详情",
  });

  return items;
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npx vitest run src/components/task/contextMenuItems.test.ts`
Expected: PASS（3 个用例全绿）

- [ ] **Step 5: 提交**

```bash
git add src/components/task/contextMenuItems.ts src/components/task/contextMenuItems.test.ts
git commit -m "feat: 右键菜单项显隐纯函数 buildContextMenuItems（含单测）

四常驻项 + 复制文件路径条件项（completed + outputPath）；
顺序与分隔线规则锁定于测试。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 4: ContextMenu 基础脚手架 + 组件测试基建

**Files:**
- Create: `src/components/ui/context-menu/`（CLI 生成，~15 个文件）
- Modify: `vitest.config.ts`（加 vue 插件；组件测试按文件声明 happy-dom）
- Modify: `package.json` + `package-lock.json`（devDeps: @vue/test-utils、happy-dom）

**Interfaces:**
- Produces: `@/components/ui/context-menu` 导出（Task 5/6 消费 `ContextMenu`、`ContextMenuTrigger`、`ContextMenuContent`、`ContextMenuItem`、`ContextMenuSeparator`）；组件测试能力（Task 5/6/7 的 `.test.ts` 依赖）

- [ ] **Step 1: 安装组件测试依赖**

Run: `npm install -D @vue/test-utils happy-dom`
Expected: 安装成功；`package.json` devDependencies 新增 `@vue/test-utils`、`happy-dom`。

- [ ] **Step 2: 更新 vitest 配置**

`vitest.config.ts` 完整目标内容：

```typescript
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
```

- [ ] **Step 3: 生成 shadcn-vue context-menu 组件**

Run: `npx shadcn-vue@latest add context-menu --yes`
Expected: 输出新增文件列表；`src/components/ui/context-menu/` 下出现 `index.ts`、`ContextMenu.vue`、`ContextMenuTrigger.vue`、`ContextMenuContent.vue`、`ContextMenuItem.vue`、`ContextMenuSeparator.vue` 等；文件内 import 来自 `reka-ui`（与现有 `ui/dropdown-menu` 同构）。

若 CLI 报错或生成内容 import 自 `radix-vue`（而非 `reka-ui`），停下向用户确认，不要手工改写脚手架。

- [ ] **Step 4: 验证**

Run: `npm run type-check && npm test`
Expected: type-check 无错误；既有测试（含 Task 3 新增）全绿——脚手架与配置改动未破坏现有套件。

- [ ] **Step 5: 提交**

```bash
git add src/components/ui/context-menu vitest.config.ts package.json package-lock.json
git commit -m "chore: 引入 shadcn-vue context-menu 脚手架与组件测试基建

- npx shadcn-vue add context-menu（reka-ui，与 dropdown-menu 同构）
- devDeps: @vue/test-utils + happy-dom
- vitest 加 vue 插件；组件测试按文件声明 happy-dom，纯函数测试保持 node

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 5: TaskContextMenu 纯展示组件（TDD）

**Files:**
- Create: `src/components/task/TaskContextMenu.vue`
- Create: `src/components/task/TaskContextMenu.test.ts`
- Modify: `src/components/task/index.ts`（barrel 导出）

**Interfaces:**
- Consumes: Task 3 的 `buildContextMenuItems` / `ContextMenuItemKey`；Task 1 的 `task.contextMenu.*`；Task 4 的 `@/components/ui/context-menu`
- Produces: `TaskContextMenu` 组件 —— props `{ task: DownloadTask; fileExists?: boolean }`（fileExists 预留，显隐不依赖），emits `redownload` / `copyUrl` / `copyFileName` / `copyFilePath` / `openDetail`；Task 6 在 TaskCard 内消费

- [ ] **Step 1: 先写失败测试**

创建 `src/components/task/TaskContextMenu.test.ts`：

```typescript
/** @vitest-environment happy-dom */
import { describe, it, expect, afterEach } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick, defineComponent } from "vue";
import type { PropType } from "vue";
import {
  ContextMenu,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import TaskContextMenu from "./TaskContextMenu.vue";
import { i18n } from "@/locales";
import type { DownloadTask } from "@/domain";

function mkTask(overrides: Partial<DownloadTask> = {}): DownloadTask {
  return {
    id: "t1",
    url: "https://example.com/v.m3u8",
    fileName: "v.mp4",
    saveDir: "/downloads",
    status: "pending",
    wasInterrupted: false,
    createdAt: "2026-08-03T00:00:00Z",
    updatedAt: "2026-08-03T00:00:00Z",
    progress: {
      percent: 0,
      overallPercent: 0,
      speed: 0,
      downloadedSize: 0,
      totalSize: 0,
      downloadedSegments: 0,
      totalSegments: 0,
      eta: 0,
      currentAction: "",
    },
    ...overrides,
  };
}

const Host = defineComponent({
  components: { ContextMenu, ContextMenuTrigger, TaskContextMenu },
  props: {
    task: { type: Object as PropType<DownloadTask>, required: true },
  },
  emits: ["redownload", "copyUrl", "copyFileName", "copyFilePath", "openDetail"],
  template: `
    <ContextMenu>
      <ContextMenuTrigger as-child>
        <div class="trigger">target</div>
      </ContextMenuTrigger>
      <TaskContextMenu
        :task="task"
        @redownload="$emit('redownload')"
        @copy-url="$emit('copyUrl')"
        @copy-file-name="$emit('copyFileName')"
        @copy-file-path="$emit('copyFilePath')"
        @open-detail="$emit('openDetail')"
      />
    </ContextMenu>
  `,
});

let wrapper: ReturnType<typeof mount> | null = null;

afterEach(() => {
  wrapper?.unmount();
  wrapper = null;
  document.body.innerHTML = "";
});

async function openMenu(task: DownloadTask): Promise<HTMLElement[]> {
  wrapper = mount(Host, {
    props: { task },
    global: { plugins: [i18n] },
    attachTo: document.body,
  });
  await wrapper.find(".trigger").trigger("contextmenu");
  await nextTick();
  await nextTick();
  return [...document.querySelectorAll('[role="menuitem"]')] as HTMLElement[];
}

describe("TaskContextMenu 渲染", () => {
  it("非 completed 状态 → 仅 4 个常驻项（测试环境默认 zh-CN）", async () => {
    const items = await openMenu(mkTask({ status: "pending" }));
    expect(items.map((i) => i.textContent?.trim())).toEqual([
      "以此链接重新下载",
      "复制下载链接",
      "复制文件名",
      "打开详情",
    ]);
  });

  it("completed + outputPath → 含复制文件路径（5 项）", async () => {
    const items = await openMenu(
      mkTask({ status: "completed", outputPath: "/downloads/v.mp4" }),
    );
    expect(items.map((i) => i.textContent?.trim())).toEqual([
      "以此链接重新下载",
      "复制下载链接",
      "复制文件名",
      "复制文件路径",
      "打开详情",
    ]);
  });

  it("completed 但无 outputPath → 无复制文件路径（4 项）", async () => {
    const items = await openMenu(mkTask({ status: "completed" }));
    expect(items).toHaveLength(4);
    expect(items.map((i) => i.textContent?.trim())).not.toContain(
      "复制文件路径",
    );
  });

  it("点击菜单项 → emit 对应事件", async () => {
    await openMenu(mkTask({ status: "pending" }));
    const items = [
      ...document.querySelectorAll('[role="menuitem"]'),
    ] as HTMLElement[];
    items[1].click(); // 复制下载链接
    await nextTick();
    expect(wrapper!.emitted("copyUrl")).toHaveLength(1);
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/TaskContextMenu.test.ts`
Expected: FAIL —— `Cannot find module './TaskContextMenu.vue'`

- [ ] **Step 3: 实现组件**

创建 `src/components/task/TaskContextMenu.vue`：

```vue
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
```

- [ ] **Step 4: barrel 导出**

`src/components/task/index.ts` 中找到：

```typescript
// TaskCard 子组件
export { default as TaskStatusBadge } from "./TaskStatusBadge.vue";
```

在其前插入一行：

```typescript
export { default as TaskContextMenu } from "./TaskContextMenu.vue";
```

- [ ] **Step 5: 运行测试确认通过**

Run: `npx vitest run src/components/task/TaskContextMenu.test.ts`
Expected: PASS（4 个用例全绿）

- [ ] **Step 6: 提交**

```bash
git add src/components/task/TaskContextMenu.vue src/components/task/TaskContextMenu.test.ts src/components/task/index.ts
git commit -m "feat: TaskContextMenu 右键菜单纯展示组件（含渲染测试）

v-for 渲染 buildContextMenuItems 结果；@select 转发 5 个动作事件；
happy-dom 下真实打开菜单断言条目构成与 emit。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 6: TaskCard 接入菜单 + 复制/重下载处理（TDD）

**Files:**
- Modify: `src/components/task/TaskCard.vue`（script imports、emits、handlers、template 包裹）
- Create: `src/components/task/TaskCard.test.ts`

**Interfaces:**
- Consumes: Task 2 的 `clipboardService.writeText`；Task 1 的 `messages.copied*`；Task 5 的 `TaskContextMenu`；Task 4 的 `ContextMenu` / `ContextMenuTrigger`
- Produces: `TaskCard` 新增 emit `redownload`（携带 task）—— Task 7 的 TaskList 透传依赖

**说明：** 用 `ContextMenu` + `ContextMenuTrigger as-child` 包裹根 div 后，`npm run lint`（含 `--fix`）会由 prettier 自动重排整个 template 缩进，产生大量纯缩进 diff，属预期。

- [ ] **Step 1: 先写失败测试**

创建 `src/components/task/TaskCard.test.ts`：

```typescript
/** @vitest-environment happy-dom */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import TaskCard from "./TaskCard.vue";
import TaskContextMenu from "./TaskContextMenu.vue";
import { i18n } from "@/locales";
import type { DownloadTask } from "@/domain";

const mocks = vi.hoisted(() => ({
  writeText: vi.fn(),
  toastSuccess: vi.fn(),
}));

vi.mock("@/services", () => ({
  systemService: {
    fileExists: vi.fn().mockResolvedValue(true),
    openInExplorer: vi.fn().mockResolvedValue(undefined),
    openFileInExplorer: vi.fn().mockResolvedValue(undefined),
    deleteFileOrFolder: vi.fn().mockResolvedValue(undefined),
  },
  clipboardService: {
    readText: vi.fn().mockResolvedValue(""),
    writeText: mocks.writeText,
    onFocus: vi.fn(),
  },
}));

vi.mock("@/composables", () => ({
  useTasks: () => ({ removeTask: vi.fn() }),
  useDownloader: () => ({
    startDownload: vi.fn(),
    stopDownload: vi.fn(),
    pauseDownload: vi.fn(),
    resumeDownload: vi.fn(),
  }),
  useToast: () => ({
    success: mocks.toastSuccess,
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
    remove: vi.fn(),
    clear: vi.fn(),
    toasts: [],
  }),
}));

vi.mock("@/stores", () => ({
  useTaskStore: () => ({
    getTaskLogs: () => [],
    retryTask: vi.fn(),
    getTaskById: () => undefined,
  }),
}));

function mkTask(overrides: Partial<DownloadTask> = {}): DownloadTask {
  return {
    id: "t1",
    url: "https://example.com/v.m3u8",
    fileName: "v.mp4",
    saveDir: "/downloads",
    status: "completed",
    wasInterrupted: false,
    createdAt: "2026-08-03T00:00:00Z",
    updatedAt: "2026-08-03T00:00:00Z",
    progress: {
      percent: 100,
      overallPercent: 100,
      speed: 0,
      downloadedSize: 0,
      totalSize: 0,
      downloadedSegments: 0,
      totalSegments: 0,
      eta: 0,
      currentAction: "",
    },
    ...overrides,
  };
}

function mountCard(task: DownloadTask) {
  return mount(TaskCard, {
    props: { task },
    global: { plugins: [i18n] },
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.writeText.mockResolvedValue(undefined);
});

describe("TaskCard 右键菜单接线", () => {
  it("copyUrl → 写入 task.url + 成功 toast", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyUrl");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("https://example.com/v.m3u8");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFileName → 写入 task.fileName + 成功 toast", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFileName");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("v.mp4");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFilePath → 有 outputPath 时写入路径 + 成功 toast", async () => {
    const wrapper = mountCard(
      mkTask({ outputPath: "/downloads/v.mp4" }),
    );
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFilePath");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("/downloads/v.mp4");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFilePath → 无 outputPath 时静默跳过", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFilePath");
    await flushPromises();
    expect(mocks.writeText).not.toHaveBeenCalled();
    expect(mocks.toastSuccess).not.toHaveBeenCalled();
  });

  it("redownload → emit('redownload', task)", async () => {
    const task = mkTask();
    const wrapper = mountCard(task);
    wrapper.findComponent(TaskContextMenu).vm.$emit("redownload");
    await nextTick();
    expect(wrapper.emitted("redownload")).toEqual([[task]]);
  });

  it("左键点击卡片 → 仍 emit('click', task)（回归）", async () => {
    const task = mkTask();
    const wrapper = mountCard(task);
    await wrapper.find(".task-card").trigger("click");
    expect(wrapper.emitted("click")?.[0]).toEqual([task]);
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/TaskCard.test.ts`
Expected: FAIL —— `findComponent(TaskContextMenu)` 找不到组件（TaskCard 尚未接入）

- [ ] **Step 3: 更新 script —— imports 与 emits**

`src/components/task/TaskCard.vue` 中：

```typescript
import { computed, ref, onMounted, watch } from "vue";
import { AppIcon } from "@/components/common";
import { useTasks, useDownloader } from "@/composables";
import { useTaskStore } from "@/stores";
import { systemService } from "@/services";
```

改为：

```typescript
import { computed, ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { AppIcon } from "@/components/common";
import { useTasks, useDownloader, useToast } from "@/composables";
import { useTaskStore } from "@/stores";
import { systemService, clipboardService } from "@/services";
import {
  ContextMenu,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
```

```typescript
import {
  TaskStatusBadge,
  TaskQuickActions,
  TaskDeleteDialog,
  LogViewer,
} from "@/components/task";
```

改为：

```typescript
import {
  TaskStatusBadge,
  TaskQuickActions,
  TaskDeleteDialog,
  TaskContextMenu,
  LogViewer,
} from "@/components/task";
```

```typescript
const emit = defineEmits<{
  (e: "click", task: DownloadTask): void;
}>();
```

改为：

```typescript
const emit = defineEmits<{
  (e: "click", task: DownloadTask): void;
  (e: "redownload", task: DownloadTask): void;
}>();
```

- [ ] **Step 4: 新增 i18n / toast 实例与菜单 handler**

在 `const taskStore = useTaskStore();` 之后插入：

```typescript
const { t } = useI18n();
const toast = useToast();
```

在 `const handleClick = () => emit("click", props.task);` 之后插入：

```typescript
// 右键菜单操作
const handleRedownload = () => emit("redownload", props.task);

const handleCopyUrl = async () => {
  try {
    await clipboardService.writeText(props.task.url);
    toast.success(t("messages.copiedUrl", "已复制下载链接"));
  } catch (e) {
    console.error("Failed to copy URL:", e);
  }
};

const handleCopyFileName = async () => {
  try {
    await clipboardService.writeText(props.task.fileName);
    toast.success(t("messages.copiedFileName", "已复制文件名"));
  } catch (e) {
    console.error("Failed to copy file name:", e);
  }
};

const handleCopyFilePath = async () => {
  const path = props.task.outputPath;
  if (!path) return;
  try {
    await clipboardService.writeText(path);
    toast.success(t("messages.copiedFilePath", "已复制文件路径"));
  } catch (e) {
    console.error("Failed to copy file path:", e);
  }
};
```

- [ ] **Step 5: template 包裹 ContextMenu**

将 template 开头：

```vue
<template>
  <div
    class="task-card group relative rounded-lg border bg-card p-3 transition-all duration-200"
```

改为：

```vue
<template>
  <ContextMenu>
    <ContextMenuTrigger as-child>
      <div
        class="task-card group relative rounded-lg border bg-card p-3 transition-all duration-200"
```

将 template 末尾（根 div 闭合处）：

```vue
    <TaskDeleteDialog
      v-model:open="showDeleteDialog"
      :task="task"
      :file-exists="fileExists ?? false"
      :is-deleting="isDeleting"
      @confirm="performDelete"
    />
  </div>
</template>
```

改为：

```vue
    <TaskDeleteDialog
      v-model:open="showDeleteDialog"
      :task="task"
      :file-exists="fileExists ?? false"
      :is-deleting="isDeleting"
      @confirm="performDelete"
    />
      </div>
    </ContextMenuTrigger>
    <TaskContextMenu
      :task="task"
      :file-exists="fileExists ?? false"
      @redownload="handleRedownload"
      @copy-url="handleCopyUrl"
      @copy-file-name="handleCopyFileName"
      @copy-file-path="handleCopyFilePath"
      @open-detail="handleClick"
    />
  </ContextMenu>
</template>
```

（内部缩进暂不手工对齐——Step 6 的 lint --fix 会自动重排。）

- [ ] **Step 6: lint 自动格式化 + 运行测试**

Run: `npm run lint && npx vitest run src/components/task/TaskCard.test.ts`
Expected: lint --fix 重排 template 缩进（大量纯缩进 diff，属预期）；6 个测试用例全绿。

- [ ] **Step 7: 全量验证**

Run: `npm run type-check && npm test`
Expected: type-check 无错误；全部测试套件全绿。

- [ ] **Step 8: 提交**

```bash
git add src/components/task/TaskCard.vue src/components/task/TaskCard.test.ts
git commit -m "feat: TaskCard 接入右键菜单与复制/重下载处理

ContextMenu 包裹卡片根节点；复制三件套经 clipboardService.writeText
+ toast 闭环；重新下载 emit('redownload') 供上层预填添加对话框。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 7: 重新下载事件链（TaskList → HomeView → AddTaskDialog 预填）

**Files:**
- Modify: `src/components/task/TaskList.vue`（emits + 事件透传）
- Modify: `src/views/HomeView.vue`（prefillUrl 状态 + handleRedownload + 关闭清空 + 绑定）
- Modify: `src/components/task/AddTaskDialog.vue`（initialUrl prop + watch 自动推进）
- Create: `src/components/task/AddTaskDialog.test.ts`

**Interfaces:**
- Consumes: Task 6 的 `TaskCard` emit `redownload`
- Produces: 无（终端接线任务）

- [ ] **Step 1: 先写 AddTaskDialog 失败测试**

创建 `src/components/task/AddTaskDialog.test.ts`：

```typescript
/** @vitest-environment happy-dom */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { shallowMount, flushPromises } from "@vue/test-utils";
import AddTaskDialog from "./AddTaskDialog.vue";

const mocks = vi.hoisted(() => ({
  submitPaste: vi.fn(),
  reset: vi.fn(),
}));

vi.mock("@/composables", async () => {
  const { ref } = await import("vue");
  return {
    useAddTaskWizard: () => ({
      step: ref("paste"),
      current: ref(null),
      index: ref(0),
      total: ref(0),
      isSingle: ref(true),
      isLast: ref(true),
      showAddAll: ref(false),
      isSubmitting: ref(false),
      parseDone: ref(0),
      parseTotal: ref(0),
      parsingId: ref(null),
      dirs: ref([]),
      defaultDir: ref(""),
      showDuplicate: ref(false),
      duplicateTask: ref(null),
      reset: mocks.reset,
      submitPaste: mocks.submitPaste,
      retryParse: vi.fn(),
      browseSaveDir: vi.fn(),
      addCurrent: vi.fn(),
      skip: vi.fn(),
      addAll: vi.fn(),
      confirmDuplicate: vi.fn(),
      cancelDuplicate: vi.fn(),
    }),
  };
});

describe("AddTaskDialog initialUrl 预填", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("带 initialUrl 打开 → 自动调用 submitPaste(initialUrl)", async () => {
    const wrapper = shallowMount(AddTaskDialog, {
      props: { open: false, initialUrl: "https://example.com/x.m3u8" },
    });
    await wrapper.setProps({ open: true });
    await flushPromises();
    expect(mocks.reset).toHaveBeenCalledOnce();
    expect(mocks.submitPaste).toHaveBeenCalledWith(
      "https://example.com/x.m3u8",
    );
  });

  it("不带 initialUrl 打开 → 不调用 submitPaste（回归原流程）", async () => {
    const wrapper = shallowMount(AddTaskDialog, {
      props: { open: false },
    });
    await wrapper.setProps({ open: true });
    await flushPromises();
    expect(mocks.reset).toHaveBeenCalledOnce();
    expect(mocks.submitPaste).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/AddTaskDialog.test.ts`
Expected: FAIL —— 第一个用例 `submitPaste` 未被调用（initialUrl 尚未实现）

- [ ] **Step 3: AddTaskDialog 新增 initialUrl prop 与 watch 分支**

`src/components/task/AddTaskDialog.vue` 中：

```typescript
interface Props {
  open: boolean;
}
```

改为：

```typescript
interface Props {
  open: boolean;
  /** 预填链接并自动推进到配置步（来自右键菜单「以此链接重新下载」） */
  initialUrl?: string | null;
}
```

```typescript
watch(isOpen, async (open) => {
  if (open) {
    reset();
    pasteText.value = "";
    await nextTick();
    textareaRef.value?.focus();
  }
});
```

改为：

```typescript
watch(isOpen, async (open) => {
  if (!open) return;
  reset();
  if (props.initialUrl) {
    // 右键菜单「以此链接重新下载」：预填并自动提交解析，
    // 复用既有 submitPaste 链路（resolveLinkToTask / 重复检测）直达配置步
    pasteText.value = props.initialUrl;
    void submitPaste(props.initialUrl);
  } else {
    pasteText.value = "";
  }
  await nextTick();
  textareaRef.value?.focus();
});
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npx vitest run src/components/task/AddTaskDialog.test.ts`
Expected: PASS（2 个用例全绿）

- [ ] **Step 5: TaskList 事件透传**

`src/components/task/TaskList.vue` 中：

```typescript
defineEmits<{
  (e: "taskClick", task: DownloadTask): void;
}>();
```

改为：

```typescript
defineEmits<{
  (e: "taskClick", task: DownloadTask): void;
  (e: "taskRedownload", task: DownloadTask): void;
}>();
```

template 中（v-for 内唯一的 `<TaskCard>` 实例）：

```vue
        @click="$emit('taskClick', $event)"
```

改为：

```vue
        @click="$emit('taskClick', $event)"
        @redownload="$emit('taskRedownload', $event)"
```

- [ ] **Step 6: HomeView 状态与绑定**

`src/views/HomeView.vue` 中：

```typescript
// 添加任务弹窗
const showAddDialog = ref(false);
```

改为：

```typescript
// 添加任务弹窗
const showAddDialog = ref(false);
// 右键菜单「重新下载」→ 预填添加对话框的 URL（一次性交接）
const prefillUrl = ref<string | null>(null);
```

在「关闭详情面板时清除选中状态」的 watch 之后插入：

```typescript
// 添加对话框关闭时清空预填 URL，防止污染下一次普通「添加任务」
watch(showAddDialog, (open) => {
  if (!open) prefillUrl.value = null;
});
```

在 `handleTaskClick` 定义之后插入：

```typescript
// 右键菜单「以此链接重新下载」：预填 URL 并打开添加对话框
const handleRedownload = (task: DownloadTask) => {
  prefillUrl.value = task.url;
  showAddDialog.value = true;
};
```

template 中两处 `<TaskList>` 的（`replace_all` 语义）：

```vue
          @task-click="handleTaskClick"
```

改为：

```vue
          @task-click="handleTaskClick"
          @task-redownload="handleRedownload"
```

```vue
    <AddTaskDialog v-model:open="showAddDialog" />
```

改为：

```vue
    <AddTaskDialog v-model:open="showAddDialog" :initial-url="prefillUrl" />
```

- [ ] **Step 7: 全量验证**

Run: `npm run type-check && npm run lint && npm test`
Expected: 三项全部通过。

- [ ] **Step 8: 提交**

```bash
git add src/components/task/TaskList.vue src/views/HomeView.vue src/components/task/AddTaskDialog.vue src/components/task/AddTaskDialog.test.ts
git commit -m "feat: 重新下载事件链——右键菜单预填添加对话框

TaskCard redownload 沿 TaskList 透传至 HomeView，prefillUrl 一次性
交接给 AddTaskDialog（新 prop initialUrl）；打开时复用 submitPaste
自动推进到配置步，重复 URL 自然走既有 UrlDuplicateDialog 流程。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 8: 功能状态文档 + 手动验收 + 终检

**Files:**
- Modify: `docs/design/06-feature-status.md`（「十、UI/UX」表）

**Interfaces:**
- Consumes: Task 1–7 已完成的实现
- Produces: 无

- [ ] **Step 1: 更新功能状态表**

`docs/design/06-feature-status.md` 中找到：

```markdown
| 任务卡片 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 渐进式披露（紧凑→悬停→点击详情） |
```

在其后插入一行：

```markdown
| 任务卡片右键菜单 | P2 | `[x]` | `src/components/task/TaskContextMenu.vue` | 右键菜单收纳复制链接/文件名/路径、以此链接重新下载（预填添加对话框）、打开详情 |
```

- [ ] **Step 2: 手动运行验证（GUI，需人工确认）**

Run: `npm run tauri dev`

按 spec §6.2 清单逐项确认：

1. 右键 pending / downloading / paused / completed / failed 五种状态卡片 → 菜单在光标处打开，条目构成符合 spec §4 矩阵（「复制文件路径」仅在 completed 且有 outputPath 时出现）
2. 三种复制 → 剪贴板内容正确（Ctrl+V 验证）+ 右下角 toast 出现
3. 重新下载 → 添加对话框打开且**自动到达配置步**、URL 已填入；调整文件名确认后新任务创建成功
4. 对已存在 URL 重新下载 → 自然进入 UrlDuplicateDialog 流程
5. Esc / 点击区域外关闭菜单；↑↓ 键导航；右键另一张卡片菜单切换
6. 无回归：左键点开详情正常；悬停快捷按钮正常；右键悬停按钮所在区域也弹出菜单
7. 设置中切换语言至 en-US / zh-TW → 菜单与 toast 文案正确

- [ ] **Step 3: 提交文档**

```bash
git add docs/design/06-feature-status.md
git commit -m "docs: 更新功能状态——任务卡片右键菜单

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

- [ ] **Step 4: 终检**

Run: `npm run type-check && npm run lint && npm test && git status --short && git log --oneline origin/main..HEAD`
Expected: 三项检查通过；工作区仅剩本计划文档（`docs/superpowers/plans/`）等无关未跟踪文件；分支提交链完整（spec + Task 1–8 提交）。
