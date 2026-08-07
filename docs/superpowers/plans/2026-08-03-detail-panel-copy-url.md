# 任务详情「下载链接」复制按钮 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在任务详情侧栏「下载链接」标题行加一个复制按钮，一键复制任务源 URL，带图标态切换 + toast 反馈；顺带补齐剪贴板插件缺失的读/写权限。

**Architecture:** 前端单向数据流：`TaskDetailPanel` → `clipboardService.writeText` → `@tauri-apps/plugin-clipboard-manager` → 系统剪贴板。权限经 Tauri capabilities 声明。组件局部 `copied` ref 管理 1.5s 图标态，`onBeforeUnmount` 清理计时器。

**Tech Stack:** Vue 3 `<script setup>` + TypeScript、vue-i18n、shadcn-vue `Button`、lucide 图标（经 `AppIcon`）、`@tauri-apps/plugin-clipboard-manager`、vitest（仅跑现有套件）。

## Global Constraints

- 组件不直接调用 Tauri API，统一经 `src/services/`（CLAUDE.md 架构规则）
- 组件销毁前必须清理副作用（计时器等）（CLAUDE.md 禁止事项）
- 禁止 `any` 类型；忽略 TypeScript 错误属禁止事项
- 新增用户可见文案必须三语（zh-CN / zh-TW / en-US）同步
- 提交信息格式：`feat:` / `fix:` / `docs:` 等（CLAUDE.md 提交规范）
- 每条 commit message 末尾附 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`  trailer
- 验证基准：`npm run type-check`、`npm run lint`、`npm test` 三项全绿
- 本功能按已批准 spec（`docs/superpowers/specs/2026-08-03-detail-panel-copy-url-design.md` §6）不新增单元测试：改动为 UI 接线 + 服务薄封装，无可测纯函数，以类型检查 + 现有测试套件 + 手动运行为验证手段

---

## Task 0: 前置——提交工作区中积压的无关改动

**背景：** 当前工作区有两批未提交改动，必须先落盘，避免与本功能混在同一次提交：
1. 上一轮「播放文件」bug 修复（`src-tauri/src/app/commands/system.rs`、`src-tauri/src/lib.rs`、`src/services/systemService.ts`、`src/components/task/TaskDetailPanel.vue`、`src/components/task/TaskCard.vue`）
2. 本功能的设计文档（`docs/superpowers/specs/2026-08-03-detail-panel-copy-url-design.md`，未跟踪）

**Files:**
- Commit: 上述 5 个已修改文件 + 1 个未跟踪 spec 文档

- [ ] **Step 1: 确认工作区状态符合预期**

Run: `git status --short`

Expected: 恰好包含以下条目（顺序可能不同）：
```
 M src-tauri/src/app/commands/system.rs
 M src-tauri/src/lib.rs
 M src/services/systemService.ts
 M src/components/task/TaskDetailPanel.vue
 M src/components/task/TaskCard.vue
?? docs/superpowers/
```
若出现其它意外改动，停下并向用户确认，不要继续。

- [ ] **Step 2: 提交播放修复**

```bash
git add src-tauri/src/app/commands/system.rs src-tauri/src/lib.rs \
        src/services/systemService.ts \
        src/components/task/TaskDetailPanel.vue src/components/task/TaskCard.vue
git commit -m "fix: 播放文件改为调用系统默认程序打开

原 handleOpenFile 误接 open_file_in_explorer（语义为在资源管理器中
定位文件），导致点播放只打开文件夹。新增 open_file_with_default 命令
（tauri-plugin-opener），TaskDetailPanel 与 TaskCard 的播放按钮改接新命令。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

- [ ] **Step 3: 提交设计文档**

```bash
git add docs/superpowers/specs/2026-08-03-detail-panel-copy-url-design.md
git commit -m "docs: 详情面板下载链接复制功能设计文档

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

- [ ] **Step 4: 确认工作区干净**

Run: `git status --short`
Expected: 无输出。

---

## Task 1: 剪贴板权限 + 写入服务

**Files:**
- Modify: `src-tauri/capabilities/default.json`（permissions 数组末尾）
- Modify: `src/services/clipboardService.ts`（import 行 + class 内新增方法）

**Interfaces:**
- Consumes: `@tauri-apps/plugin-clipboard-manager` 的 `writeText`（插件已在 Rust 侧 lib.rs:29 注册，无需改动后端）
- Produces: `clipboardService.writeText(text: string): Promise<void>` —— Task 2 的 UI 层依赖此签名

**说明：** `clipboard-manager:allow-read-text` 为顺带修复——现有剪贴板监控
（`useClipboardWatcher`）的 `readText` 因缺此权限大概率一直在静默失败
（spec §4）。两项权限一并补上。

- [ ] **Step 1: 在 capabilities 中新增两项权限**

将 `src-tauri/capabilities/default.json` 的 permissions 数组末尾两行：

```json
    "dialog:allow-open",
    "dialog:allow-save"
  ]
