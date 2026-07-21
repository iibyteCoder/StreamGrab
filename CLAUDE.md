# StreamGrab

> 基于 Tauri 2.0 + Vue 3 的现代视频流下载器 GUI 应用

## 项目简介

将 N_m3u8DL-RE 与 FFmpeg 封装为一个功能完善、界面优美的桌面应用：N_m3u8DL-RE 引擎处理 HLS/DASH/MSS 流媒体，FFmpeg 引擎处理 HTTP 直链视频。用户无需选择工具——按 URL 类型自动分派（策略模式，详见 `docs/design/07-tool-config-architecture.md`）。

## 技术栈

- **前端**: Vue 3 + TypeScript + TailwindCSS + Shadcn-Vue
- **状态管理**: Pinia
- **构建工具**: Vite 5
- **桌面框架**: Tauri 2.0
- **后端**: Rust

## 项目结构

```
src/
├── domain/              # 领域类型唯一来源（与后端 JSON 契约一一对应）
├── components/          # UI 组件
│   ├── common/         # 通用组件 (AppIcon, UrlDuplicateDialog...)
│   ├── task/           # 任务相关组件（TaskCard, AddTaskDialog...）
│   ├── settings/       # 设置组件（tabs/ 4 标签页 + ToolManagerCard + sections/）
│   ├── stream/         # 流选择器
│   └── ui/             # shadcn-vue 基础组件
├── composables/        # 组合式函数（useDownloader 含队列+定时调度器）
├── stores/             # Pinia 状态管理（缓存层：task/settings/preset/history）
├── services/           # 服务层（与后端命令组一一对应的 invoke 封装）
├── utils/              # 工具函数（format/validate/url/id）
├── locales/            # i18n 三语（zh-CN/en-US/zh-TW）
└── views/              # 页面视图（Home/Settings/History）

src-tauri/src/           # Rust 后端（四层架构）
├── app/                 # 应用层：commands/（按域分组的瘦命令）+ tray.rs
├── domain/              # 领域层：config / task（状态机）/ download（DownloadEngine 策略契约）/ media
├── infrastructure/      # 基础设施：engines/（nm3u8dl + ffmpeg 策略实现）/ db（schema v4 + repository）/ process / tools / media / platform / fs
└── shared/              # 共享错误类型（AppError，thiserror）
```

## 开发命令

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 前端单元测试（vitest）
npm test

# 后端测试 / clippy
cd src-tauri && cargo test && cargo clippy -- -D warnings
```

## 核心设计原则

### 1. 渐进式披露 (Progressive Disclosure)

所有 UI 必须遵循三层信息模型：

| 层级    | 可见性   | 内容                         |
| ------- | -------- | ---------------------------- |
| Level 1 | 始终可见 | URL 输入、下载按钮、任务状态 |
| Level 2 | 悬停展开 | 任务详情、快速操作           |
| Level 3 | 导航进入 | 完整设置、流选择器           |

**规则**：默认只显示 Level 1，避免信息过载。

### 2. 80/20 法则

- 80% 用户只用 20% 功能
- 优先展示核心功能（URL 输入、开始下载、任务列表）
- 高级设置放入独立面板

### 3. 功能性动画

- 只为反馈使用，不为装饰
- 时长：100-200ms
- 缓动：ease-out

## 代码规范

### Vue 组件

```vue
<script setup lang="ts">
// ✅ 使用 Composition API + <script setup>
interface Props {
  title: string;
}
const props = defineProps<Props>();

const emit = defineEmits<{ (e: "click"): void }>();
</script>
```

### TypeScript

```typescript
// ✅ 显式类型定义
interface Task {
  id: string;
  url: string;
}

