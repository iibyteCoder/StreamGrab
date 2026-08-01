# 添加任务弹窗重设计 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把"主从详情式暂存外壳"的添加任务弹窗重写为三段式向导（粘贴 → 解析 → 逐条配置卡），路径记忆 + 高级项折叠，单/多链接同构，按层解耦。

**Architecture:** 四层单向依赖——UI 组件（`AddTaskDialog` 薄壳 + `LinkConfigCard` + `LinkAdvancedSection`）→ 编排 composable（`useAddTaskWizard` + `useRecentDirs`）→ 纯逻辑（`parseLinks` + `resolveLinkToTask`）→ 既有服务/状态层（`taskStore` / `useDownloader` / `downloadService` / `systemService`）。纯函数可单测；composable 与组件经 type-check + 手动运行验证（本项目无组件测试设施，沿用"只单测纯逻辑"的既有约定）。

**Tech Stack:** Vue 3 `<script setup>` + TS、Pinia、VueUse `useStorage`、Vitest、reka-ui/shadcn-vue 既有组件。

**Spec:** `docs/superpowers/specs/2026-08-01-add-task-dialog-redesign-design.md`

**关键契约（已核对，勿改）：**
- `taskStore.addTask({url, fileName?, saveDir?, overrides?, skipUrlCheck?}): Promise<{task, wasRenamed}>`
- `taskStore.checkUrlExists(url): TaskRecord | undefined`
- `useDownloader().addAndStartTask(url, fileName?, saveDir?, overrides?): Promise<{task, wasRenamed}>`（内部已读 `auto_start_download` 决定是否 `processQueue`）
- `downloadService.parseUrl(url): Promise<StreamInfo>`（失败 reject；**不**经 `useDownloader.parseUrl`，避免批量逐条弹 toast）
- `settingsStore.defaultSaveDir` / `settingsStore.autoStartDownload`（computed）
- `systemService.selectDirectory(): Promise<string | null>`
- `useToast().success/error/warning/info(message)`
- `UrlDuplicateDialog` props `{open, existingTask}` emits `update:open/confirm/cancel`
- `StreamPickerInline` props `{streamInfo, loading?}` emits `confirm(selection)/cancel`
- `detectUrlType(url): UrlType`、`isStreamingType(t): boolean`（`@/domain/url`）；`extractFileName(url)`（`@/utils/format`）；`generateId()`（`@/utils/id`）

**设计决策（实现时遵守）：**
1. `resolveLinkToTask` 第二参是**有效默认目录** = `useRecentDirs.defaultDir`（最近记忆 > 全局），非裸 `settingsStore.defaultSaveDir`。
2. `maxSpeed/customRange` 沿用现有 `linkOptionVisibility` 归为流媒体专属（引擎能力决定）；HTTP 直链高级区实际只显示"定时开始"。spec §8.1 表中将限速/范围列入直链为**预期差异**，本次不改引擎可见性，留待后续。
3. 高级区折叠用 disclosure 按钮 + `v-if`（无 Accordion 组件，沿用现状）。
4. 解析用 `downloadService.parseUrl` 直连（编排层允许触达服务），集中错误处理。

---

## File Structure

| 文件 | 动作 | 职责 |
|---|---|---|
| `src/components/task/addTaskTypes.ts` | 新增 | `WizardStep` / `ParsedLink` / `StagedLink` / `LinkOption` 类型 + `URL_TYPE_BADGE` 集中映射 |
| `src/components/task/parseLinks.ts` | 新增 | 纯函数：粘贴文本 → 分类链接，剔除 unknown |
| `src/components/task/parseLinks.test.ts` | 新增 | parseLinks 单测 |
| `src/components/task/resolveLinkToTask.ts` | 重写 | 两层映射；删 `seedPresetOverrides`/`BatchDefaults` 依赖 |
| `src/components/task/resolveLinkToTask.test.ts` | 重写 | 两层语义单测 |
| `src/components/task/linkOptionVisibility.ts` | 改 import | `LinkOption` 改自 `./addTaskTypes` |
| `src/composables/useRecentDirs.ts` | 新增 | `useStorage` 最近目录记忆 + 纯 helper `rememberDir`/`resolveDefaultDir` |
| `src/composables/useRecentDirs.test.ts` | 新增 | 纯 helper 单测 |
| `src/composables/useAddTaskWizard.ts` | 新增 | 状态机 + 导航 + 提交调度（编排层） |
| `src/composables/index.ts` | 改 | 导出 `useAddTaskWizard` / `useRecentDirs` |
| `src/components/task/LinkAdvancedSection.vue` | 新增 | 引擎驱动高级项 + 内联流选择 + 解析重试 |
| `src/components/task/LinkConfigCard.vue` | 新增 | L1 字段 + 记忆下拉 + 高级手风琴 |
| `src/components/task/AddTaskDialog.vue` | 重写 | 三段式薄壳 |
| `src/components/task/TaskStagingList.vue` | 删除 | 批次概念移除 |
| `src/components/task/LinkConfigPanel.vue` | 删除 | 拆分为 Card + AdvancedSection |
| `src/components/task/staging-types.ts` | 删除 | 被 `addTaskTypes.ts` 取代 |

---

### Task 1: 领域类型与集中映射 `addTaskTypes.ts`

**Files:**
- Create: `src/components/task/addTaskTypes.ts`

- [ ] **Step 1: 创建类型文件**