```

改为：

```json
    "dialog:allow-open",
    "dialog:allow-save",
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text"
  ]
```

- [ ] **Step 2: 为 clipboardService 增加 writeText**

`src/services/clipboardService.ts` 完整目标内容（在现有基础上改 import 行 + 加一个方法，其余不动）：

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

（文档注释首行「读取」→「读取/写入」同步更新。）

- [ ] **Step 3: 验证类型与现有测试**

Run: `npm run type-check && npm test`
Expected: type-check 无错误；vitest 现有套件全绿。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/capabilities/default.json src/services/clipboardService.ts
git commit -m "feat: 剪贴板写入服务并补齐 clipboard 插件权限

- clipboardService 新增 writeText（供复制链接使用）
- capabilities 补 allow-write-text；顺带补 allow-read-text，
  修复剪贴板监控 readText 因缺权限静默失败的问题

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 2: 三语 i18n 文案

**Files:**
- Modify: `src/locales/zh-CN.ts`（messages 块，`clipboardUrlsDetected` 行之后）
- Modify: `src/locales/zh-TW.ts`（同上）
- Modify: `src/locales/en-US.ts`（同上）

**Interfaces:**
- Produces: i18n key `messages.urlCopied` —— Task 3 的 toast 依赖此 key

**说明：** 复制失败的 toast 复用既有 `settings.preset.copyFailed`（文案即通用的「复制失败」），不新增 key（spec §3.4 的最小化原则）。

- [ ] **Step 1: zh-CN**

`src/locales/zh-CN.ts` 中找到：

```typescript
    clipboardUrlDetected: "已添加下载链接",
    clipboardUrlsDetected: "已添加 {count} 个下载链接",
```

在其后插入一行：

```typescript
    urlCopied: "链接已复制",
```

- [ ] **Step 2: zh-TW**

`src/locales/zh-TW.ts` 中找到：

```typescript
    clipboardUrlDetected: "已新增下載連結",
    clipboardUrlsDetected: "已新增 {count} 個下載連結",
```

在其后插入一行：

```typescript
    urlCopied: "連結已複製",
```

- [ ] **Step 3: en-US**

`src/locales/en-US.ts` 中找到：

```typescript
    clipboardUrlDetected: "Download link added",
    clipboardUrlsDetected: "{count} download links added",
```

在其后插入一行：

```typescript
    urlCopied: "Link copied",
```

- [ ] **Step 4: 验证**

Run: `npm run type-check && npm run lint`
Expected: 无错误、无新告警。

- [ ] **Step 5: 提交**

```bash
git add src/locales/zh-CN.ts src/locales/zh-TW.ts src/locales/en-US.ts
git commit -m "feat: 新增链接复制 toast 三语文案 messages.urlCopied

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 3: TaskDetailPanel 复制按钮与逻辑

**Files:**
- Modify: `src/components/task/TaskDetailPanel.vue`（script imports、状态与 handler、template URL 区块标题行）

**Interfaces:**
- Consumes: Task 1 的 `clipboardService.writeText(text: string): Promise<void>`；Task 2 的 `messages.urlCopied`；既有 `common.copy`、`settings.preset.copyFailed`
- Produces: 无（终端 UI 任务）

**说明：** 按钮样式镜像 `PresetsTab.vue:202-210` 的 ghost 图标按钮
（`variant="ghost" size="icon" class="h-7 w-7 cursor-pointer"` + 14px 图标），
但**不**带 `opacity-0 group-hover:opacity-100`（spec §5：按钮常显）。
图标成功态用 `text-green-500`（对应 CLAUDE.md `--accent-success: #22c55e`，
与本文件既有 `text-amber-600` 等硬编码调色板用法一致）。

- [ ] **Step 1: 更新 script 的 imports**

`src/components/task/TaskDetailPanel.vue` 中：

```typescript
import { computed, ref, watch, onMounted } from "vue";
```
改为：
```typescript
import { computed, ref, watch, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
```