// ❌ 禁止 any 类型
function process(data: any) {} // 禁止
```

### 文件命名

| 类型       | 规范           | 示例               |
| ---------- | -------------- | ------------------ |
| 组件       | PascalCase.vue | `TaskCard.vue`     |
| 组合式函数 | camelCase.ts   | `useDownloader.ts` |
| Store      | camelCase.ts   | `taskStore.ts`     |

## 架构规则

### 状态管理

```typescript
// Store 结构：状态 → 计算属性 → 方法 → 返回
export const useTaskStore = defineStore('task', () => {
  const tasks = ref<Task[]>([])
  const activeTasks = computed(() => tasks.value.filter(...))

  function addTask(url: string) { /* ... */ }

  return { tasks, activeTasks, addTask }
})
```

### 服务层

- 组件/Store 不直接调用 Tauri API，统一经 `src/services/`（`invokeTauri`/`subscribeToEvent` 封装）
- 每个 service 与后端一个命令域对应：task / download / settings / preset / history / tools / system

### 三层配置模型

- 全局默认（设置中心 → `app_settings` + `tool_settings` 表，按工具分离管理）
- 任务级覆盖（`TaskOverrides`，随任务持久化，非空覆盖优先于全局默认）
- **命令行参数由后端引擎构建**（`infrastructure/engines/<tool>/args.rs`）——前端不持有任何工具的 CLI 知识；新增下载工具见 `07-tool-config-architecture.md` 的五步扩展契约

### 错误处理（后端）

- 基础设施与领域层统一 `AppResult<T>`（`AppError`，thiserror）
- 仅命令层边界转换为 `Result<T, String>`（Tauri 前端契约）

## UI 规范

### 色彩

```css
:root {
  --bg-base: #0a0a0a;
  --bg-surface: #111111;
  --text-primary: #fafafa;
  --text-secondary: #888888;
  --accent-primary: #3b82f6;
  --accent-success: #22c55e;
  --accent-error: #ef4444;
}
```

### 间距

- 组件内: 8px / 12px
- 组件间: 16px / 24px
- 页面边距: 24px

### 任务卡片

- 默认：只显示文件名、进度、速度（紧凑）
- 悬停：展开显示大小、剩余时间
- 点击：显示完整详情

## 禁止事项

```
❌ 使用 any 类型
❌ 组件直接调用 Tauri 命令（应通过 Service）
❌ 模板中使用复杂表达式
❌ Store 外部直接修改状态
❌ 忽略 TypeScript 错误
❌ 组件销毁后不清理副作用
❌ 一次性显示过多信息
```

## 提交规范

```
feat: 新功能
fix: 修复 bug
docs: 文档更新
refactor: 重构
perf: 性能优化
chore: 构建/工具
```

## 设计文档

详细设计文档位于 `docs/design/`:

- `00-overview.md` - 项目概述
- `01-tech-stack.md` - 技术选型
- `02-features.md` - 功能规格
- `03-ui-design.md` - 界面设计
- `04-architecture.md` - 项目架构
- `05-development-plan.md` - 开发计划
- `06-feature-status.md` - **功能实现状态追踪**
- `07-tool-config-architecture.md` - **工具架构与配置体系**（2026-07 重构设计：引擎策略、三层配置、schema v4）

## 任务追踪规则

**重要**: 完成任何功能后，必须更新 `docs/design/06-feature-status.md` 中的状态：

| 状态 | 符号 | 使用场景 |
| --- | --- | --- |
| 已完成 | `[x]` | 功能已实现并可用 |
| 进行中 | `[/]` | 正在开发或部分完成 |
| 计划中 | `[ ]` | 完全未开始 |
| 暂不实现 | `[-]` | 明确暂不开发 |

状态更新示例：

```markdown
| 单链接输入 | P0 | `[x]` | `src/components/task/AddTaskDialog.vue` | 带类型徽章的闭环输入 |
```

## 参考文档

- [n_m3u8dl-re-reference.md](docs/n_m3u8dl-re-reference.md) - N_m3u8DL-RE 命令行工具参数参考
- [RELEASE_NOTES_TEMPLATE.md](docs/RELEASE_NOTES_TEMPLATE.md) - **发行说明格式规范**