```ts
import type { StreamInfo, TaskOverrides, UrlType } from "@/domain";

/** 向导步骤 */
export type WizardStep = "paste" | "parsing" | "config" | "done";

/** 高级选项键（供 linkOptionVisibility 使用） */
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

/** 纯解析产出（无 id、无 overrides —— 由向导装配为 StagedLink） */
export interface ParsedLink {
  url: string;
  detectedType: UrlType;
  fileName: string;
  streaming: boolean;
}

/** 向导内单条待配置链接（仅前端暂存，不进领域/后端） */
export interface StagedLink {
  id: string;
  url: string;
  detectedType: UrlType;
  fileName: string;
  saveDir: string;
  overrides: TaskOverrides;
  streamInfo?: StreamInfo;
  /** 流媒体解析失败（失败 ≠ 无效；无效在解析阶段已剔除） */
  parseFailed: boolean;
}

/** URL 类型 → 徽章文案（集中一处，消除分散） */
export const URL_TYPE_BADGE: Record<UrlType, string> = {
  hls: "HLS",
  dash: "DASH",
  mss: "MSS",
  httpVideo: "直链视频",
  unknown: "未知",
};

export function typeBadgeLabel(t: UrlType): string {
  return URL_TYPE_BADGE[t];
}
```

- [ ] **Step 2: 验证类型成立**

Run: `npm run type-check`
Expected: 通过（此时 `UrlType` 联合须恰好含 hls/dash/mss/httpVideo/unknown；若 `Record<UrlType>` 报缺键/多键，按编译器提示对齐键集）。

- [ ] **Step 3: Commit**

```bash
git add src/components/task/addTaskTypes.ts
git commit -m "feat: 添加任务向导领域类型与类型徽章集中映射"
```

---

### Task 2: 纯解析 `parseLinks.ts`（TDD）

**Files:**
- Create: `src/components/task/parseLinks.ts`
- Create: `src/components/task/parseLinks.test.ts`

- [ ] **Step 1: 先写失败测试**

```ts
import { describe, it, expect } from "vitest";
import { extractLinks, classifyLink, parsePastedText } from "./parseLinks";

describe("extractLinks", () => {
  it("trim 并只保留 http(s) 行", () => {
    expect(extractLinks("  https://a/1.m3u8  \nftp://x\nhttp://b/2.mp4\nnotaurl")).toEqual([
      "https://a/1.m3u8",
      "http://b/2.mp4",
    ]);
  });
  it("按出现顺序去重", () => {
    expect(extractLinks("https://a/1\nhttps://a/1\nhttps://a/2")).toEqual([
      "https://a/1",
      "https://a/2",
    ]);
  });
  it("空文本返回空数组", () => {
    expect(extractLinks("  \n  ")).toEqual([]);
  });
});

describe("classifyLink", () => {
  it("hls 标记 streaming", () => {
    const r = classifyLink("https://a/x.m3u8");
    expect(r.detectedType).toBe("hls");
    expect(r.streaming).toBe(true);
  });
  it("mp4 直链非 streaming，且提取文件名", () => {
    const r = classifyLink("https://a/dir/movie.mp4");
    expect(r.detectedType).toBe("httpVideo");
    expect(r.streaming).toBe(false);
    expect(r.fileName).toBe("movie");
  });
});

describe("parsePastedText", () => {
  it("剔除 unknown 并计数，保留有效链接", () => {
    const { links, skipped } = parsePastedText(
      "https://a/x.m3u8\nhttps://a/page.html\nhttps://a/y.mp4",
    );
    expect(skipped).toBe(1);
    expect(links.map((l) => l.url)).toEqual(["https://a/x.m3u8", "https://a/y.mp4"]);
  });
  it("全部无效时 links 为空", () => {
    const { links, skipped } = parsePastedText("https://a/page.html");
    expect(links).toEqual([]);
    expect(skipped).toBe(1);
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/parseLinks.test.ts`
Expected: FAIL — `Cannot find module './parseLinks'`。

- [ ] **Step 3: 实现 parseLinks**

```ts
import { detectUrlType, isStreamingType } from "@/domain/url";
import { extractFileName } from "@/utils/format";
import type { ParsedLink } from "./addTaskTypes";

/** 从粘贴文本提取合法链接行：trim、过滤 http(s)、按出现顺序去重 */
export function extractLinks(text: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of text.split("\n")) {
    const url = raw.trim();
    if (!(url.startsWith("http://") || url.startsWith("https://"))) continue;
    if (seen.has(url)) continue;
    seen.add(url);
    out.push(url);
  }
  return out;
}

/** 分类单条链接（同步，纯本地检测） */
export function classifyLink(url: string): ParsedLink {
  const detectedType = detectUrlType(url);
  return {
    url,
    detectedType,
    fileName: extractFileName(url),
    streaming: isStreamingType(detectedType),
  };
}

/** 解析粘贴文本：分类 + 剔除无法识别(unknown)，返回有效链接与跳过数 */
export function parsePastedText(text: string): {
  links: ParsedLink[];
  skipped: number;
} {
  const links: ParsedLink[] = [];
  let skipped = 0;
  for (const url of extractLinks(text)) {
    const link = classifyLink(url);
    if (link.detectedType === "unknown") {
      skipped++;
      continue;
    }
    links.push(link);
  }
  return { links, skipped };
}
```

> 注：`https://a/page.html` 无视频扩展名 → `detectUrlType` 返回 `unknown`，符合剔除预期。若 `extractFileName("https://a/dir/movie.mp4")` 返回含扩展名，则把测试断言改为实际返回（以 `utils/format.ts` 为准）。

- [ ] **Step 4: 运行测试确认通过**

Run: `npx vitest run src/components/task/parseLinks.test.ts`
Expected: PASS（5 个用例）。

- [ ] **Step 5: Commit**

```bash
git add src/components/task/parseLinks.ts src/components/task/parseLinks.test.ts
git commit -m "feat: 粘贴链接纯解析（分类+剔除无效）及单测"
```

