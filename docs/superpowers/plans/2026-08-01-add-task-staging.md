# 添加任务暂存层实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `AddTaskDialog` 从「输入+全部配置混在一起」重构为「主从详情式暂存层」——粘贴→逐条聚焦配置→提交，配置项按链接类型动态显示，消除「直链设 MKV 是无操作」的歧义。

**Architecture:** 新增两个组件（`TaskStagingList`=L1 总览、`LinkConfigPanel`=L2 聚焦），`AddTaskDialog` 退化为纯编排外壳。引擎专属可见性由一张纯函数表 `linkOptionVisibility` 驱动；三层合并（逐条>批次>全局）由纯函数 `resolveLinkToTask` 持有。`StreamSelector` 的选择体抽成 `StreamPickerInline` 供聚焦面板内联使用，不再「弹窗套弹窗」。后端命令/引擎/schema 零改动。

**Tech Stack:** Vue 3 `<script setup>` + TS + vitest（node 纯函数环境，无 DOM）+ shadcn-vue + Tauri invoke。

## Global Constraints

- **禁止 `any`**；所有新增 TS 显式类型（见 `CLAUDE.md`）。
- 组件/Store 不直接调 Tauri API，经 `src/services/`（`useDownloader`/`taskStore` 已封装）。
- 后端契约不变：`addAndStartTask(url, fileName?, saveDir?, overrides?)` 与 `taskStore.addTask({url,fileName?,saveDir?,overrides?,skipUrlCheck?})`。
- vitest 仅 node 环境，**无组件渲染测试**：纯函数走 TDD，组件靠 `npm run type-check` + `npm run tauri dev` 手动验证。
- 提交规范：`feat`/`refactor`/`test` 前缀，结尾 `Co-Authored-By: Claude <noreply@anthropic.com>`。
- 每个任务结束前必须 `npm run type-check` 通过。

## File Structure

| 文件 | 责任 | 动作 |
|---|---|---|
| `src/components/task/staging-types.ts` | 暂存层类型（`StagedLink`/`BatchDefaults`/`LinkOption`/`LinkStatus`） | 新建 |
| `src/components/task/linkOptionVisibility.ts` | 选项可见性纯谓词 | 新建 |
| `src/components/task/linkOptionVisibility.test.ts` | 可见性单测 | 新建 |
| `src/components/task/resolveLinkToTask.ts` | 三层合并 + 预设播种纯函数 | 新建 |
| `src/components/task/resolveLinkToTask.test.ts` | 合并/播种单测 | 新建 |
| `src/components/stream/StreamPickerInline.vue` | 流选择体（无 Dialog 外壳） | 新建 |
| `src/components/stream/StreamSelector.vue` | 退化为薄 Dialog 外壳，包 `StreamPickerInline` | 改 |
| `src/components/stream/index.ts` | 导出 `StreamPickerInline` | 改 |
| `src/components/task/LinkConfigPanel.vue` | L2 单条聚焦配置 | 新建 |
| `src/components/task/TaskStagingList.vue` | L1 粘贴+行清单+批次默认 | 新建 |
| `src/components/task/AddTaskDialog.vue` | 编排外壳（状态机+提交） | 重写 |
| `docs/design/06-feature-status.md` | 状态追踪更新 | 改 |

---

## Task 1: 暂存层类型 + 选项可见性纯函数

**Files:**
- Create: `src/components/task/staging-types.ts`
- Create: `src/components/task/linkOptionVisibility.ts`
- Test: `src/components/task/linkOptionVisibility.test.ts`

**Interfaces:**
- Produces: `StagedLink`、`BatchDefaults`、`LinkOption`、`LinkStatus`（来自 `staging-types.ts`）；`isOptionVisible(option: LinkOption, urlType: UrlType | null): boolean`（来自 `linkOptionVisibility.ts`）。后续任务消费这些类型与函数。

- [ ] **Step 1: 写 `staging-types.ts`**

```ts
// src/components/task/staging-types.ts
import type { StreamInfo, TaskOverrides, UrlType } from "@/domain";

/** 聚焦面板中按类型动态显示的配置项 */
export type LinkOption =
  | "fileName"
  | "saveDir"
  | "schedule"
  | "maxSpeed"
  | "customRange"
  | "muxFormat"
  | "subtitleFormat"
  | "subtitlesOnly"
  | "streamSelection"
  | "key";

/** 暂存链接状态：pending 未查看 / parsed 已解析 / ready 已确认 / invalid 无效 */
export type LinkStatus = "pending" | "parsed" | "ready" | "invalid";

/** 单条暂存链接（仅前端暂存层使用，不进领域/后端） */
export interface StagedLink {
  id: string;
  url: string;
  detectedType: UrlType | null;
  fileName: string;
  saveDir: string;
  overrides: TaskOverrides;
  status: LinkStatus;
  streamInfo?: StreamInfo;
}

/** 批次默认（AddTaskDialog 持有，不随任务持久化） */
export interface BatchDefaults {
  saveDir: string;
  autoStart: boolean;
}
```

- [ ] **Step 2: 写 `linkOptionVisibility.ts`**

```ts
// src/components/task/linkOptionVisibility.ts
import { isStreamingType } from "@/domain/url";
import type { UrlType } from "@/domain";
import type { LinkOption } from "./staging-types";

/** 仅流媒体（HLS/DASH/MSS）行可见的选项 */
const STREAMING_ONLY: ReadonlySet<LinkOption> = new Set<LinkOption>([
  "maxSpeed",
  "customRange",
  "muxFormat",
  "subtitleFormat",
  "subtitlesOnly",
  "streamSelection",
  "key",
]);

/**
 * 某选项在给定 URL 类型下是否可见。
 * 通用项（fileName/saveDir/schedule）始终可见；
 * 流媒体专属项仅当类型为 HLS/DASH/MSS 时可见。
 */
export function isOptionVisible(
  option: LinkOption,
  urlType: UrlType | null,
): boolean {
  if (STREAMING_ONLY.has(option)) {
    return urlType !== null && isStreamingType(urlType);
  }
  return true;
}
```

- [ ] **Step 3: 写失败测试**

```ts
// src/components/task/linkOptionVisibility.test.ts
import { describe, it, expect } from "vitest";
import { isOptionVisible } from "./linkOptionVisibility";
import type { UrlType } from "@/domain";

const T = (s: string) => s as UrlType;

describe("isOptionVisible", () => {
  it("通用选项对任意类型（含 null）可见", () => {
    expect(isOptionVisible("fileName", null)).toBe(true);
    expect(isOptionVisible("saveDir", T("httpVideo"))).toBe(true);
    expect(isOptionVisible("schedule", T("hls"))).toBe(true);
  });

  it("流媒体选项对直链/未知/null 不可见", () => {
    expect(isOptionVisible("maxSpeed", T("httpVideo"))).toBe(false);
    expect(isOptionVisible("muxFormat", T("unknown"))).toBe(false);
    expect(isOptionVisible("streamSelection", null)).toBe(false);
    expect(isOptionVisible("key", T("httpVideo"))).toBe(false);
  });

  it("流媒体选项对流媒体可见", () => {
    expect(isOptionVisible("maxSpeed", T("hls"))).toBe(true);
    expect(isOptionVisible("muxFormat", T("dash"))).toBe(true);
    expect(isOptionVisible("subtitlesOnly", T("mss"))).toBe(true);
    expect(isOptionVisible("customRange", T("hls"))).toBe(true);
    expect(isOptionVisible("subtitleFormat", T("dash"))).toBe(true);
    expect(isOptionVisible("streamSelection", T("mss"))).toBe(true);
    expect(isOptionVisible("key", T("hls"))).toBe(true);
  });
});
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npm test -- linkOptionVisibility`
Expected: PASS（3 测试全过）

