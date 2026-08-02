# 任务卡片右键上下文菜单 · 设计规格

> **日期**：2026-08-03
> **状态**：已批准，待实现
> **主要产物**：`src/components/task/TaskContextMenu.vue`（新建）
> **涉及文件**：`TaskCard.vue` · `TaskList.vue` · `HomeView.vue` · `AddTaskDialog.vue` · `clipboardService.ts` · `locales/{zh-CN,en-US,zh-TW}.ts` · `src/components/ui/context-menu/`（新增脚手架）

---

## 1. 背景与目标

### 1.1 现状

每个任务卡片已有两条操作通道，且按状态划分的核心操作已全覆盖：

| 通道 | 操作 | 层级 |
| --- | --- | --- |
| 悬停快捷按钮（`TaskQuickActions`） | 打开目录、播放、日志、暂停/继续/开始、重试、停止、删除 | Level 2（悬停展开） |
| 点击卡片 → `TaskDetailPanel` | 完整详情 + 底部操作按钮 | Level 3（导航进入） |

### 1.2 问题

存在一类**当前 UI 无处安放**的操作：复制下载链接、复制文件名、复制本地文件路径、以此链接重新下载。放进悬停按钮会挤爆卡片（悬停区最多已 6 个按钮）；放进详情面板又太深（需要先点开 Level 3）。

### 1.3 目标

为任务卡片添加右键上下文菜单，定位为**「收纳放不进悬停按钮的次要操作」**：

1. 补齐 4 项当前缺失的操作：复制下载链接、复制文件名、复制文件路径、以此链接重新下载；
2. 提供「打开详情」作为点击卡片的等价入口；
3. **不重复**悬停按钮已有的状态操作（开始/暂停/停止/重试/删除），保持单一操作入口；
4. 纯前端实现，零后端 / 零数据库改动。

## 2. 非目标（YAGNI 边界）

- ❌ 菜单不包含任何状态操作（开始/暂停/停止/重试/删除由悬停按钮独占）；
- ❌ 不做重命名 / 修改保存位置（需要新增后端命令，另立条目）；
- ❌ 不清理 `TaskQuickActions` 现有硬编码中文（另立任务，不随本次改动）；
- ❌ 不做 TaskList 空白区域右键菜单（"清除全部已完成"等批量操作已有独立入口）；
- ❌ 不做菜单内快捷键标注（Radix 自带首字母 typeahead 已足够）。

## 3. 总体设计

### 3.1 方案选型

| 方案 | 方法 | 结论 |
| --- | --- | --- |
| **A（采用）** | 新增 shadcn-vue **ContextMenu** 组件（基于 radix-vue），每张卡片包 `ContextMenuTrigger` | 原生右键语义；键盘导航 / Esc / 焦点管理内建；Portal 渲染不被列表容器裁剪；与已 vendored 的 `dropdown-menu` 同构 |
| B | 复用 `DropdownMenu` + 自造 `@contextmenu` 光标锚点 | Radix DropdownMenu 只能锚定到 trigger 元素，需隐形锚点 hack，否决 |
| C | 手写绝对定位浮层 | 键盘导航 / 点外关闭 / 焦点管理需自造，与项目 shadcn-vue 统一基础组件的方针相悖，否决 |

**落地动作**：`npx shadcn-vue@latest add context-menu`，产物落入 `src/components/ui/context-menu/`。

### 3.2 组件结构

```
新增  src/components/ui/context-menu/        shadcn-vue 标准脚手架（~13 个小文件）
新增  src/components/task/TaskContextMenu.vue  纯展示组件（仿 TaskQuickActions 模型）
修改  src/services/clipboardService.ts       新增 writeText() 方法
修改  src/components/task/TaskCard.vue       包 ContextMenuTrigger + 新增 handler/事件
修改  src/components/task/TaskList.vue       事件透传 taskRedownload
修改  src/views/HomeView.vue                 prefillUrl + handleRedownload
修改  src/components/task/AddTaskDialog.vue  新增 initialUrl prop + 自动推进
修改  src/locales/{zh-CN,en-US,zh-TW}.ts     新增 8 个键 × 3 语言
```

**`TaskContextMenu.vue` 契约**（严格仿 `TaskQuickActions` 纯展示模型）：

```typescript
interface Props {
  task: DownloadTask;
  fileExists: boolean;   // 预留，当前显隐矩阵不依赖它（见 §4 注 ①）
}

const emit = defineEmits<{
  (e: "redownload"): void;
  (e: "copyUrl"): void;
  (e: "copyFileName"): void;
  (e: "copyFilePath"): void;
  (e: "openDetail"): void;
}>();
```