---

### Task 3: 两层映射 `resolveLinkToTask.ts`（重写 + TDD）

**Files:**
- Modify: `src/components/task/resolveLinkToTask.ts`
- Modify: `src/components/task/resolveLinkToTask.test.ts`

- [ ] **Step 1: 先改测试为两层语义**

```ts
import { describe, it, expect } from "vitest";
import { cleanOverrides, resolveLinkToTask } from "./resolveLinkToTask";
import type { StagedLink } from "./addTaskTypes";
import type { TaskOverrides } from "@/domain";

function mkLink(over: Partial<TaskOverrides> = {}, saveDir = ""): StagedLink {
  return {
    id: "1",
    url: "https://x/a.m3u8",
    detectedType: "hls",
    fileName: "a",
    saveDir,
    overrides: over as TaskOverrides,
    parseFailed: false,
  };
}

describe("cleanOverrides", () => {
  it("剔除空字段，全空返回 undefined", () => {
    expect(cleanOverrides({} as TaskOverrides)).toBeUndefined();
    expect(cleanOverrides({ maxSpeed: "" } as TaskOverrides)).toBeUndefined();
  });
  it("保留非空字段", () => {
    expect(cleanOverrides({ maxSpeed: "5M" } as TaskOverrides)?.maxSpeed).toBe("5M");
  });
});

describe("resolveLinkToTask（两层：逐条 > 默认）", () => {
  it("saveDir：行内非空优先，否则用默认目录", () => {
    expect(resolveLinkToTask(mkLink({}, ""), "D:/default").saveDir).toBe("D:/default");
    expect(resolveLinkToTask(mkLink({}, "D:/row"), "D:/default").saveDir).toBe("D:/row");
    expect(resolveLinkToTask(mkLink({}, "  "), "D:/default").saveDir).toBe("D:/default");
  });
  it("两者皆空 → saveDir undefined", () => {
    expect(resolveLinkToTask(mkLink({}, ""), "").saveDir).toBeUndefined();
  });
  it("空 overrides → undefined", () => {
    expect(resolveLinkToTask(mkLink(), "D:/default").overrides).toBeUndefined();
  });
  it("hasSchedule 由 scheduledStartAt 决定", () => {
    expect(
      resolveLinkToTask(mkLink({ scheduledStartAt: "2026-01-01T00:00:00" }), "D:/").hasSchedule,
    ).toBe(true);
    expect(resolveLinkToTask(mkLink(), "D:/").hasSchedule).toBe(false);
  });
  it("fileName 空白 → undefined", () => {
    expect(resolveLinkToTask({ ...mkLink(), fileName: "  " }, "D:/").fileName).toBeUndefined();
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/components/task/resolveLinkToTask.test.ts`
Expected: FAIL（签名不匹配 / 仍引用 staging-types）。

- [ ] **Step 3: 重写实现**

```ts
import type { TaskOverrides } from "@/domain";
import type { StagedLink } from "./addTaskTypes";

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
export function cleanOverrides(overrides: TaskOverrides): TaskOverrides | undefined {
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
 * 两层合并「逐条配置 > 有效默认目录」，产出可直接建任务的规格。
 * fallbackSaveDir 传 useRecentDirs.defaultDir（最近记忆 > 全局默认）。
 */
export function resolveLinkToTask(link: StagedLink, fallbackSaveDir: string): ResolvedTask {
  const saveDir = firstNonEmpty(link.saveDir, fallbackSaveDir);
  const fileName = link.fileName.trim() || undefined;
  const overrides = cleanOverrides(link.overrides);
  const hasSchedule = !!overrides?.scheduledStartAt;
  return { url: link.url, fileName, saveDir, overrides, hasSchedule };
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npx vitest run src/components/task/resolveLinkToTask.test.ts`
Expected: PASS（7 个用例）。

- [ ] **Step 5: Commit**

```bash
git add src/components/task/resolveLinkToTask.ts src/components/task/resolveLinkToTask.test.ts
git commit -m "refactor: resolveLinkToTask 降为两层映射，移除预设播种"
```

---

### Task 4: `linkOptionVisibility.ts` 迁移 import

**Files:**
- Modify: `src/components/task/linkOptionVisibility.ts`

- [ ] **Step 1: 改 import 来源**

将第 3 行：

```ts
import type { LinkOption } from "./staging-types";
```

改为：

```ts
import type { LinkOption } from "./addTaskTypes";
```

其余逻辑（`STREAMING_ONLY` 集合与 `isOptionVisible`）保持不变。

- [ ] **Step 2: 验证**

Run: `npm run type-check`
Expected: 通过（`staging-types` 此时尚存，后续 Task 9 删除）。

- [ ] **Step 3: Commit**

```bash
git add src/components/task/linkOptionVisibility.ts
git commit -m "refactor: linkOptionVisibility 的 LinkOption 改自 addTaskTypes"
```

---

### Task 5: 路径记忆 `useRecentDirs.ts`（TDD 纯 helper）

**Files:**
- Create: `src/composables/useRecentDirs.ts`
- Create: `src/composables/useRecentDirs.test.ts`

- [ ] **Step 1: 先写失败测试（只测纯 helper，node 环境无需 localStorage）**