- [ ] **Step 5: type-check**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 6: 提交**

```bash
git add src/components/task/staging-types.ts src/components/task/linkOptionVisibility.ts src/components/task/linkOptionVisibility.test.ts
git commit -m "feat(task): 暂存层类型与选项可见性纯函数

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 三层合并 + 预设播种纯函数

**Files:**
- Create: `src/components/task/resolveLinkToTask.ts`
- Test: `src/components/task/resolveLinkToTask.test.ts`

**Interfaces:**
- Consumes: `StagedLink`、`BatchDefaults`（Task 1）；`TaskOverrides`、`UrlType`（`@/domain`）；`isStreamingType`（`@/domain/url`）。
- Produces: `ResolvedTask`、`cleanOverrides(overrides): TaskOverrides | undefined`、`resolveLinkToTask(link, batch, globalSaveDir): ResolvedTask`、`seedPresetOverrides(preset, urlType): TaskOverrides`。Task 6（AddTaskDialog）在提交循环里调用 `resolveLinkToTask`，在创建/重播种时调用 `seedPresetOverrides`。

- [ ] **Step 1: 写 `resolveLinkToTask.ts`**

```ts
// src/components/task/resolveLinkToTask.ts
import { isStreamingType } from "@/domain/url";
import type { TaskOverrides, UrlType } from "@/domain";
import type { BatchDefaults, StagedLink } from "./staging-types";

/** 解析后的任务规格（喂给 addAndStartTask / taskStore.addTask） */
export interface ResolvedTask {
  url: string;
  fileName?: string;
  saveDir?: string;
  overrides?: TaskOverrides;
  hasSchedule: boolean;
}

function firstNonEmpty(...vals: string[]): string | undefined {
  const found = vals.map((v) => v.trim()).find((v) => v.length > 0);
  return found || undefined;
}

/** 剔除空字段，返回干净 TaskOverrides（无字段则 undefined） */
export function cleanOverrides(
  overrides: TaskOverrides,
): TaskOverrides | undefined {
  const o: TaskOverrides = {};
  if (overrides.saveDir) o.saveDir = overrides.saveDir;
  if (overrides.saveName) o.saveName = overrides.saveName;
  if (overrides.muxFormat) o.muxFormat = overrides.muxFormat;
  if (overrides.maxSpeed) o.maxSpeed = overrides.maxSpeed;
  if (overrides.customRange) o.customRange = overrides.customRange;
  if (overrides.subtitleFormat) o.subtitleFormat = overrides.subtitleFormat;
  if (overrides.subtitlesOnly != null) o.subtitlesOnly = overrides.subtitlesOnly;
  if (overrides.scheduledStartAt) o.scheduledStartAt = overrides.scheduledStartAt;
  if (overrides.selection) o.selection = overrides.selection;
  if (overrides.presetId) o.presetId = overrides.presetId;
  if (overrides.key) o.key = overrides.key;
  const hasAny = Object.values(o).some((v) => v !== undefined && v !== null);
  return hasAny ? o : undefined;
}

/**
 * 合并「逐条 > 批次默认 > 全局默认」三层，产出可直接建任务的规格。
 * 唯一的合并规则持有者（设计 4.4）。
 */
export function resolveLinkToTask(
  link: StagedLink,
  batch: BatchDefaults,
  globalSaveDir: string,
): ResolvedTask {
  const saveDir = firstNonEmpty(link.saveDir, batch.saveDir, globalSaveDir);
  const fileName = link.fileName.trim() || undefined;
  const overrides = cleanOverrides(link.overrides);
  const hasSchedule = !!overrides?.scheduledStartAt;
  return { url: link.url, fileName, saveDir, overrides, hasSchedule };
}

/**
 * 预设作为「初值提供者」（设计 4.3）：
 * 仅流媒体行接受预设 overrides 作初值；直链行返回空对象。
 * selection 做浅拷贝避免多行共享同一引用。
 */
export function seedPresetOverrides(
  preset: TaskOverrides | null,
  urlType: UrlType | null,
): TaskOverrides {
  if (urlType === null || !isStreamingType(urlType)) return {};
  if (!preset) return {};
  return {
    ...preset,
    selection: preset.selection ? { ...preset.selection } : undefined,
  };
}
```

- [ ] **Step 2: 写失败测试**

```ts
// src/components/task/resolveLinkToTask.test.ts
import { describe, it, expect } from "vitest";
import {
  cleanOverrides,
  resolveLinkToTask,
  seedPresetOverrides,
} from "./resolveLinkToTask";
import type { BatchDefaults, StagedLink } from "./staging-types";
import type { TaskOverrides, UrlType } from "@/domain";

const T = (s: string) => s as UrlType;

function mkLink(over: Partial<TaskOverrides> = {}): StagedLink {
  return {
    id: "1",
    url: "https://x/a.m3u8",
    detectedType: T("hls"),
    fileName: "a",
    saveDir: "",
    overrides: over as TaskOverrides,
    status: "pending",
  };
}
const BATCH: BatchDefaults = { saveDir: "D:/batch", autoStart: true };

describe("cleanOverrides", () => {
  it("剔除空字段，全空返回 undefined", () => {
    expect(cleanOverrides({} as TaskOverrides)).toBeUndefined();
    expect(cleanOverrides({ maxSpeed: "" } as TaskOverrides)).toBeUndefined();
  });
  it("保留非空字段", () => {
    const o = cleanOverrides({ maxSpeed: "5M" } as TaskOverrides);
    expect(o?.maxSpeed).toBe("5M");
  });
});

describe("resolveLinkToTask", () => {
  it("saveDir 继承顺序：行 > 批次 > 全局", () => {
    expect(resolveLinkToTask(mkLink(), BATCH, "D:/global").saveDir).toBe(
      "D:/batch",
    );
    expect(
      resolveLinkToTask({ ...mkLink(), saveDir: "D:/row" }, BATCH, "D:/global")
        .saveDir,
    ).toBe("D:/row");
    expect(
      resolveLinkToTask(mkLink(), { saveDir: "", autoStart: true }, "D:/global")
        .saveDir,
    ).toBe("D:/global");
  });
  it("空 overrides → undefined", () => {
    expect(
      resolveLinkToTask(mkLink(), BATCH, "D:/global").overrides,
    ).toBeUndefined();
  });
  it("hasSchedule 由 scheduledStartAt 决定", () => {
    expect(
      resolveLinkToTask(
        mkLink({ scheduledStartAt: "2026-01-01T00:00:00" }),
        BATCH,
        "D:/global",
      ).hasSchedule,
    ).toBe(true);
    expect(resolveLinkToTask(mkLink(), BATCH, "D:/global").hasSchedule).toBe(
      false,
    );
  });
  it("fileName 空时 undefined", () => {
    expect(
      resolveLinkToTask({ ...mkLink(), fileName: "  " }, BATCH, "D:/global")
        .fileName,
    ).toBeUndefined();
  });
});

