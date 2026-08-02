# 设计文档：任务详情「下载链接」复制按钮

- **日期**：2026-08-03
- **状态**：已批准，待实现
- **影响范围**：前端 + Tauri capabilities 配置（无 Rust 代码改动，无数据库改动）

## 1. 背景与目标

任务详情侧边栏（`TaskDetailPanel.vue`）的「下载链接」区块目前是一段纯文本
（`text-xs break-all text-muted-foreground`），没有任何复制入口。在 320px 宽的
侧栏里手动拖选一条长 URL 体验很差，而「取回源链接」是下载器的高频操作
（链接过期后重新取链、分享给他人、换工具处理）。

**目标**：一键复制任务源 URL 到剪贴板，并给出明确的操作反馈。

## 2. 交互规格

```
下载链接                                 [⧉]   ← ghost 图标按钮
┌────────────────────────────┐
│ https://example.com/video/ │
│ index.m3u8?token=abc123... │
└────────────────────────────┘

点击后 1.5 秒内：
下载链接                                 [✓]   ← Check 图标，success 色
+ toast「链接已复制」
```

| 状态 | 表现 |
| --- | --- |
| 默认 | `Copy` 图标（14px），ghost 按钮，hover 显示背景（沿用 `PresetsTab.vue` 复制按钮样式） |
| 按钮 title | 复用现有 i18n key `common.copy` |
| 复制成功 | 图标切换为 `Check` 并着 success 色，1.5 秒后自动还原为 `Copy`；同时 toast「链接已复制」 |
| 复制失败 | toast「复制失败」+ `console.error`；图标保持 `Copy` 不变 |
| 连续点击 | 重置 1.5s 计时器并再次 toast，无副作用累积（不额外防抖） |
| 组件卸载 | `onBeforeUnmount` 清理还原计时器（遵守 CLAUDE.md「组件销毁后不清理副作用」禁令） |

URL 文本块本身保持现状（可选中、不可点击），不改变其展示形态。

## 3. 技术设计

数据流：`TaskDetailPanel → clipboardService.writeText → @tauri-apps/plugin-clipboard-manager → 系统剪贴板`

### 3.1 权限层 — `src-tauri/capabilities/default.json`

新增两项权限：

- `clipboard-manager:allow-write-text` — 本功能必需。插件 JS API 经 IPC 调用，
  受 capabilities 约束；缺失时 `writeText` 会被拒绝。
- `clipboard-manager:allow-read-text` — **顺带修复**（见 §4）。

插件本体已在 Cargo.toml:27 / lib.rs:29 / package.json:29 注册，无需新增依赖。

### 3.2 服务层 — `src/services/clipboardService.ts`

新增 `writeText(text: string): Promise<void>`，封装插件 `writeText`，
与现有 `readText` 同款形态。服务层只做透传，失败由调用方处理。

### 3.3 组件层 — `src/components/task/TaskDetailPanel.vue`

- URL 区块标题行（现 L149-153）改为 `flex items-center justify-between`，
  右侧放置复制按钮（`AppIcon` + 原生 `button`，样式对齐 `PresetsTab.vue`
  的 ghost 图标按钮：`h-6 w-6 rounded hover:bg-muted` 级别）
- 组件局部状态：`const copied = ref(false)`
- `handleCopyUrl()`：`await clipboardService.writeText(task.url)` 成功后
  `copied = true` + `toast.success(t("messages.urlCopied"))`，1.5s 后还原；
  捕获异常则 `toast.error` + `console.error`
- 计时器句柄存于组件作用域，`onBeforeUnmount` 时 `clearTimeout`

**不**新建 `useCopyToClipboard` composable：当前仅一个消费点，逻辑约 15 行；
待出现第二个复制场景（如错误信息复制）时再提取（YAGNI）。

**不**使用 Web `navigator.clipboard.writeText`：项目已有插件依赖、服务层约定
与权限体系，Web API 在 WebView 内的可靠性与一致性不如插件路径。

### 3.4 国际化 — `src/locales/{zh-CN,zh-TW,en-US}.ts`

新增 `messages.urlCopied`：

| 语言 | 文案 |
| --- | --- |
| zh-CN | 链接已复制 |
| zh-TW | 連結已複製 |
| en-US | Link copied |

按钮 title 复用既有 `common.copy`（复制 / 複製 / Copy），不新增 key。

## 4. 顺带修复：剪贴板监控的读权限缺失

排查发现 `capabilities/default.json` 中**没有任何 clipboard-manager 权限**。
现有剪贴板监控功能（`useClipboardWatcher` → `clipboardService.readText`）的
调用失败会被 `console.debug` 静默吞掉——即该功能目前极可能从未生效。

本次补上 `allow-read-text`，使监控功能恢复设计预期行为。此为一行配置的
顺带修复，不展开为独立任务；commit message 中单独说明。

## 5. 范围边界（明确不做）

- ❌ 「在浏览器打开」按钮（用户已确认本次只做复制）
- ❌ 错误信息区块的复制
- ❌ URL 截断/折叠展示优化
- ❌ hover 才显示按钮（按钮常显，符合侧栏信息的可发现性）

## 6. 测试与验证

项目测试惯例是纯函数 co-located 测试（`parseLinks.test.ts` 等），无组件挂载
测试基建。本次改动为 UI 接线 + 服务薄封装，无可测纯函数，故：

- 不新增单元测试
- `npm run type-check` 通过
- `npm test` 确认不破坏现有测试
- 手动验证：`npm run tauri dev` → 打开任一任务详情 → 点复制按钮 →
  确认图标态切换、toast、剪贴板内容正确；确认剪贴板监控开关打开后
  粘贴 URL 能被检测（验证 §4 修复）

## 7. 变更清单

| 文件 | 变更 |
| --- | --- |
| `src-tauri/capabilities/default.json` | + `clipboard-manager:allow-write-text`、`clipboard-manager:allow-read-text` |
| `src/services/clipboardService.ts` | + `writeText()` |
| `src/components/task/TaskDetailPanel.vue` | URL 标题行加复制按钮 + 复制逻辑 + 计时器清理 |
| `src/locales/zh-CN.ts` / `zh-TW.ts` / `en-US.ts` | + `messages.urlCopied` |