组件内部仅负责：按状态计算各项 `v-if` 显隐 + 渲染 `ContextMenuContent`。零业务逻辑、零 service 调用。

### 3.3 事件流

只有「重新下载」需要向上冒泡，其余在 TaskCard 内闭环：

```
TaskContextMenu（emit 动作）
  ├─ copyUrl / copyFileName / copyFilePath
  │    → TaskCard.handleCopyXxx → clipboardService.writeText() + toast      【TaskCard 闭环】
  ├─ openDetail
  │    → 复用现有 handleClick → emit("click", task)                          【TaskCard 闭环，走已有详情面板链路】
  └─ redownload
       → TaskCard emit("redownload", task)
       → TaskList emit("taskRedownload", task)          【镜像既有 taskClick 透传链】
       → HomeView.handleRedownload(task):
            prefillUrl.value = task.url
            showAddDialog.value = true
```

## 4. 菜单项矩阵

平铺三段式（动作 / 复制 / 导航），无子菜单：

```
┌─────────────────────────────┐
│ ⟳  以此链接重新下载            │  ← RotateCw
├─────────────────────────────┤
│ 🔗 复制下载链接                │  ← Link2
│ 📄 复制文件名                  │  ← FileText
│ 📁 复制文件路径                │  ← Folder（条件显示 ①）
├─────────────────────────────┤
│ ⧉  打开详情                   │  ← PanelRightOpen
└─────────────────────────────┘
```

图标均取自 `lucide-vue-next`（`AppIcon` 全量导入，无白名单限制）。

**显隐矩阵**（与 `TaskQuickActions` 一致：用 `v-if` 隐藏，不做 disabled 态）：

| 菜单项 | pending | analyzing | downloading | paused | merging/muxing | completed | failed | cancelled |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| 以此链接重新下载 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 复制下载链接 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 复制文件名 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 复制文件路径 | — | — | — | — | — | ✓ ① | — | — |
| 打开详情 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

**①「复制文件路径」显示条件**：`status === "completed" && task.outputPath` 非空。**不**以 `fileExists` 为门槛——文件被移动/删除后，复制原路径反而能帮用户定位"它原来在哪"。

**刻意的设计决定**：

1. 不与悬停按钮重复——状态操作一概不进菜单，避免两套入口的认知负担；
2. 「复制文件路径」（取字符串）与悬停的「打开目录」（开资源管理器）语义分离，图标也不同（`Folder` vs `FolderOpen`）。

## 5. 交互细节

### 5.1 重新下载：预填 + 自动推进到配置步

**链路**：

1. 菜单点击 → 按 §3.3 冒泡至 HomeView，`prefillUrl = task.url`、`showAddDialog = true`；
2. `AddTaskDialog` 新增可选 prop `initialUrl?: string | null`，在既有 `watch(isOpen)` 分支中：

   ```typescript
   watch(isOpen, async (open) => {
     if (!open) return;
     reset();
     if (initialUrl) {
       pasteText.value = initialUrl;
       void submitPaste(initialUrl);   // 复用既有解析链路（resolveLinkToTask），自动进入 config 步
     } else {
       pasteText.value = "";
     }
     await nextTick();
     textareaRef.value?.focus();
   });
   ```

3. HomeView 在 `watch(showAddDialog)` 中，关闭时 `prefillUrl = null`（一次性交接，防止污染下一次普通「添加任务」）。

**边界情况**：

- **URL 重复**：`submitPaste` 走既有链路，自然进入 `UrlDuplicateDialog` 流程，用户可选择"仍然添加"，无需特殊处理；
- **解析失败**（理论上不会——URL 曾成功使用过）：wizard 既有错误态在对话框内展示，用户可见原因；
- **文件名冲突**：既有 `addTask` 时间戳重命名逻辑照常生效。

**不新建解析路径**：预填只是替用户省掉手动粘贴，`resolveLinkToTask`、默认保存目录、重复检测、冲突重命名全部复用。

### 5.2 复制系列：三步闭环

```typescript
// src/services/clipboardService.ts —— 插件已安装（@tauri-apps/plugin-clipboard-manager ^2.3.2），
// 现仅封装了 readText，新增：
writeText(text: string): Promise<void>   // 转调插件 writeText，错误向上抛
```

TaskCard 三个 handler：

| Handler | 写入内容 | 成功 toast |
| --- | --- | --- |
| `handleCopyUrl` | `task.url` | 已复制下载链接 |
| `handleCopyFileName` | `task.fileName` | 已复制文件名 |
| `handleCopyFilePath` | `task.outputPath!`（由矩阵保证非空） | 已复制文件路径 |

成功时：toast 即为确认反馈（菜单项点击后由 Radix 自动关闭，默认行为）。写入失败（极罕见，如权限问题）：仅 `console.error`，不弹 toast——不为近乎不可能发生的路径扩张文案键面。不加其他动效。