describe("seedPresetOverrides", () => {
  const preset = { maxSpeed: "5M", selection: { video: "res:1080" } } as TaskOverrides;
  it("流媒体行接受预设初值", () => {
    expect(seedPresetOverrides(preset, T("hls"))).toEqual({
      maxSpeed: "5M",
      selection: { video: "res:1080" },
    });
  });
  it("直链/未知/null 返回空对象", () => {
    expect(seedPresetOverrides(preset, T("httpVideo"))).toEqual({});
    expect(seedPresetOverrides(preset, T("unknown"))).toEqual({});
    expect(seedPresetOverrides(preset, null)).toEqual({});
  });
  it("selection 不共享引用（拷贝）", () => {
    const a = seedPresetOverrides(preset, T("hls"));
    const b = seedPresetOverrides(preset, T("hls"));
    expect(a.selection).not.toBe(b.selection);
    expect(a.selection).toEqual(b.selection);
  });
  it("null 预设返回空对象", () => {
    expect(seedPresetOverrides(null, T("hls"))).toEqual({});
  });
});
```

- [ ] **Step 3: 运行测试确认通过**

Run: `npm test -- resolveLinkToTask`
Expected: PASS（cleanOverrides 2 + resolve 4 + seed 4 = 10 测试全过）

- [ ] **Step 4: type-check**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 5: 提交**

```bash
git add src/components/task/resolveLinkToTask.ts src/components/task/resolveLinkToTask.test.ts
git commit -m "feat(task): 三层合并与预设播种纯函数

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 流选择体内联化（抽 `StreamPickerInline`，`StreamSelector` 退化为外壳）

**Files:**
- Create: `src/components/stream/StreamPickerInline.vue`
- Modify: `src/components/stream/StreamSelector.vue`（整文件替换为薄外壳）
- Modify: `src/components/stream/index.ts`（加导出）

**Interfaces:**
- Produces: `StreamPickerInline`（props `{ streamInfo: StreamInfo|null; loading?: boolean }`，emits `{ confirm(selection: StreamSelection); cancel() }`）。Task 4 的 `LinkConfigPanel` 内联使用它；`StreamSelector` 也包它。

- [ ] **Step 1: 写 `StreamPickerInline.vue`**

```vue
<!-- src/components/stream/StreamPickerInline.vue -->
<script setup lang="ts">
/**
 * 流选择体（无 Dialog 外壳）。
 * 供 LinkConfigPanel 内联嵌入；StreamSelector 也可包它做独立弹窗。
 * 业务逻辑在 useStreamSelector。
 */
import { toRef } from "vue";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AppIcon } from "@/components/common";
import { StreamList } from "@/components/stream";
import { useStreamSelector } from "@/composables/useStreamSelector";
import type { StreamInfo, StreamSelection } from "@/domain";

const props = defineProps<{
  streamInfo: StreamInfo | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "confirm", selection: StreamSelection): void;
  (e: "cancel"): void;
}>();

const selector = useStreamSelector(toRef(props, "streamInfo"));

const handleConfirm = () => emit("confirm", selector.getSelection());
const handleCancel = () => emit("cancel");
</script>

<template>
  <div class="flex flex-col">
    <!-- 统计 -->
    <p
      v-if="selector.stats.value"
      class="mb-3 text-sm text-muted-foreground"
    >
      共 {{ selector.stats.value.videoCount }} 个视频流、{{
        selector.stats.value.audioCount
      }}个音频流、{{ selector.stats.value.subtitleCount }}个字幕流
      <span v-if="selector.stats.value.duration !== '未知'"
        >· 时长 {{ selector.stats.value.duration }}</span
      >
      <span v-if="selector.stats.value.isLive" class="ml-1 text-red-400"
        >· 直播</span
      >
      <span v-if="selector.stats.value.isEncrypted" class="ml-1 text-yellow-400"
        >· 加密</span
      >
    </p>

    <!-- 加载 -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <div class="flex flex-col items-center gap-3">
        <AppIcon name="Loader2" :size="32" class="animate-spin text-primary" />
        <span class="text-muted-foreground">正在解析流信息...</span>
      </div>
    </div>

    <!-- 无数据 -->
    <div
      v-else-if="!streamInfo"
      class="flex items-center justify-center py-12"
    >
      <div class="flex flex-col items-center gap-3">
        <AppIcon
          name="AlertCircle"
          :size="32"
          class="text-muted-foreground"
        />
        <span class="text-muted-foreground">无法获取流信息</span>
      </div>
    </div>

    <!-- 流列表 -->
    <template v-else>
      <Tabs
        v-model="selector.activeTab.value"
        class="flex min-h-0 flex-col"
      >
        <TabsList class="grid w-full shrink-0 grid-cols-3">
          <TabsTrigger value="video">
            <AppIcon name="Video" :size="16" class="mr-1.5" />
            视频 ({{ streamInfo.videos.length }})
          </TabsTrigger>
          <TabsTrigger value="audio">
            <AppIcon name="Music" :size="16" class="mr-1.5" />
            音频 ({{ streamInfo.audios.length }})
          </TabsTrigger>
          <TabsTrigger value="subtitle">
            <AppIcon name="Subtitles" :size="16" class="mr-1.5" />
            字幕 ({{ streamInfo.subtitles.length }})
          </TabsTrigger>
        </TabsList>

        <ScrollArea class="mt-3 max-h-[40vh]">
          <TabsContent value="video" class="mt-0">
            <StreamList
              :streams="streamInfo.videos"
              :selected-ids="selector.selectedVideos.value"
              type="video"
              empty-text="没有可用的视频流"
              @toggle="selector.toggleVideo"
            />
          </TabsContent>
          <TabsContent value="audio" class="mt-0">
            <StreamList
              :streams="streamInfo.audios"
              :selected-ids="selector.selectedAudios.value"
              type="audio"
              show-select-all
              empty-text="没有可用的音频流"
              @toggle="selector.toggleAudio"
              @toggle-all="selector.toggleAllAudio"
            />
          </TabsContent>
          <TabsContent value="subtitle" class="mt-0">
            <StreamList
              :streams="streamInfo.subtitles"
              :selected-ids="selector.selectedSubtitles.value"
              type="subtitle"
              show-select-all
              empty-text="没有可用的字幕流"
              @toggle="selector.toggleSubtitle"
              @toggle-all="selector.toggleAllSubtitle"
            />
          </TabsContent>
        </ScrollArea>
      </Tabs>
    </template>

    <!-- 操作 -->
    <div class="mt-3 flex shrink-0 justify-end gap-2 border-t pt-3">
      <Button variant="outline" size="sm" @click="handleCancel">取消</Button>
      <Button
        size="sm"
        :disabled="!selector.canConfirm.value"
        @click="handleConfirm"
      >
        <AppIcon name="Check" :size="16" class="mr-1.5" />
        确认选择
      </Button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 重写 `StreamSelector.vue` 为薄外壳**

```vue
<!-- src/components/stream/StreamSelector.vue -->
<script setup lang="ts">
/**
 * 流选择器（独立弹窗形态）。薄 Dialog 外壳，包 StreamPickerInline。
 * AddTaskDialog 重构后不再直接使用；保留以供独立调用与导出稳定。
 */
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { AppIcon } from "@/components/common";
import StreamPickerInline from "./StreamPickerInline.vue";
import type { StreamInfo, StreamSelection } from "@/domain";