```typescript
import { useTasks, useDownloader } from "@/composables";
```
改为：
```typescript
import { useTasks, useDownloader, useToast } from "@/composables";
```

```typescript
import { systemService } from "@/services";
```
改为：
```typescript
import { systemService, clipboardService } from "@/services";
```

在 `import { AppIcon } from "@/components/common";` 之后新增一行：

```typescript
import { Button } from "@/components/ui/button";
```

- [ ] **Step 2: 新增状态与 handler**

在 `const taskStore = useTaskStore();` 之后新增两行：

```typescript
const { t } = useI18n();
const toast = useToast();
```

在 `handleClose` 定义之后新增：

```typescript
// 复制下载链接（图标态 1.5s 自动还原）
const copied = ref(false);
let copiedTimer: ReturnType<typeof setTimeout> | null = null;

const handleCopyUrl = async () => {
  if (!task.value) return;
  try {
    await clipboardService.writeText(task.value.url);
    copied.value = true;
    toast.success(t("messages.urlCopied", "链接已复制"));
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (e) {
    console.error("Failed to copy URL:", e);
    toast.error(t("settings.preset.copyFailed", "复制失败"));
  }
};

onBeforeUnmount(() => {
  if (copiedTimer) clearTimeout(copiedTimer);
});
```

- [ ] **Step 3: 改造 template 的 URL 区块标题行**

将：

```vue
            <!-- URL -->
            <div class="space-y-2">
              <h4
                class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
              >
                下载链接
              </h4>
              <div class="bg-muted/30 rounded-lg p-2.5">
```

改为：

```vue
            <!-- URL -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <h4
                  class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
                >
                  下载链接
                </h4>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 cursor-pointer"
                  :title="t('common.copy', '复制')"
                  @click="handleCopyUrl"
                >
                  <AppIcon
                    :name="copied ? 'Check' : 'Copy'"
                    :size="14"
                    :class="copied ? 'text-green-500' : ''"
                  />
                </Button>
              </div>
              <div class="bg-muted/30 rounded-lg p-2.5">
```

（标题行以下的 URL 文本块原样保留。）

- [ ] **Step 4: 验证类型、lint、现有测试**

Run: `npm run type-check && npm run lint && npm test`
Expected: 三项全部通过，无新告警。

- [ ] **Step 5: 提交**

```bash
git add src/components/task/TaskDetailPanel.vue
git commit -m "feat: 任务详情下载链接支持一键复制

标题行常显复制按钮（ghost 图标按钮），点击写入剪贴板后图标切换为
Check 并 toast「链接已复制」，1.5s 自动还原；失败 toast 提示。
计时器在组件卸载时清理。

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 4: 功能状态文档 + 手动验证

**Files:**
- Modify: `docs/design/06-feature-status.md`（「十、UI/UX」表格）

**Interfaces:**
- Consumes: Task 1-3 已完成的实现
- Produces: 无

- [ ] **Step 1: 在 06-feature-status.md 的 UI/UX 表中插入一行**

在「任务列表」行（`| 任务列表 | P0 | `[x]` | ... |`）之后插入：

```markdown
| 详情链接复制 | P2 | `[x]` | `src/components/task/TaskDetailPanel.vue` + `src/services/clipboardService.ts` | 标题行复制按钮：图标态切换 + toast 反馈；同期补齐 clipboard 读/写权限（修复剪贴板监控静默失败） |
```

- [ ] **Step 2: 手动运行验证（GUI，需人工确认）**

Run: `npm run tauri dev`

验证清单：
1. 任一任务 → 点开详情侧栏 → 「下载链接」标题行右侧出现复制图标按钮（常显）
2. 点击 → 图标变绿色 Check，1.5s 后还原；右下角 toast「链接已复制」
3. 在任意编辑器 Ctrl+V，粘贴内容与任务 URL 完全一致
4. 连续快速点击两次 → 计时器重置、两次 toast，无报错
5. 设置 → 常规 → 打开「剪贴板监视」→ 复制一条 m3u8 链接后切回应用 → 应弹出「已添加下载链接」提示（验证 Task 1 的 read-text 权限修复生效）

- [ ] **Step 3: 提交文档**

```bash
git add docs/design/06-feature-status.md
git commit -m "docs: 更新功能状态——详情链接复制

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

- [ ] **Step 4: 终检**

Run: `npm run type-check && npm run lint && npm test && git status --short`
Expected: 三项检查通过；工作区仅剩本计划文档（`docs/superpowers/plans/`）未跟踪——是否提交交由用户决定。