```ts
import { describe, it, expect } from "vitest";
import { rememberDir, resolveDefaultDir } from "./useRecentDirs";

describe("rememberDir", () => {
  it("新目录置最前", () => {
    expect(rememberDir(["A", "B"], "C")).toEqual(["C", "A", "B"]);
  });
  it("已存在则提到最前去重", () => {
    expect(rememberDir(["A", "B", "C"], "B")).toEqual(["B", "A", "C"]);
  });
  it("截断到上限 5", () => {
    expect(rememberDir(["1", "2", "3", "4", "5"], "6")).toEqual(["6", "1", "2", "3", "4"]);
  });
  it("空白目录忽略，原样返回", () => {
    const list = ["A"];
    expect(rememberDir(list, "   ")).toEqual(["A"]);
  });
  it("trim 后写入", () => {
    expect(rememberDir([], "  D:/x  ")).toEqual(["D:/x"]);
  });
});

describe("resolveDefaultDir", () => {
  it("最近记忆优先", () => {
    expect(resolveDefaultDir(["D:/recent", "D:/old"], "D:/global")).toBe("D:/recent");
  });
  it("无记忆回退全局", () => {
    expect(resolveDefaultDir([], "D:/global")).toBe("D:/global");
  });
  it("皆无返回空串", () => {
    expect(resolveDefaultDir([], "  ")).toBe("");
  });
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `npx vitest run src/composables/useRecentDirs.test.ts`
Expected: FAIL — `Cannot find module './useRecentDirs'`。

- [ ] **Step 3: 实现**

```ts
import { computed } from "vue";
import { useStorage } from "@vueuse/core";
import { useSettingsStore } from "@/stores";

const STORAGE_KEY = "streamgrab:recentSaveDirs";
const MAX_RECENT = 5;

/** 纯函数：把 dir 记到最前，去重、去空、截断到 MAX_RECENT */
export function rememberDir(list: string[], dir: string): string[] {
  const t = dir.trim();
  if (!t) return list;
  return [t, ...list.filter((d) => d !== t)].slice(0, MAX_RECENT);
}

/** 纯函数：有效默认目录 = 最近记忆 > 全局默认 > 空 */
export function resolveDefaultDir(recent: string[], global: string): string {
  return (recent[0] ?? "").trim() || global.trim() || "";
}