const props = defineProps<{
  open: boolean;
  streamInfo: StreamInfo | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm", selection: StreamSelection): void;
  (e: "cancel"): void;
}>();

const close = () => emit("update:open", false);
const onConfirm = (s: StreamSelection) => {
  emit("confirm", s);
  close();
};
const onCancel = () => {
  emit("cancel");
  close();
};
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent class="flex max-h-[85vh] max-w-2xl flex-col">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="ListVideo" :size="20" />
          选择流
        </DialogTitle>
        <DialogDescription>选择要下载的视频/音频/字幕流</DialogDescription>
      </DialogHeader>
      <StreamPickerInline
        :stream-info="props.streamInfo"
        :loading="props.loading"
        @confirm="onConfirm"
        @cancel="onCancel"
      />
    </DialogContent>
  </Dialog>
</template>
```

- [ ] **Step 3: 更新 `index.ts` 导出**

```ts
// src/components/stream/index.ts
/**
 * 流组件导出
 */

export { default as StreamSelector } from "./StreamSelector.vue";
export { default as StreamPickerInline } from "./StreamPickerInline.vue";
export { default as StreamItem } from "./StreamItem.vue";
export { default as StreamList } from "./StreamList.vue";
```

- [ ] **Step 4: type-check**

Run: `npm run type-check`
Expected: 无错误（`AppIcon` 的 `Check`/`ListVideo` 等图标名若不存在则按现有图标库替换；运行时由 dev 验证）

- [ ] **Step 5: 提交**

```bash
git add src/components/stream/StreamPickerInline.vue src/components/stream/StreamSelector.vue src/components/stream/index.ts
git commit -m "refactor(stream): 抽 StreamPickerInline，StreamSelector 退化为薄外壳

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: `LinkConfigPanel`（L2 聚焦配置）

**Files:**
- Create: `src/components/task/LinkConfigPanel.vue`

**Interfaces:**
- Consumes: `StagedLink`、`LinkOption`（Task 1）；`isOptionVisible`（Task 1）；`StreamPickerInline`（Task 3）；`parseUrl`、`isParsing`（`useDownloader`）；`usePresetManager.applyPreset`（返回 `TaskOverrides | null`）；`isStreamingType`（`@/domain/url`）。
- Produces: `LinkConfigPanel`，`v-model` 绑定一个 `StagedLink`（深层字段双向）；emits `{ done() }`（用户点「完成」→ 父置 `status='ready'` 并退回列表）。父传入 `saveDirPlaceholder: string`（合并后的占位提示，面板不读批次/全局，见设计 4.4）。

- [ ] **Step 1: 写 `LinkConfigPanel.vue`**