toast 走 `src/composables/useToast`（`Toaster` 已在 `App.vue` 挂载）。

### 5.3 i18n：8 个键 × 3 语言

菜单项挂 `task.contextMenu.*`（顶层 `task` 命名空间已存在）；toast 挂 `messages.*`（与既有 `clipboardUrlDetected` 同级）：

| Key | zh-CN | en-US | zh-TW |
| --- | --- | --- | --- |
| `task.contextMenu.redownload` | 以此链接重新下载 | Redownload from this link | 以此連結重新下載 |
| `task.contextMenu.copyUrl` | 复制下载链接 | Copy download link | 複製下載連結 |
| `task.contextMenu.copyFileName` | 复制文件名 | Copy file name | 複製檔案名稱 |
| `task.contextMenu.copyFilePath` | 复制文件路径 | Copy file path | 複製檔案路徑 |
| `task.contextMenu.openDetail` | 打开详情 | Open details | 開啟詳情 |
| `messages.copiedUrl` | 已复制下载链接 | Download link copied | 已複製下載連結 |
| `messages.copiedFileName` | 已复制文件名 | File name copied | 已複製檔案名稱 |
| `messages.copiedFilePath` | 已复制文件路径 | File path copied | 已複製檔案路徑 |

键盘交互零开发：Radix ContextMenu 自带 ↑↓ 导航、Esc 关闭、首字母 typeahead。

## 6. 测试与验收

### 6.1 单元测试（Vitest）

**前置**：新增 `@vue/test-utils` 到 devDependencies（项目当前仅有纯函数测试，无组件挂载能力；此为组件测试的标准配套）。

| 测试文件 | 用例 |
| --- | --- |
| `src/components/task/TaskContextMenu.test.ts`（新建） | ① 任意状态下 4 个常驻项均渲染；② "复制文件路径" 仅在 `completed + outputPath` 时渲染，其余状态（含 completed 无 outputPath）不渲染；③ 点击各项 emit 对应事件 |
| `src/components/task/TaskCard.test.ts`（新建） | ① mock `clipboardService`，三个复制 handler 写入正确内容并触发 toast；② 菜单 redownload → 组件 emit `redownload`；③ 左键点击卡片仍 emit `click`（回归） |
| `src/components/task/AddTaskDialog.test.ts`（新建） | ① mock `useAddTaskWizard`，带 `initialUrl` 打开 → `submitPaste(initialUrl)` 被调用；② 不带 `initialUrl` 打开 → `submitPaste` 不被调用（回归） |

### 6.2 手工验收清单

- [ ] 右键 pending / downloading / paused / completed / failed 五种状态卡片 → 菜单在光标处打开，条目构成符合 §4 矩阵；
- [ ] 三种复制 → 剪贴板内容正确 + toast 出现；
- [ ] 重新下载 → 对话框打开且**自动到达配置步**、URL 已填入；调整文件名确认后新任务创建成功；
- [ ] 对已存在 URL 重新下载 → 自然进入 `UrlDuplicateDialog` 流程；
- [ ] Esc / 点击区域外关闭菜单；↑↓ 键导航；右键另一张卡片菜单切换；
- [ ] 无回归：左键点开详情正常；悬停快捷按钮正常；右键悬停按钮所在区域也弹出菜单；
- [ ] 三语言切换 → 菜单与 toast 文案正确。

### 6.3 质量门禁与文档收尾

```bash
npm run type-check && npm run lint && npm test
```

按 CLAUDE.md 规则：完成后在 `docs/design/06-feature-status.md` 任务卡片相关表格新增一行：

```markdown
| 任务卡片右键菜单 | P2 | `[x]` | `src/components/task/TaskContextMenu.vue` | 复制链接/文件名/路径、重新下载、打开详情 |
```

## 7. 影响范围（Blast Radius）

| 被改动 | 依赖方 | 风险 |
| --- | --- | --- |
| `TaskCard.vue` | `TaskList.vue` | 低：新增 emit 与 DOM 包装层，既有 `click` 行为不变 |
| `TaskList.vue` | `HomeView.vue`（2 处渲染） | 低：仅新增事件透传 |
| `AddTaskDialog.vue` | `HomeView.vue` | 低：新 prop 可选，缺省走原路径 |
| `clipboardService.ts` | `useClipboardWatcher` | 无：纯新增方法，不改 readText |
| `locales/*` | 全局 i18n | 无：纯新增键 |

**发布节奏提示**：本功能纯前端、零后端/DB 风险，与进行中的发布前审计（HANDOFF.md）不冲突；若临近发布冻结，可于发布后合入，设计不受影响。