/** 最近保存目录记忆（localStorage，MRU-first，上限 5） */
export function useRecentDirs() {
  const settingsStore = useSettingsStore();
  const dirs = useStorage<string[]>(STORAGE_KEY, []);

  const defaultDir = computed(() => resolveDefaultDir(dirs.value, settingsStore.defaultSaveDir));

  function remember(dir: string): void {
    dirs.value = rememberDir(dirs.value, dir);
  }

  return { dirs, defaultDir, remember };
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `npx vitest run src/composables/useRecentDirs.test.ts`
Expected: PASS（8 个用例）。

- [ ] **Step 5: Commit**

```bash
git add src/composables/useRecentDirs.ts src/composables/useRecentDirs.test.ts
git commit -m "feat: 最近保存目录记忆（useStorage + 纯 helper 单测）"
```

---

### Task 6: 编排层 `useAddTaskWizard.ts`

**Files:**
- Create: `src/composables/useAddTaskWizard.ts`

> 本 composable 依赖 store/service/toast，按项目既有约定不做单测（同 `useDownloader`），由 Task 10 type-check + 手动运行验证。

- [ ] **Step 1: 实现向导编排**

```ts
import { ref, computed } from "vue";
import { useTaskStore } from "@/stores";
import { useDownloader, useToast } from "@/composables";
import { useRecentDirs } from "./useRecentDirs";
import { downloadService } from "@/services/downloadService";
import { systemService } from "@/services/systemService";
import { parsePastedText } from "@/components/task/parseLinks";
import { resolveLinkToTask } from "@/components/task/resolveLinkToTask";
import { generateId } from "@/utils/id";
import { isStreamingType } from "@/domain/url";
import type { DownloadTask } from "@/domain";
import type { StagedLink, WizardStep } from "@/components/task/addTaskTypes";

export function useAddTaskWizard() {
  const taskStore = useTaskStore();
  const { addAndStartTask } = useDownloader();
  const { dirs, defaultDir, remember } = useRecentDirs();
  const toast = useToast();

  // ===== 状态 =====
  const step = ref<WizardStep>("paste");
  const links = ref<StagedLink[]>([]);
  const index = ref(0);
  const parseDone = ref(0);
  const parseTotal = ref(0);
  const parsingId = ref<string | null>(null);
  const isSubmitting = ref(false);
  const addedCount = ref(0);
  const showDuplicate = ref(false);
  const duplicateTask = ref<DownloadTask | null>(null);
  let duplicatePending: StagedLink | null = null;

  // ===== 派生 =====
  const total = computed(() => links.value.length);
  const current = computed<StagedLink | null>(() => links.value[index.value] ?? null);
  const isSingle = computed(() => total.value === 1);
  const isLast = computed(() => index.value >= total.value - 1);
  const showAddAll = computed(() => total.value > 1);

  // ===== 生命周期 =====
  function reset(): void {
    step.value = "paste";
    links.value = [];
    index.value = 0;
    parseDone.value = 0;
    parseTotal.value = 0;
    parsingId.value = null;
    isSubmitting.value = false;
    addedCount.value = 0;
    showDuplicate.value = false;
    duplicateTask.value = null;
    duplicatePending = null;
  }

  // ===== 步骤 1 → 2：粘贴 → 解析 =====
  async function submitPaste(text: string): Promise<void> {
    const { links: parsed, skipped } = parsePastedText(text);
    if (skipped > 0) toast.warning(`${skipped} 个链接无法识别已跳过`);
    if (parsed.length === 0) {
      if (skipped === 0) toast.warning("未识别到有效链接");
      return;
    }
    links.value = parsed.map((p) => ({
      id: generateId(),
      url: p.url,
      detectedType: p.detectedType,
      fileName: p.fileName,
      saveDir: "",
      overrides: {},
      parseFailed: false,
    }));
    index.value = 0;

    const streaming = links.value.filter((l) => isStreamingType(l.detectedType));
    if (streaming.length === 0) {
      step.value = "config";
      return;
    }
    step.value = "parsing";
    parseTotal.value = streaming.length;
    parseDone.value = 0;
    await Promise.all(
      streaming.map(async (link) => {
        try {
          link.streamInfo = await downloadService.parseUrl(link.url);
          link.parseFailed = false;
        } catch {
          link.parseFailed = true;
        } finally {
          parseDone.value++;
        }
      }),
    );
    step.value = "config";
  }

  // ===== 配置卡内：重试解析 / 浏览目录 =====
  async function retryParse(link: StagedLink): Promise<void> {
    parsingId.value = link.id;
    try {
      link.streamInfo = await downloadService.parseUrl(link.url);
      link.parseFailed = false;
    } catch {
      link.parseFailed = true;
    } finally {
      parsingId.value = null;
    }
  }

  async function browseSaveDir(): Promise<void> {
    const dir = await systemService.selectDirectory();
    if (dir && current.value) current.value.saveDir = dir;
  }

  // ===== 提交 =====
  async function commitOne(link: StagedLink, fallback: string): Promise<boolean> {
    const resolved = resolveLinkToTask(link, fallback);
    try {
      if (resolved.hasSchedule) {
        await taskStore.addTask({
          url: resolved.url,
          fileName: resolved.fileName,
          saveDir: resolved.saveDir,
          overrides: resolved.overrides,
        });
      } else {
        await addAndStartTask(
          resolved.url,
          resolved.fileName,
          resolved.saveDir,
          resolved.overrides,
        );
      }
      if (resolved.saveDir) remember(resolved.saveDir);
      return true;
    } catch (e) {
      console.error("Failed to add task:", e);
      return false;
    }
  }

  function closeWithSummary(added: number, dupSkipped = 0): void {
    const parts: string[] = [];
    if (added > 0) parts.push(`已添加 ${added} 个任务`);
    if (dupSkipped > 0) parts.push(`跳过 ${dupSkipped} 个重复`);
    if (parts.length) toast.success(parts.join("，"));
    step.value = "done";
  }

  function advance(): void {
    if (isLast.value) closeWithSummary(addedCount.value);
    else index.value++;
  }

  async function commitAndAdvance(link: StagedLink): Promise<void> {
    isSubmitting.value = true;
    try {
      if (await commitOne(link, defaultDir.value)) addedCount.value++;
    } finally {
      isSubmitting.value = false;
    }
    advance();
  }

  /** 逐条"添加/完成"：命中重复 → 弹确认；否则提交并推进 */
  async function addCurrent(): Promise<void> {
    const link = current.value;
    if (!link || isSubmitting.value) return;
    const existing = taskStore.checkUrlExists(link.url);
    if (existing) {
      duplicateTask.value = existing;
      duplicatePending = link;
      showDuplicate.value = true;
      return;
    }
    await commitAndAdvance(link);
  }

  /** 跳过当前条 */
  function skip(): void {
    advance();
  }

  /** 批量：剩余链接按当前卡目录（回退 defaultDir）默认入库，重复静默跳过 */
  async function addAll(): Promise<void> {
    if (isSubmitting.value) return;
    isSubmitting.value = true;
    const batchDir = current.value?.saveDir.trim() || defaultDir.value;
    let added = 0;
    let dupSkipped = 0;
    try {
      for (let i = index.value; i < links.value.length; i++) {
        const link = links.value[i]!;
        if (taskStore.checkUrlExists(link.url)) {
          dupSkipped++;
          continue;
        }
        if (await commitOne(link, batchDir)) added++;
      }
    } finally {
      isSubmitting.value = false;
    }
    closeWithSummary(added, dupSkipped);
  }

  // ===== 重复确认 =====
  async function confirmDuplicate(): Promise<void> {
    showDuplicate.value = false;
    const link = duplicatePending;
    duplicateTask.value = null;
    duplicatePending = null;
    if (link) await commitAndAdvance(link);
  }

  function cancelDuplicate(): void {
    showDuplicate.value = false;
    duplicateTask.value = null;
    duplicatePending = null;
    advance(); // 取消 = 跳过该条
  }

  return {
    // state
    step,
    links,
    index,
    parseDone,
    parseTotal,
    parsingId,
    isSubmitting,
    showDuplicate,
    duplicateTask,
    // derived
    total,
    current,
    isSingle,
    isLast,
    showAddAll,
    dirs,
    defaultDir,
    // actions
    reset,
    submitPaste,
    retryParse,
    browseSaveDir,
    addCurrent,
    skip,
    addAll,
    confirmDuplicate,
    cancelDuplicate,
  };
}
```

- [ ] **Step 2: 验证类型**

Run: `npm run type-check`
Expected: 通过。

- [ ] **Step 3: Commit**

```bash
git add src/composables/useAddTaskWizard.ts
git commit -m "feat: 添加任务向导编排层（状态机+导航+提交调度）"
```

---

### Task 7: 导出 composable

**Files:**
- Modify: `src/composables/index.ts`

- [ ] **Step 1: 追加导出**

在 `src/composables/index.ts` 末尾追加：

```ts
export { useRecentDirs } from "./useRecentDirs";
export { useAddTaskWizard } from "./useAddTaskWizard";
```

- [ ] **Step 2: 验证**

Run: `npm run type-check`
Expected: 通过。

- [ ] **Step 3: Commit**

```bash
git add src/composables/index.ts
git commit -m "chore: 导出 useRecentDirs / useAddTaskWizard"
```

---

### Task 8: 高级设置组件 `LinkAdvancedSection.vue`

**Files:**
- Create: `src/components/task/LinkAdvancedSection.vue`

> 定时开始 + 流选择（含解析/重试）为本组件新写；限速/范围/容器/字幕/仅字幕/密钥六块从现有 `LinkConfigPanel.vue` 第 215–330 行**逐字复制**（字段绑定、`isOptionVisible` 门控、placeholder 均不变）。

- [ ] **Step 1: 创建组件**

```vue
<script setup lang="ts">
/**
 * 高级设置区（L2/L3）：按引擎类型动态渲染。
 * 纯编辑 + 上抛 parse；解析由向导执行（本组件不触达 service）。
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
import { isStreamingType } from "@/domain/url";
import { isOptionVisible } from "./linkOptionVisibility";
import type { StagedLink } from "./addTaskTypes";
import type { MuxFormat, StreamSelection, SubtitleFormat } from "@/domain";

const props = defineProps<{ parsing: boolean }>();
const emit = defineEmits<{ (e: "parse"): void }>();
const link = defineModel<StagedLink>({ required: true });

const isStreaming = computed(() => isStreamingType(link.value.detectedType));
const showStreamPicker = ref(false);

const minScheduleTime = computed(() => {
  const now = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
});
const scheduleTime = computed<string>({
  get: () => link.value.overrides.scheduledStartAt ?? "",
  set: (v: string) => {
    link.value.overrides.scheduledStartAt = v || undefined;
  },
});
function handleStreamConfirm(sel: StreamSelection) {
  link.value.overrides.selection = sel;
  showStreamPicker.value = false;
}
</script>

<template>
  <div class="space-y-4">
    <!-- 定时开始（通用） -->
    <div class="space-y-1.5">
      <div class="flex items-center justify-between">
        <Label class="cursor-pointer text-xs text-muted-foreground">定时开始</Label>
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
        <!-- 流选择 / 解析 / 重试 -->
        <div v-if="isOptionVisible('streamSelection', link.detectedType)" class="space-y-1.5">
          <Label class="text-xs text-muted-foreground">流选择</Label>
          <div class="flex gap-2">
            <Button variant="outline" size="sm" class="h-9" :disabled="parsing" @click="emit('parse')">
              <AppIcon v-if="parsing" name="Loader2" :size="14" class="mr-1.5 animate-spin" />
              <AppIcon v-else name="Search" :size="14" class="mr-1.5" />
              {{ link.streamInfo ? "重新解析" : link.parseFailed ? "重试解析" : "解析流" }}
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
          <p v-if="link.parseFailed && !parsing" class="text-xs text-red-400">
            解析失败，可重试或直接添加（下载时按默认处理）
          </p>
          <p v-else-if="link.overrides.selection" class="text-xs text-muted-foreground/70">
            已选：视频 {{ link.overrides.selection.video ?? "自动" }} · 音频
            {{ link.overrides.selection.audio ?? "自动" }} · 字幕
            {{ link.overrides.selection.subtitle ?? "自动" }}
          </p>
          <div v-if="showStreamPicker && link.streamInfo" class="rounded-lg border bg-muted/30 p-3">
            <StreamPickerInline
              :stream-info="link.streamInfo"
              :loading="parsing"
              @confirm="handleStreamConfirm"
              @cancel="showStreamPicker = false"
            />
          </div>
        </div>

        <!--
          以下六块（限速 maxSpeed / 下载范围 customRange / 容器格式 muxFormat /
          字幕格式 subtitleFormat / 仅下载字幕 subtitlesOnly / 解密密钥 key）
          从 LinkConfigPanel.vue 第 215–330 行逐字复制，字段绑定与 isOptionVisible
          门控保持不变。
        -->
      </div>
    </template>
  </div>
</template>

<style scoped>
.datetime-dark {
  color-scheme: dark;
}
</style>
```

- [ ] **Step 2: 从 LinkConfigPanel.vue 复制六个选项块**

打开 `src/components/task/LinkConfigPanel.vue`，将第 215–330 行的六个 `<div v-if="isOptionVisible(...)">` 块（限速、下载范围、容器格式、字幕格式、仅下载字幕、解密密钥）复制，替换上一步模板中的注释占位（放在流选择块之后、`</div>` 收尾之前）。这些块引用的 `link`、`isOptionVisible`、`MuxFormat`、`SubtitleFormat` 在本组件均已具备。

- [ ] **Step 3: 验证类型**

Run: `npm run type-check`
Expected: 通过。

- [ ] **Step 4: Commit**

```bash
git add src/components/task/LinkAdvancedSection.vue
git commit -m "feat: 高级设置组件（引擎驱动动态项 + 内联流选择 + 解析重试）"
```

---

### Task 9: 单条配置卡 `LinkConfigCard.vue`

**Files:**
- Create: `src/components/task/LinkConfigCard.vue`

- [ ] **Step 1: 创建组件**

```vue
<script setup lang="ts">
/**
 * 单条链接配置卡（L1 字段 + 高级手风琴）。
 * 纯编辑；目录浏览 / 解析经 emit 上抛向导（子组件不直接调 service）。
 */
import { computed, ref } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { AppIcon } from "@/components/common";
import LinkAdvancedSection from "./LinkAdvancedSection.vue";
import { typeBadgeLabel } from "./addTaskTypes";
import type { StagedLink } from "./addTaskTypes";

const props = defineProps<{
  recentDirs: string[];
  defaultDir: string;
  parsing: boolean;
}>();
const emit = defineEmits<{
  (e: "parse"): void;
  (e: "browseSaveDir"): void;
}>();
const link = defineModel<StagedLink>({ required: true });

const showAdvanced = ref(false);
const badge = computed(() => typeBadgeLabel(link.value.detectedType));
const saveDirPlaceholder = computed(() => props.defaultDir || "使用全局默认");
</script>

<template>
  <div class="space-y-4">
    <!-- 链接 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">链接</Label>
      <Input v-model="link.url" class="h-9 text-sm" />
      <div class="flex items-center gap-2 text-xs">
        <span class="rounded-full bg-primary/20 px-2 py-0.5 font-medium text-primary">
          {{ badge }}
        </span>
        <span v-if="link.parseFailed" class="text-red-400">解析失败</span>
        <span v-else-if="link.streamInfo" class="text-muted-foreground">已解析</span>
      </div>
    </div>

    <!-- 保存位置 + 记忆下拉 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">保存位置</Label>
      <div class="flex gap-2">
        <Input
          v-model="link.saveDir"
          :placeholder="saveDirPlaceholder"
          class="h-9 flex-1 text-sm"
        />
        <Button variant="outline" size="sm" class="h-9 px-3" @click="emit('browseSaveDir')">
          <AppIcon name="FolderOpen" :size="14" />
        </Button>
      </div>
      <DropdownMenu v-if="recentDirs.length">
        <DropdownMenuTrigger as-child>
          <button
            class="flex cursor-pointer items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
          >
            <AppIcon name="History" :size="12" />
            最近：{{ recentDirs.slice(0, 3).join(" · ") }}
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" class="max-w-[320px]">
          <DropdownMenuItem v-for="d in recentDirs" :key="d" @click="link.saveDir = d">
            <span class="truncate">{{ d }}</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <!-- 文件名 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">文件名</Label>
      <Input v-model="link.fileName" placeholder="自动从 URL 提取" class="h-9 text-sm" />
    </div>

    <!-- 高级设置（手风琴） -->
    <div class="border-t border-border/60 pt-3">
      <button
        class="flex cursor-pointer items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
        @click="showAdvanced = !showAdvanced"
      >
        <AppIcon :name="showAdvanced ? 'ChevronDown' : 'ChevronRight'" :size="14" />
        高级设置
      </button>
      <div v-if="showAdvanced" class="mt-3">
        <LinkAdvancedSection v-model="link" :parsing="parsing" @parse="emit('parse')" />
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 验证类型**

Run: `npm run type-check`
Expected: 通过。

- [ ] **Step 3: Commit**

```bash
git add src/components/task/LinkConfigCard.vue
git commit -m "feat: 单条配置卡（L1 字段 + 记忆下拉 + 高级手风琴）"
```

---

### Task 10: 重写 `AddTaskDialog.vue` 薄壳

**Files:**
- Modify: `src/components/task/AddTaskDialog.vue`（整体替换）

- [ ] **Step 1: 整体替换为三段式薄壳**

```vue
<script setup lang="ts">
/**
 * AddTaskDialog —— 三段式向导薄壳。
 * 流程编排在 useAddTaskWizard；本组件只做渲染 + 路由用户操作。
 */
import { computed, ref, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useAddTaskWizard } from "@/composables";
import LinkConfigCard from "./LinkConfigCard.vue";

interface Props {
  open: boolean;
}
const props = defineProps<Props>();
const emit = defineEmits<{ (e: "update:open", value: boolean): void }>();

const isOpen = computed({
  get: () => props.open,
  set: (v) => emit("update:open", v),
});

const {
  step,
  current,
  index,
  total,
  isSingle,
  isLast,
  showAddAll,
  isSubmitting,
  parseDone,
  parseTotal,
  parsingId,
  dirs,
  defaultDir,
  showDuplicate,
  duplicateTask,
  reset,
  submitPaste,
  retryParse,
  browseSaveDir,
  addCurrent,
  skip,
  addAll,
  confirmDuplicate,
  cancelDuplicate,
} = useAddTaskWizard();

const pasteText = ref("");
const isDragging = ref(false);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

watch(isOpen, async (open) => {
  if (open) {
    reset();
    pasteText.value = "";
    await nextTick();
    textareaRef.value?.focus();
  }
});

// 向导进入 done → 关闭弹窗
watch(step, (s) => {
  if (s === "done") isOpen.value = false;
});

function handleSubmitPaste() {
  if (pasteText.value.trim()) void submitPaste(pasteText.value);
}
function onPasteKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleSubmitPaste();
  }
}
function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const text = e.dataTransfer?.getData("text/plain");
  if (text) pasteText.value = text;
}
// 配置步 Enter = 添加/完成（避开 textarea，其由 onPasteKeydown 处理）
function onContentKeydown(e: KeyboardEvent) {
  if (e.key !== "Enter" || e.shiftKey) return;
  if ((e.target as HTMLElement)?.tagName === "TEXTAREA") return;
  if (step.value !== "config") return;
  e.preventDefault();
  void addCurrent();
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="flex max-h-[85vh] max-w-[min(600px,calc(100vw-2rem))] flex-col"
      @keydown="onContentKeydown"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
          <span
            v-if="step === 'config' && !isSingle"
            class="text-sm font-normal text-muted-foreground"
          >
            {{ index + 1 }}/{{ total }}
          </span>
        </DialogTitle>
        <DialogDescription class="sr-only">粘贴链接并配置下载任务</DialogDescription>
      </DialogHeader>

      <div class="-mx-2 flex-1 space-y-4 overflow-y-auto px-2">
        <!-- 步骤 1：粘贴 -->
        <div
          v-if="step === 'paste'"
          class="space-y-4"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          @drop="onDrop"
        >
          <div class="relative">
            <div
              v-if="isDragging"
              class="absolute inset-0 z-10 flex items-center justify-center rounded-lg border-2 border-dashed border-primary bg-primary/10"
            >
              <span class="text-sm font-medium text-primary">释放以粘贴链接</span>
            </div>
            <textarea
              ref="textareaRef"
              v-model="pasteText"
              placeholder="粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 直链）"
              class="h-40 w-full resize-none rounded-lg border bg-muted/50 px-3 py-2 text-sm transition-colors focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/50"
              @keydown="onPasteKeydown"
            />
          </div>
          <div class="flex justify-end">
            <Button :disabled="!pasteText.trim()" @click="handleSubmitPaste">
              <AppIcon name="ArrowRight" :size="16" class="mr-2" />
              解析并添加
            </Button>
          </div>
        </div>

        <!-- 步骤 2：解析中 -->
        <div
          v-else-if="step === 'parsing'"
          class="flex flex-col items-center justify-center gap-3 py-16"
        >
          <AppIcon name="Loader2" :size="32" class="animate-spin text-primary" />
          <span class="text-sm text-muted-foreground">
            正在解析 {{ parseTotal }} 个链接…（{{ parseDone }}/{{ parseTotal }}）
          </span>
        </div>

        <!-- 步骤 3：逐条配置 -->
        <div v-else-if="step === 'config' && current" class="space-y-4">
          <LinkConfigCard
            :model-value="current"
            :recent-dirs="dirs"
            :default-dir="defaultDir"
            :parsing="parsingId === current.id"
            @parse="current && retryParse(current)"
            @browse-save-dir="browseSaveDir"
          />
          <div class="flex items-center justify-between border-t pt-3">
            <Button v-if="!isSingle" variant="ghost" size="sm" @click="skip">跳过</Button>
            <span v-else />
            <div class="flex gap-2">
              <Button
                v-if="showAddAll"
                variant="outline"
                size="sm"
                :disabled="isSubmitting"
                @click="addAll"
              >
                全部添加
              </Button>
              <Button size="sm" :disabled="isSubmitting" @click="addCurrent">
                <AppIcon name="Download" :size="16" class="mr-1.5" />
                {{ isLast ? "完成" : "添加" }}
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- URL 重复确认 -->
      <UrlDuplicateDialog
        v-model:open="showDuplicate"
        :existing-task="duplicateTask"
        @confirm="confirmDuplicate"
        @cancel="cancelDuplicate"
      />
    </DialogContent>
  </Dialog>
</template>
```

- [ ] **Step 2: 验证类型**

Run: `npm run type-check`
Expected: 通过。

- [ ] **Step 3: Commit**

```bash
git add src/components/task/AddTaskDialog.vue
git commit -m "refactor: AddTaskDialog 重写为三段式向导薄壳"
```

---

### Task 11: 删除旧文件 + 校验无残留引用

**Files:**
- Delete: `src/components/task/TaskStagingList.vue`
- Delete: `src/components/task/LinkConfigPanel.vue`
- Delete: `src/components/task/staging-types.ts`

- [ ] **Step 1: 确认无残留引用**

Run:
```bash
grep -rn "TaskStagingList\|LinkConfigPanel\|staging-types\|seedPresetOverrides\|BatchDefaults" src/ || echo "NO_RESIDUE"
```
Expected: 输出 `NO_RESIDUE`（`index.ts` 桶从未导出这些内部件；`resolveLinkToTask`/`linkOptionVisibility` 已在前序任务迁走）。若有残留，先修正引用再继续。

- [ ] **Step 2: 删除三个文件**

```bash
git rm src/components/task/TaskStagingList.vue src/components/task/LinkConfigPanel.vue src/components/task/staging-types.ts
```

- [ ] **Step 3: 验证类型与测试**

Run: `npm run type-check && npx vitest run`
Expected: type-check 通过；全部单测 PASS（parseLinks / resolveLinkToTask / useRecentDirs）。

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor: 删除暂存外壳旧件（TaskStagingList/LinkConfigPanel/staging-types）"
```

---

### Task 12: 全量验证

**Files:** （无新增）

- [ ] **Step 1: 前端静态检查**

Run: `npm run type-check && npm run lint:check`
Expected: 均通过。若 lint 报未用 import（如 `MuxFormat`/`SubtitleFormat` 在复制块后确被使用则无碍），按提示修正。

- [ ] **Step 2: 前端单测**

Run: `npm test`
Expected: 全部 PASS。

- [ ] **Step 3: 后端契约未动确认**

Run: `cd src-tauri && cargo clippy -- -D warnings && cargo fmt --check`
Expected: 通过（后端零改动，此步防回归）。

- [ ] **Step 4: 手动冒烟（`npm run tauri dev`）**

验证清单：
- 单链接：粘贴 1 条 → 一张卡（无页码/全部添加/跳过）→ 完成 → 任务入库；`auto_start_download` 开则自动开始。
- 多链接：逐条"添加"推进、页码 `i/N` 正确；"全部添加"批量入库剩余；"跳过"跳过当前。
- 路径记忆：提交后重开弹窗，默认/最近下拉出现该目录；上限 5、去重、最新在前。
- 无效链接（如 `.html`）解析阶段剔除并 toast；流媒体解析失败显示"解析失败/重试"，可重试或直接添加。
- 重复 URL：逐条弹 `UrlDuplicateDialog`；批量静默跳过并在结束 toast 汇报。
- 键盘：粘贴框 Enter 提交、Shift+Enter 换行；配置步 Enter 添加/完成；Esc 关闭。

- [ ] **Step 5: 更新功能状态文档**

按项目规则更新 `docs/design/06-feature-status.md` 中添加任务相关条目为 `[x]`，指向新文件。

- [ ] **Step 6: Commit**

```bash
git add docs/design/06-feature-status.md
git commit -m "docs: 更新添加任务向导重设计的功能状态"
```