```vue
<!-- src/components/task/LinkConfigPanel.vue -->
<script setup lang="ts">
/**
 * 单条链接聚焦配置（L2）。
 * 按 detectedType 动态渲染：通用三件（文件名/保存位置/定时）始终可见；
 * 流媒体专属项（限速/范围/容器/字幕/流选择/解密）经 isOptionVisible 控制。
 * 流选择内联嵌入 StreamPickerInline，不再弹窗套弹窗。
 */
import { computed, ref } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppIcon } from "@/components/common";
import { StreamPickerInline } from "@/components/stream";
import { useDownloader, usePresetManager } from "@/composables";
import { isStreamingType } from "@/domain/url";
import { isOptionVisible } from "./linkOptionVisibility";
import type { StagedLink } from "./staging-types";
import type { MuxFormat, StreamSelection, SubtitleFormat } from "@/domain";

const props = defineProps<{
  saveDirPlaceholder: string;
}>();

const emit = defineEmits<{ (e: "done"): void }>();

/** 绑定整条 StagedLink（深层字段就地修改，响应式回传父） */
const link = defineModel<StagedLink>({ required: true });

const { parseUrl, isParsing } = useDownloader();

const isStreaming = computed(
  () =>
    link.value.detectedType !== null && isStreamingType(link.value.detectedType),
);

/** 流选择折叠 */
const showStreamPicker = ref(false);

/** 单链接流媒体：进入即自动解析一次（沿用原单链接体验） */
const autoParsed = ref(false);
async function ensureParsed() {
  if (!isStreaming.value || autoParsed.value) return;
  autoParsed.value = true;
  await handleParse();
}

async function handleParse() {
  if (!isStreaming.value) return;
  const info = await parseUrl(link.value.url);
  if (info) {
    link.value.streamInfo = info;
    link.value.status = "parsed";
    showStreamPicker.value = true;
  }
}

function handleStreamConfirm(sel: StreamSelection) {
  link.value.overrides.selection = sel;
  showStreamPicker.value = false;
}

function handleStreamCancel() {
  showStreamPicker.value = false;
}

/** 最小调度时间 */
const minScheduleTime = computed(() => {
  const now = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
});

const scheduleTime = computed({
  get: () => link.value.overrides.scheduledStartAt ?? "",
  set: (v: string) => {
    link.value.overrides.scheduledStartAt = v || undefined;
  },
});

const typeBadgeLabel = computed(() => {
  const t = link.value.detectedType;
  if (!t) return "";
  const labels: Record<string, string> = {
    hls: "HLS",
    dash: "DASH",
    mss: "MSS",
    httpVideo: "直链视频",
    unknown: "未知",
  };
  return labels[t] ?? "";
});

// 组件挂载后若是单链接流媒体场景由父触发 ensureParsed；这里保留入口供父调用
defineExpose({ ensureParsed });
</script>

<template>
  <div class="space-y-4">
    <!-- 行头：文件名 + 类型徽章 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">文件名</Label>
      <Input
        v-model="link.fileName"
        placeholder="自动从 URL 提取"
        class="h-9 text-sm"
      />
      <div class="flex items-center gap-2 text-xs">
        <span
          v-if="typeBadgeLabel"
          class="rounded-full bg-primary/20 px-2 py-0.5 font-medium text-primary"
          >{{ typeBadgeLabel }}</span
        >
      </div>
    </div>

    <!-- 保存位置（通用） -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">保存位置</Label>
      <Input
        v-model="link.saveDir"
        :placeholder="props.saveDirPlaceholder"
        class="h-9 text-sm"
      />
    </div>

    <!-- 定时开始（通用） -->
    <div class="space-y-1.5">
      <div class="flex items-center justify-between">
        <Label class="cursor-pointer text-xs text-muted-foreground"
          >定时开始</Label
        >
        <Switch
          :checked="!!scheduleTime"
          @update:checked="(v: boolean) => (scheduleTime = v ? minScheduleTime : '')"
        />
      </div>
      <Input
        v-if="scheduleTime"
        v-model="scheduleTime"
        type="datetime-local"
        :min="minScheduleTime"
        class="datetime-dark h-9 text-sm"
      />
    </div>

    <!-- 流媒体专属（按 isOptionVisible 动态） -->
    <template v-if="isStreaming">
      <div class="space-y-3 border-t border-border/60 pt-3">
        <!-- 解析流 -->
        <div v-if="isOptionVisible('streamSelection', link.detectedType)" class="space-y-1.5">
          <Label class="text-xs text-muted-foreground">流选择</Label>
          <div class="flex gap-2">
            <Button variant="outline" size="sm" class="h-9" @click="handleParse">
              <AppIcon
                v-if="isParsing"
                name="Loader2"
                :size="14"
                class="mr-1.5 animate-spin"
              />
              <AppIcon v-else name="Search" :size="14" class="mr-1.5" />
              {{ link.streamInfo ? "重新解析" : "解析流" }}
            </Button>
            <Button
              v-if="link.streamInfo"
              variant="ghost"
              size="sm"
              class="h-9"
              @click="showStreamPicker = !showStreamPicker"
            >
              <AppIcon name="ListVideo" :size="14" class="mr-1.5" />
              {{ showStreamPicker ? "收起" : "选择流" }}
            </Button>
          </div>
          <p v-if="link.overrides.selection" class="text-xs text-muted-foreground/70">
            已选：视频 {{ link.overrides.selection.video ?? "自动" }} · 音频
            {{ link.overrides.selection.audio ?? "自动" }} · 字幕
            {{ link.overrides.selection.subtitle ?? "自动" }}
          </p>
          <!-- 内联流选择体 -->
          <div
            v-if="showStreamPicker && link.streamInfo"
            class="rounded-lg border bg-muted/30 p-3"
          >
            <StreamPickerInline
              :stream-info="link.streamInfo"
              :loading="isParsing"
              @confirm="handleStreamConfirm"
              @cancel="handleStreamCancel"
            />
          </div>
        </div>

        <!-- 限速 -->
        <div
          v-if="isOptionVisible('maxSpeed', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">限速</Label>
          <Input
            v-model="link.overrides.maxSpeed"
            placeholder="如 10M，留空跟随全局"
            class="h-9 text-sm"
          />
        </div>

        <!-- 下载范围 -->
        <div
          v-if="isOptionVisible('customRange', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">下载范围</Label>
          <Input
            v-model="link.overrides.customRange"
            placeholder="如 00:00:00-00:10:00"
            class="h-9 text-sm"
          />
        </div>

        <!-- 容器格式 -->
        <div
          v-if="isOptionVisible('muxFormat', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">容器格式</Label>
          <Select
            :model-value="link.overrides.muxFormat ?? ''"
            @update:model-value="
              (v: string) =>
                (link.overrides.muxFormat = (v || undefined) as MuxFormat | undefined)
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">跟随全局</SelectItem>
              <SelectItem value="mp4">MP4</SelectItem>
              <SelectItem value="mkv">MKV</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 字幕格式 -->
        <div
          v-if="isOptionVisible('subtitleFormat', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">字幕格式</Label>
          <Select
            :model-value="link.overrides.subtitleFormat ?? ''"
            @update:model-value="
              (v: string) =>
                (link.overrides.subtitleFormat =
                  (v || undefined) as SubtitleFormat | undefined)
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">跟随全局</SelectItem>
              <SelectItem value="SRT">SRT</SelectItem>
              <SelectItem value="VTT">VTT</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 仅下载字幕 -->
        <div
          v-if="isOptionVisible('subtitlesOnly', link.detectedType)"
          class="flex items-center justify-between"
        >
          <Label class="cursor-pointer text-xs text-muted-foreground"
            >仅下载字幕</Label
          >
          <Switch
            :checked="!!link.overrides.subtitlesOnly"
            @update:checked="(v: boolean) => (link.overrides.subtitlesOnly = v)"
          />
        </div>

        <!-- 任务级解密密钥 -->
        <div
          v-if="isOptionVisible('key', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">解密密钥</Label>
          <Input
            v-model="link.overrides.key"
            placeholder="全局密钥库为空时生效"
            class="h-9 text-sm"
          />
        </div>
      </div>
    </template>

    <!-- 完成 -->
    <div class="flex justify-end border-t pt-3">
      <Button size="sm" @click="emit('done')">
        <AppIcon name="Check" :size="16" class="mr-1.5" />
        完成
      </Button>
    </div>
  </div>
</template>

<style scoped>
.datetime-dark {
  color-scheme: dark;
}
</style>
```

- [ ] **Step 2: type-check**

Run: `npm run type-check`
Expected: 无错误。若 `AppIcon` 缺 `Search`/`Check` 图标，替换为库中已有图标（如 `Plus`/`Download`）。

- [ ] **Step 3: 提交**

```bash
git add src/components/task/LinkConfigPanel.vue
git commit -m "feat(task): LinkConfigPanel 单条聚焦配置（引擎类型驱动可见性）

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: `TaskStagingList`（L1 总览）

**Files:**
- Create: `src/components/task/TaskStagingList.vue`

**Interfaces:**
- Consumes: `StagedLink`、`BatchDefaults`（Task 1）；`presets`（`usePresetManager`）。
- Produces: `TaskStagingList`，props `{ links: StagedLink[]; batch: BatchDefaults; batchPresetId: string; dragEnabled?: boolean }`；emits `{ update:batch(b: BatchDefaults); paste(text: string); select(id: string); remove(id: string); commit() }`。不碰 overrides、不碰合并规则（设计 4.4）。

- [ ] **Step 1: 写 `TaskStagingList.vue`**

```vue
<!-- src/components/task/TaskStagingList.vue -->
<script setup lang="ts">
/**
 * L1 总览：粘贴框 + 批次公共默认 + 紧凑行清单。
 * 只渲染 + 通知；不构造 StagedLink（paste 事件交父）、不碰 overrides、不碰合并。
 */
import { ref } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppIcon } from "@/components/common";
import { usePresetManager } from "@/composables";
import type { BatchDefaults, StagedLink } from "./staging-types";

const props = defineProps<{
  links: StagedLink[];
  batch: BatchDefaults;
  batchPresetId: string;
  globalSaveDir: string;
}>();

