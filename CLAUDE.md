# M3U8 Downloader Pro

> 基于 Tauri 2.0 + Vue 3 的现代视频流下载器 GUI 应用

## 项目简介

将 N_m3u8DL-RE 命令行工具封装为一个功能完善、界面优美的桌面应用，支持 HLS/DASH/MSS 流媒体下载。

## 技术栈

- **前端**: Vue 3 + TypeScript + TailwindCSS + Shadcn-Vue
- **状态管理**: Pinia
- **构建工具**: Vite 5
- **桌面框架**: Tauri 2.0
- **后端**: Rust

## 项目结构

```
src/
├── components/          # UI 组件
│   ├── common/         # 通用组件 (Button, Input, Modal...)
│   ├── task/           # 任务相关组件
│   └── settings/       # 设置相关组件
├── composables/        # 组合式函数
├── stores/             # Pinia 状态管理
├── services/           # 服务层 (API 调用封装)
├── types/              # TypeScript 类型定义
├── utils/              # 工具函数
└── views/              # 页面视图

src-tauri/              # Tauri 后端
├── src/
│   ├── commands/       # Tauri 命令
│   └── process/        # 进程管理
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

- 组件不直接调用 Tauri API
- 通过 Service 层封装
- 使用事件订阅机制处理实时数据

### 命令行参数构建

```typescript
// utils/commandBuilder.ts
export function buildCommandArgs(task: Task, settings: Settings): string[];
```

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
| 单链接输入 | P0 | `[x]` | `src/components/input/UrlInput.vue` | 带验证的输入框 |
```

## 参考文档

- [n_m3u8dl-re-reference.md](docs/n_m3u8dl-re-reference.md) - N_m3u8DL-RE 命令行工具参数参考
- [RELEASE_NOTES_TEMPLATE.md](docs/RELEASE_NOTES_TEMPLATE.md) - **发行说明格式规范**