const emit = defineEmits<{
  (e: "update:batch", b: BatchDefaults): void;
  (e: "update:preset", presetId: string): void;
  (e: "paste", text: string): void;
  (e: "select", id: string): void;
  (e: "remove", id: string): void;
  (e: "commit"): void;
}>();

const { presets } = usePresetManager();

const pasteText = ref("");
const isDragging = ref(false);

function onPaste() {
  emit("paste", pasteText.value);
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}
function onDragLeave() {
  isDragging.value = false;
}
function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const text = e.dataTransfer?.getData("text/plain");
  if (text) emit("paste", text);
}

function patchBatch(patch: Partial<BatchDefaults>) {
  emit("update:batch", { ...props.batch, ...patch });
}

const statusColor: Record<string, string> = {
  pending: "text-muted-foreground",
  parsed: "text-primary",
  ready: "text-primary",
  invalid: "text-accent-error",
};
const statusLabel: Record<string, string> = {
  pending: "待配置",
  parsed: "已解析",
  ready: "就绪",
  invalid: "无效",
};

const saveDirPlaceholder = props.globalSaveDir || "使用全局默认";
</script>

<template>
  <div class="space-y-4">
    <!-- 粘贴框 -->
    <div
      class="relative"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <div
        v-if="isDragging"
        class="absolute inset-0 z-10 flex items-center justify-center rounded-lg border-2 border-dashed border-primary bg-primary/10"
      >
        <span class="text-sm font-medium text-primary">释放以添加链接</span>
      </div>
      <textarea
        v-model="pasteText"
        placeholder="粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 等）"
        class="h-20 w-full resize-none rounded-lg border bg-muted/50 px-3 py-2 text-sm transition-colors focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/50"
        @blur="onPaste"
      />
    </div>

    <!-- 批次公共默认 -->
    <div class="grid grid-cols-1 gap-3 rounded-lg border bg-muted/20 p-3 sm:grid-cols-3">
      <div class="space-y-1.5">
        <Label class="text-xs text-muted-foreground">保存位置（本批默认）</Label>
        <div class="flex gap-2">
          <Input
            :model-value="props.batch.saveDir"
            :placeholder="saveDirPlaceholder"
            class="h-9 flex-1 text-sm"
            @update:model-value="(v: string) => patchBatch({ saveDir: v })"
          />
          <!-- 浏览按钮由父（编排者）填充，守「子组件不直接调 service」 -->
          <slot name="saveDirBrowse" />
        </div>
      </div>
      <div class="space-y-1.5">
        <Label class="text-xs text-muted-foreground">预设（初值）</Label>
        <Select
          :model-value="props.batchPresetId"
          @update:model-value="(v: string) => emit('update:preset', v)"
        >
          <SelectTrigger class="h-9 text-sm">
            <SelectValue placeholder="不使用预设" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__none__">不使用预设</SelectItem>
            <SelectItem v-for="p in presets" :key="p.id" :value="p.id">
              {{ p.name }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="flex items-end justify-between">
        <Label class="text-xs text-muted-foreground">自动开始</Label>
        <Switch
          :checked="props.batch.autoStart"
          @update:checked="(v: boolean) => patchBatch({ autoStart: v })"
        />
      </div>
    </div>

    <!-- 行清单 -->
    <div v-if="props.links.length > 1" class="space-y-2">
      <div
        v-for="row in props.links"
        :key="row.id"
        class="flex cursor-pointer items-center gap-3 rounded-lg border px-3 py-2 transition-colors hover:border-primary"
        @click="emit('select', row.id)"
      >
        <AppIcon name="FileVideo" :size="16" class="shrink-0 text-muted-foreground" />
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium">
            {{ row.fileName || row.url }}
          </div>
          <div class="truncate text-xs text-muted-foreground">{{ row.url }}</div>
        </div>
        <span
          v-if="row.detectedType"
          class="rounded-full bg-primary/20 px-2 py-0.5 text-xs font-medium text-primary"
          >{{ row.detectedType.toUpperCase() }}</span
        >
        <span :class="['text-xs', statusColor[row.status]]">{{
          statusLabel[row.status]
        }}</span>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2"
          @click.stop="emit('remove', row.id)"
        >
          <AppIcon name="X" :size="14" />
        </Button>
      </div>
    </div>

    <!-- 提交 -->
    <div class="flex justify-end border-t pt-3">
      <Button :disabled="props.links.length === 0" @click="emit('commit')">
        <AppIcon name="Download" :size="16" class="mr-2" />
        全部添加
      </Button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: type-check**

Run: `npm run type-check`
Expected: 无错误。`text-accent-error` 若非项目既有类，改为内联 `style` 或现有错误色类。

- [ ] **Step 3: 提交**

```bash
git add src/components/task/TaskStagingList.vue
git commit -m "feat(task): TaskStagingList L1 总览（粘贴+批次默认+行清单）

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: `AddTaskDialog` 重写为编排外壳

**Files:**
- Modify: `src/components/task/AddTaskDialog.vue`（整文件替换）

**Interfaces:**
- Consumes: 全部前述产出。`useDownloader`（`addAndStartTask`/`parseUrl`/`isParsing`）、`usePresetManager`（`applyPreset`）、`useSettingsStore`（`defaultSaveDir`/`autoStartDownload`）、`useTaskStore`（`addTask`/`checkUrlExists`）、`systemService`（`selectDirectory`）、`UrlDuplicateDialog`、`resolveLinkToTask`/`seedPresetOverrides`、`LinkConfigPanel`/`TaskStagingList`、`detectUrlType`/`isStreamingType`/`extractFileName`/`generateId`。

- [ ] **Step 1: 整文件替换 `AddTaskDialog.vue`**

```vue
<!-- src/components/task/AddTaskDialog.vue -->
<script setup lang="ts">
/**
 * AddTaskDialog —— 主从详情式暂存层编排外壳（重写）。
 *
 * L1 总览（TaskStagingList）：粘贴 + 批次默认 + 行清单。
 * L2 聚焦（LinkConfigPanel）：单条引擎类型驱动配置 + 内联流选择。
 * 单链接（len==1）：直接进 L2，零跳转。
 * 提交：resolveLinkToTask 三层合并 → addAndStartTask / taskStore.addTask。
 * 后端契约零改动。
 */
import { ref, computed, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useToast, useDownloader, usePresetManager } from "@/composables";
import { useSettingsStore, useTaskStore } from "@/stores";
import { systemService } from "@/services";
import { detectUrlType, isStreamingType } from "@/domain/url";
import { extractFileName } from "@/utils/format";
import { generateId } from "@/utils/id";
import type { DownloadTask, StreamSelection, TaskOverrides } from "@/domain";
import TaskStagingList from "./TaskStagingList.vue";
import LinkConfigPanel from "./LinkConfigPanel.vue";
import { resolveLinkToTask, seedPresetOverrides } from "./resolveLinkToTask";
import type { BatchDefaults, StagedLink } from "./staging-types";

interface Props {
  open: boolean;
}
const props = defineProps<Props>();
const emit = defineEmits<{ (e: "update:open", value: boolean): void }>();

const toast = useToast();
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();
const { addAndStartTask, parseUrl, isParsing } = useDownloader();
const { applyPreset } = usePresetManager();

const isOpen = computed({
  get: () => props.open,
  set: (v) => emit("update:open", v),
});

// ===== 状态 =====
const staged = ref<StagedLink[]>([]);
const view = ref<"list" | "focus">("list");
const selectedId = ref<string | null>(null);
const isSubmitting = ref(false);

const batch = ref<BatchDefaults>({ saveDir: "", autoStart: false });
const batchPresetId = ref<string>("__none__");

// URL 重复
const showUrlDuplicateDialog = ref(false);
const duplicateTask = ref<DownloadTask | null>(null);
const pendingResume = ref<(() => Promise<void>) | null>(null);

const isSingle = computed(() => staged.value.length === 1);
const selectedLink = computed(
  () => staged.value.find((l) => l.id === selectedId.value) ?? null,
);
const canCommit = computed(
  () => staged.value.some((l) => l.status !== "invalid") && !isSubmitting.value,
);
const globalSaveDir = computed(() => settingsStore.defaultSaveDir);
const saveDirPlaceholder = computed(() => {
  const b = batch.value.saveDir.trim();
  const g = globalSaveDir.value;
  if (b) return `将使用批次默认：${b}`;
  if (g) return `将使用全局默认：${g}`;
  return "使用全局默认";
});

// ===== 生命周期 =====
watch(isOpen, async (open) => {
  if (open) {
    batch.value = {
      saveDir: "",
      autoStart: settingsStore.autoStartDownload,
    };
    batchPresetId.value = "__none__";
    staged.value = [];
    view.value = "list";
    selectedId.value = null;
    await nextTick();
  }
});

const reset = () => {
  staged.value = [];
  view.value = "list";
  selectedId.value = null;
  isSubmitting.value = false;
  batch.value = { saveDir: "", autoStart: false };
  batchPresetId.value = "__none__";
  showUrlDuplicateDialog.value = false;
  duplicateTask.value = null;
  pendingResume.value = null;
};

// ===== 粘贴 → 构造 StagedLink[]（编排者持有构造逻辑） =====
function buildLinks(text: string): StagedLink[] {
  const presetOv =
    batchPresetId.value !== "__none__"
      ? applyPreset(batchPresetId.value)
      : null;
  const lines = text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.startsWith("http://") || l.startsWith("https://"));
  return lines.map((url) => {
    const detectedType = detectUrlType(url);
    const streaming = isStreamingType(detectedType);
    return {
      id: generateId(),
      url,
      detectedType,
      fileName: extractFileName(url),
      saveDir: "",
      overrides: seedPresetOverrides(presetOv, detectedType),
      status: streaming ? "pending" : "ready",
    };
  });
}

function handlePaste(text: string) {
  const links = buildLinks(text);
  if (links.length === 0) return;
  // 合并（保留已存在的，追加新解析的）
  staged.value = [...staged.value, ...links];
  if (isSingle.value) {
    selectedId.value = staged.value[0]!.id;
    view.value = "focus";
    void maybeAutoParse();
  } else {
    view.value = "list";
  }
}

// 单链接流媒体：进入聚焦即自动解析一次
async function maybeAutoParse() {
  if (!isSingle.value || !selectedLink.value) return;
  const link = selectedLink.value;
  if (
    link.detectedType &&
    isStreamingType(link.detectedType) &&
    !link.streamInfo
  ) {
    const info = await parseUrl(link.url);
    if (info) {
      link.streamInfo = info;
      link.status = "parsed";
    }
  }
}

// ===== 批次预设变更：重播种未触碰的流媒体行 =====
function handlePresetChange(presetId: string) {
  batchPresetId.value = presetId;
  const presetOv = presetId !== "__none__" ? applyPreset(presetId) : null;
  for (const link of staged.value) {
    if (link.status === "pending" && link.detectedType && isStreamingType(link.detectedType)) {
      link.overrides = seedPresetOverrides(presetOv, link.detectedType);
    }
  }
}

// ===== 选择/导航 =====
function handleSelect(id: string) {
  selectedId.value = id;
  view.value = "focus";
}
function handleFocusDone() {
  if (selectedLink.value && selectedLink.value.status !== "invalid") {
    selectedLink.value.status = "ready";
  }
  view.value = isSingle.value ? "focus" : "list";
}
function handleRemove(id: string) {
  staged.value = staged.value.filter((l) => l.id !== id);
  if (selectedId.value === id) selectedId.value = null;
  if (isSingle.value && staged.value[0]) {
    selectedId.value = staged.value[0]!.id;
  }
}

// 保存目录浏览（经 systemService，仅编排者知道 service）
async function handleBrowseSaveDir() {
  const dir = await systemService.selectDirectory();
  if (dir) batch.value.saveDir = dir;
}

// ===== 提交 =====
async function handleCommit() {
  if (isSubmitting.value || !canCommit.value) return;
  isSubmitting.value = true;
  const links = staged.value.filter((l) => l.status !== "invalid");
  await runSubmit(links, 0);
}

async function runSubmit(links: StagedLink[], from: number) {
  let success = 0;
  for (let i = from; i < links.length; i++) {
    const link = links[i]!;
    // URL 重复检测
    const existing = taskStore.checkUrlExists(link.url);
    if (existing) {
      duplicateTask.value = existing;
      showUrlDuplicateDialog.value = true;
      // 暂停，等待用户确认后从 i 继续（强制跳过检查）
      pendingResume.value = async () => {
        await addOne(link, true);
        success++;
        await runSubmit(links, i + 1);
      };
      return; // 暂停
    }
    try {
      await addOne(link, false);
      success++;
    } catch {
      // 逐条失败不阻塞
    }
  }
  if (success > 0) {
    toast.success(`已添加 ${success} 个任务`);
  }
  isSubmitting.value = false;
  handleClose();
}

async function addOne(link: StagedLink, skipUrlCheck: boolean) {
  const resolved = resolveLinkToTask(link, batch.value, globalSaveDir.value);
  if (batch.value.autoStart && !resolved.hasSchedule) {
    await addAndStartTask(
      resolved.url,
      resolved.fileName,
      resolved.saveDir,
      resolved.overrides,
    );
  } else {
    await taskStore.addTask({
      url: resolved.url,
      fileName: resolved.fileName,
      saveDir: resolved.saveDir,
      overrides: resolved.overrides,
      skipUrlCheck,
    });
  }
}

async function handleUrlDuplicateConfirm() {
  showUrlDuplicateDialog.value = false;
  const resume = pendingResume.value;
  pendingResume.value = null;
  if (resume) await resume();
}

function handleUrlDuplicateCancel() {
  showUrlDuplicateDialog.value = false;
  pendingResume.value = null;
  isSubmitting.value = false;
  toast.warning("已取消，部分任务未添加");
}

// ===== 关闭 =====
function handleClose() {
  reset();
  isOpen.value = false;
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="flex max-h-[85vh] max-w-[min(640px,calc(100vw-2rem))] flex-col"
      @close-auto-focus="reset"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
        </DialogTitle>
        <DialogDescription class="sr-only">
          粘贴链接，逐条配置后添加下载任务
        </DialogDescription>
      </DialogHeader>

      <div class="-mx-2 flex-1 space-y-4 overflow-y-auto px-2">
        <!-- L1 总览：单链接时也显示（便于继续粘贴追加） -->
        <TaskStagingList
          v-if="view === 'list' || isSingle"
          :links="staged"
          :batch="batch"
          :batch-preset-id="batchPresetId"
          :global-save-dir="globalSaveDir"
          @update:batch="(b) => (batch = b)"
          @update:preset="handlePresetChange"
          @paste="handlePaste"
          @select="handleSelect"
          @remove="handleRemove"
          @commit="handleCommit"
        >
          <template #saveDirBrowse>
            <Button
              variant="outline"
              size="sm"
              class="h-9 px-3"
              @click="handleBrowseSaveDir"
            >
              <AppIcon name="FolderOpen" :size="14" />
            </Button>
          </template>
        </TaskStagingList>

        <!-- L2 聚焦 -->
        <div v-if="view === 'focus' && selectedLink" class="space-y-3">
          <button
            v-if="!isSingle"
            class="flex cursor-pointer items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
            @click="view = 'list'"
          >
            <AppIcon name="ChevronLeft" :size="14" />
            返回列表
          </button>
          <LinkConfigPanel
            :model-value="selectedLink"
            :save-dir-placeholder="saveDirPlaceholder"
            @done="handleFocusDone"
          />
        </div>
      </div>

      <!-- URL 重复确认 -->
      <UrlDuplicateDialog
        v-model:open="showUrlDuplicateDialog"
        :existing-task="duplicateTask"
        @confirm="handleUrlDuplicateConfirm"
        @cancel="handleUrlDuplicateCancel"
      />
    </DialogContent>
  </Dialog>
</template>
```

> **设计说明（非占位，已定型）：** `LinkConfigPanel` 用 `defineModel` 接收 `StagedLink` 引用，深层字段（`link.value.overrides.maxSpeed` 等）就地突变——由于传入的是 `staged` 数组中同一响应式对象引用，突变直接生效，**无需 `@update:model-value`**，故父用单向 `:model-value="selectedLink"`（computed 只读）即可。浏览目录按钮经 `#saveDirBrowse` 插槽由编排者填充（Task 5 在保存位置行预留该插槽），守住「子组件不直接调 service」。

- [ ] **Step 2: type-check**

Run: `npm run type-check`
Expected: 无错误

- [ ] **Step 3: lint**

Run: `npm run lint`
Expected: 无错误（自动修复后）

- [ ] **Step 4: 手动 dev 验证**

Run: `npm run tauri dev`
验证清单：
- 单链接直链 `.mp4`：粘贴→直接进 L2，只露通用三件（文件名/保存位置/定时），无流媒体选项→「全部添加」可建任务。
- 单链接 `.m3u8`：粘贴→进 L2→自动解析→内联流选择→选流→完成→添加。
- 多链接：粘贴→L1 行清单→点行进 L2 配置→返回→行变就绪→「全部添加」批量建。
- 批次保存位置：留空→单行也留空→任务落到全局默认目录。
- 批次预设：选预设后粘贴→流媒体行 overrides 带初值；改预设→仅 pending 行重播种。
- URL 重复：粘已有 URL→弹 UrlDuplicateDialog→确认继续→剩余条继续提交。
- 拖拽文本：拖入多行→并入清单。

- [ ] **Step 5: 提交**

```bash
git add src/components/task/AddTaskDialog.vue
git commit -m "refactor(task): AddTaskDialog 重写为主从详情式暂存编排外壳

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: 状态追踪更新 + 设计文档同步

**Files:**
- Modify: `docs/design/06-feature-status.md`
- Modify: `docs/superpowers/specs/2026-08-01-add-task-staging-design.md`（5.5 提交语义微调：invalid 才跳过，其余皆提交）

**Interfaces:**
- 无代码接口；文档同步。

- [ ] **Step 1: 更新功能状态**

打开 `docs/design/06-feature-status.md`，把「添加任务」相关行更新为暂存层交互，并新增「多链接逐条配置」一行（符号 `[x]`，文件指向新组件）。

- [ ] **Step 2: 同步设计文档 5.5**

把 `2026-08-01-add-task-staging-design.md` 的 5.5「就绪门槛」段改为：
> 提交门槛：仅 `invalid` 行被跳过；`pending`/`parsed`/`ready` 均参与提交（未配置者用默认值）。`ready` 是「已查看」指示而非门槛——「全部添加」不强制逐条打开，逐条配置是机会而非必经。

- [ ] **Step 3: 提交**

```bash
git add docs/design/06-feature-status.md docs/superpowers/specs/2026-08-01-add-task-staging-design.md
git commit -m "docs: 同步暂存层功能状态与提交语义

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Self-Review（撰写后自检）

**1. Spec 覆盖：**
- 第 2 节引擎能力事实 → 驱动 Task 1 可见性表 ✓
- 第 3 节主从二态 + 单链接零跳转 → Task 6 `isSingle`/`view` 状态机 ✓
- 第 4 节可见性表 + 数据模型 + 高内聚低耦合 → Task 1/2 纯函数 + Task 4/5 职责边界 ✓
- 第 5 节交互流程（解析时机/自动 vs 手动/流选择内联/提交语义/边界） → Task 3 内联化 + Task 6 提交循环 ✓
- 第 6 节三层配置归属 + 渐进披露重校准 + 删除项 → Task 6 删除旧 ref、Task 7 文档 ✓
- 内联化（用户强调不可省） → Task 3 ✓

**2. 类型一致性：** `StagedLink`/`BatchDefaults`/`LinkOption` 全程同名；`resolveLinkToTask`/`seedPresetOverrides`/`cleanOverrides` 签名一致；`StreamPickerInline` props/emits 在 Task 3 定义、Task 4 消费一致。Select 的 union 类型（`MuxFormat`/`SubtitleFormat`）用显式 `@update:model-value` + 内联 `as` 转换处理，已写定。

**3. 已无占位/注记：** 删除了 Task 4 行内预设（违设计 4.3）、`StagedLinkImport` 别名、`as LinkOption` 冗余断言、`handleBrowse` 死代码、Task 6「实现注记」块。`LinkConfigPanel` 经 `:model-value` 单向传 `StagedLink` 引用、子组件深层就地突变（同一响应式对象），无需 `@update:model-value`——此为定型设计，非待办。`#saveDirBrowse` 插槽已在 Task 5 保存位置行预留 `<slot name="saveDirBrowse" />`，与 Task 6 编排者填充对接。剩余仅 `AppIcon` 图标名（`Search`/`Check`/`FileVideo`/`X`/`ChevronLeft`）需在 dev/type-check 时按现有图标库对齐——属正常实现校验，非规格缺口。
