# 项目架构设计

## 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            应用架构图                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                      渲染进程 (WebView)                          │   │
│   │  ┌────────────────────────────────────────────────────────────┐ │   │
│   │  │                      视图层 (Views)                         │ │   │
│   │  │    HomePage  │  SettingsPage  │  HistoryPage  │  AboutPage │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   │                                 │                               │   │
│   │  ┌──────────────────────────────┴─────────────────────────────┐ │   │
│   │  │                    组件层 (Components)                      │ │   │
│   │  │  UrlInput │ TaskQueue │ TaskCard │ SettingsPanel │ ...     │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   │                                 │                               │   │
│   │  ┌──────────────────────────────┴─────────────────────────────┐ │   │
│   │  │                  组合式函数 (Composables)                   │ │   │
│   │  │  useDownloader │ useTasks │ useSettings │ useTheme │ ...   │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   │                                 │                               │   │
│   │  ┌──────────────────────────────┴─────────────────────────────┐ │   │
│   │  │                   状态管理 (Pinia Stores)                   │ │   │
│   │  │     taskStore     │    settingsStore    │    uiStore       │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   │                                 │                               │   │
│   │  ┌──────────────────────────────┴─────────────────────────────┐ │   │
│   │  │                    服务层 (Services)                        │ │   │
│   │  │   downloaderService   │   parserService   │   fileService  │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   │                                 │                               │   │
│   │  ┌──────────────────────────────┴─────────────────────────────┐ │   │
│   │  │                    Tauri API Layer                          │ │   │
│   │  │         invoke()    │    listen()    │    emit()           │ │   │
│   │  └──────────────────────────────┬─────────────────────────────┘ │   │
│   └──────────────────────────────────┼──────────────────────────────┘   │
│                                      │                                  │
│                              IPC 通信 │                                  │
│                                      ▼                                  │
│   ┌──────────────────────────────────────────────────────────────────┐  │
│   │                        主进程 (Rust)                              │  │
│   │  ┌─────────────────────────────────────────────────────────────┐ │  │
│   │  │                     Commands (Tauri)                        │ │  │
│   │  │  start_download │ stop_download │ get_config │ save_config │ │  │
│   │  └──────────────────────────────┬──────────────────────────────┘ │  │
│   │                                 │                                │  │
│   │  ┌──────────────────────────────┴──────────────────────────────┐ │  │
│   │  │                    Process Manager                           │ │  │
│   │  │        管理子进程生命周期，解析输出流                         │ │  │
│   │  └──────────────────────────────┬──────────────────────────────┘ │  │
│   │                                 │                                │  │
│   │                                 ▼                                │  │
│   │                    N_m3u8DL-RE.exe (子进程)                      │  │
│   └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 目录结构

```
m3u8-downloader-pro/
│
├── 📁 src/                          # 前端源码
│   │
│   ├── 📁 main.ts                   # 应用入口
│   ├── 📁 App.vue                   # 根组件
│   │
│   ├── 📁 assets/                   # 静态资源
│   │   ├── 📁 images/
│   │   └── 📁 fonts/
│   │
│   ├── 📁 components/               # 组件
│   │   │
│   │   ├── 📁 common/               # 通用组件
│   │   │   ├── Button.vue
│   │   │   ├── Input.vue
│   │   │   ├── Select.vue
│   │   │   ├── Switch.vue
│   │   │   ├── Modal.vue
│   │   │   ├── Tooltip.vue
│   │   │   ├── Toast.vue
│   │   │   └── index.ts             # 统一导出
│   │   │
│   │   ├── 📁 layout/               # 布局组件
│   │   │   ├── TitleBar.vue         # 标题栏
│   │   │   ├── Sidebar.vue          # 侧边栏
│   │   │   └── MainLayout.vue       # 主布局
│   │   │
│   │   ├── 📁 input/                # 输入相关
│   │   │   ├── UrlInput.vue         # URL 输入框
│   │   │   ├── BatchImport.vue      # 批量导入
│   │   │   └── DropZone.vue         # 拖放区域
│   │   │
│   │   ├── 📁 task/                 # 任务相关
│   │   │   ├── TaskQueue.vue        # 任务队列
│   │   │   ├── TaskCard.vue         # 任务卡片
│   │   │   ├── TaskDetail.vue       # 任务详情
│   │   │   ├── ProgressBar.vue      # 进度条
│   │   │   └── TaskActions.vue      # 任务操作按钮
│   │   │
│   │   ├── 📁 settings/             # 设置相关
│   │   │   ├── SettingsPanel.vue    # 设置面板
│   │   │   ├── SettingsModal.vue    # 设置弹窗
│   │   │   ├── BasicSettings.vue    # 基础设置
│   │   │   ├── StreamSelector.vue   # 流选择器
│   │   │   ├── DecryptSettings.vue  # 解密设置
│   │   │   ├── ProxySettings.vue    # 代理设置
│   │   │   ├── LiveSettings.vue     # 直播设置
│   │   │   └── MuxSettings.vue      # 混流设置
│   │   │
│   │   └── 📁 ui/                   # Shadcn-Vue 组件
│   │       ├── button/
│   │       ├── input/
│   │       ├── select/
│   │       ├── dialog/
│   │       └── ...
│   │
│   ├── 📁 views/                    # 页面视图
│   │   ├── HomeView.vue             # 主页
│   │   ├── SettingsView.vue         # 设置页
│   │   ├── HistoryView.vue          # 历史记录
│   │   └── AboutView.vue            # 关于页
│   │
│   ├── 📁 composables/              # 组合式函数
│   │   ├── useDownloader.ts         # 下载器逻辑
│   │   ├── useTasks.ts              # 任务管理
│   │   ├── useSettings.ts           # 设置管理
│   │   ├── useTheme.ts              # 主题切换
│   │   ├── useToast.ts              # Toast 提示
│   │   └── useClipboard.ts          # 剪贴板
│   │
│   ├── 📁 stores/                   # Pinia 状态
│   │   ├── index.ts                 # Store 入口
│   │   ├── taskStore.ts             # 任务状态
│   │   ├── settingsStore.ts         # 设置状态
│   │   └── uiStore.ts               # UI 状态
│   │
│   ├── 📁 services/                 # 服务层
│   │   ├── downloader.ts            # 下载服务
│   │   ├── parser.ts                # 输出解析
│   │   ├── file.ts                  # 文件操作
│   │   └── tauri.ts                 # Tauri API 封装
│   │
│   ├── 📁 types/                    # TypeScript 类型
│   │   ├── task.ts                  # 任务类型
│   │   ├── settings.ts              # 设置类型
│   │   ├── stream.ts                # 流信息类型
│   │   └── progress.ts              # 进度类型
│   │
│   ├── 📁 utils/                    # 工具函数
│   │   ├── commandBuilder.ts        # 命令构建器
│   │   ├── format.ts                # 格式化工具
│   │   ├── validate.ts              # 验证工具
│   │   └── constants.ts             # 常量定义
│   │
│   ├── 📁 router/                   # 路由配置
│   │   └── index.ts
│   │
│   └── 📁 styles/                   # 全局样式
│       ├── main.css                 # 主样式入口
│       ├── variables.css            # CSS 变量
│       └── animations.css           # 动画样式
│
├── 📁 src-tauri/                    # Tauri 后端
│   ├── 📁 src/
│   │   ├── main.rs                  # 主入口
│   │   ├── lib.rs                   # 库入口
│   │   │
│   │   ├── 📁 commands/             # Tauri 命令
│   │   │   ├── mod.rs
│   │   │   ├── download.rs          # 下载相关命令
│   │   │   ├── config.rs            # 配置相关命令
│   │   │   └── system.rs            # 系统相关命令
│   │   │
│   │   ├── 📁 process/              # 进程管理
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs           # 进程管理器
│   │   │   └── parser.rs            # 输出解析
│   │   │
│   │   └── 📁 utils/                # Rust 工具
│   │       ├── mod.rs
│   │       └── path.rs              # 路径工具
│   │
│   ├── Cargo.toml                   # Rust 依赖
│   └── tauri.conf.json              # Tauri 配置
│
├── 📁 public/                       # 公共资源
│   └── favicon.ico
│
├── index.html                       # HTML 入口
├── package.json                     # Node 依赖
├── pnpm-lock.yaml                   # pnpm 锁文件
├── tsconfig.json                    # TypeScript 配置
├── vite.config.ts                   # Vite 配置
├── tailwind.config.js               # TailwindCSS 配置
├── postcss.config.js                # PostCSS 配置
├── .eslintrc.cjs                    # ESLint 配置
├── .prettierrc                      # Prettier 配置
└── README.md                        # 项目说明
```

---

## 核心类型定义

### types/task.ts

```typescript
// 任务状态
export type TaskStatus =
  | "pending" // 等待中
  | "analyzing" // 解析中
  | "downloading" // 下载中
  | "paused" // 已暂停
  | "merging" // 合并中
  | "muxing" // 混流中
  | "completed" // 已完成
  | "failed" // 失败
  | "cancelled"; // 已取消

// 任务定义
export interface Task {
  id: string;
  url: string;
  name: string;
  status: TaskStatus;
  createdAt: Date;
  startedAt?: Date;
  completedAt?: Date;

  // 进度信息
  progress: TaskProgress;

  // 流信息
  streams?: StreamInfo;

  // 配置
  config: TaskConfig;

  // 错误信息
  error?: string;

  // 输出路径
  outputPath?: string;
}

// 进度信息
export interface TaskProgress {
  downloadedSegments: number;
  totalSegments: number;
  percentage: number;

  speed: number; // bytes/s
  speedFormatted: string;

  downloadedBytes: number;
  totalBytes: number;

  elapsedTime: number; // seconds
  estimatedTime: number; // seconds

  currentAction: string;
}

// 任务配置
export interface TaskConfig {
  saveDir: string;
  saveName: string;
  threadCount: number;
  retryCount: number;
  timeout: number;
  maxSpeed: string;

  // 流选择
  autoSelect: boolean;
  selectVideo?: string;
  selectAudio?: string;
  selectSubtitle?: string;

  // 混流
  muxFormat?: "mp4" | "mkv";
  muxAfterDone: boolean;

  // 其他选项
  skipMerge: boolean;
  delAfterDone: boolean;
  checkSegmentsCount: boolean;

  // 高级
  headers?: HeaderConfig[];
  proxy?: string;
  key?: string;
}
```

### types/settings.ts

```typescript
// 全局设置
export interface Settings {
  // 基础设置
  general: GeneralSettings;

  // 下载设置
  download: DownloadSettings;

  // 混流设置
  mux: MuxSettings;

  // 网络设置
  network: NetworkSettings;

  // 直播设置
  live: LiveSettings;

  // UI 设置
  ui: UISettings;
}

export interface GeneralSettings {
  saveDir: string;
  tmpDir: string;
  language: "zh-CN" | "en-US";
  autoStartDownload: boolean;
  confirmBeforeDelete: boolean;
}

export interface DownloadSettings {
  threadCount: number;
  retryCount: number;
  retryDelay: number;
  timeout: number;
  maxSpeed: number;

  autoSelect: boolean;
  selectVideo: string;
  selectAudio: string;
  selectSubtitle: string;

  checkSegmentsCount: boolean;
  delAfterDone: boolean;
  skipMerge: boolean;
  writeMetaJson: boolean;
}

export interface MuxSettings {
  format: "mp4" | "mkv";
  muxer: "ffmpeg" | "mkvmerge";
  ffmpegPath: string;
  mkvmergePath: string;
  keepOriginal: boolean;
  skipSubtitles: boolean;
}

export interface NetworkSettings {
  useSystemProxy: boolean;
  customProxy: string;
  headers: HeaderConfig[];
}

export interface LiveSettings {
  performAsVod: boolean;
  realTimeMerge: boolean;
  keepSegments: boolean;
  pipeMux: boolean;
  recordLimit: string;
  waitTime: number;
  takeCount: number;
}

export interface UISettings {
  theme: "light" | "dark" | "system";
  minimizeToTray: boolean;
  showNotification: boolean;
  clipboardWatch: boolean;
}

export interface HeaderConfig {
  key: string;
  value: string;
  enabled: boolean;
}
```

### types/stream.ts

```typescript
// 流信息
export interface StreamInfo {
  videos: VideoStream[];
  audios: AudioStream[];
  subtitles: SubtitleStream[];

  duration: number;
  segmentCount: number;
  isLive: boolean;
  isEncrypted: boolean;
}

export interface BaseStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string;
  selected?: boolean;
}

export interface VideoStream extends BaseStream {
  resolution: string;
  width: number;
  height: number;
  frameRate: number;
  videoRange: "SDR" | "HDR10" | "HDR10+" | "DV" | "HLG";
}

export interface AudioStream extends BaseStream {
  channels: string;
  sampleRate: number;
  isDefault: boolean;
}

export interface SubtitleStream extends BaseStream {
  format: "srt" | "vtt" | "ttml";
  isDefault: boolean;
  isForced: boolean;
}
```

---

## 核心服务设计

### services/downloader.ts

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Task, TaskProgress, StreamInfo } from "@/types";

// 事件类型
type DownloadEvent =
  | { type: "progress"; data: TaskProgress }
  | { type: "completed"; data: { outputPath: string } }
  | { type: "error"; data: { message: string } }
  | { type: "log"; data: { level: string; message: string } };

// 下载服务
export class DownloaderService {
  private unlisteners: (() => void)[] = [];

  // 开始下载
  async startDownload(task: Task): Promise<void> {
    await invoke("start_download", { task });
  }

  // 停止下载
  async stopDownload(taskId: string): Promise<void> {
    await invoke("stop_download", { taskId });
  }

  // 解析 URL
  async parseUrl(url: string): Promise<StreamInfo> {
    return await invoke("parse_url", { url });
  }

  // 订阅下载事件
  async subscribe(
    taskId: string,
    callback: (event: DownloadEvent) => void,
  ): Promise<() => void> {
    const unlisten = await listen<DownloadEvent>(
      `download:${taskId}`,
      (event) => {
        callback(event.payload);
      },
    );

    this.unlisteners.push(unlisten);
    return unlisten;
  }

  // 清理所有监听器
  cleanup(): void {
    this.unlisteners.forEach((unlisten) => unlisten());
    this.unlisteners = [];
  }
}

export const downloaderService = new DownloaderService();
```

### utils/commandBuilder.ts

```typescript
import type { Task, TaskConfig, Settings } from "@/types";

// 构建命令行参数
export function buildCommandArgs(task: Task, settings: Settings): string[] {
  const args: string[] = [task.url];

  const config = { ...settings.download, ...task.config } as TaskConfig;

  // 基础参数
  if (config.saveDir) {
    args.push("--save-dir", config.saveDir);
  }
  if (config.saveName) {
    args.push("--save-name", config.saveName);
  }
  if (config.tmpDir) {
    args.push("--tmp-dir", config.tmpDir);
  }

  // 下载参数
  if (config.threadCount) {
    args.push("--thread-count", String(config.threadCount));
  }
  if (config.retryCount) {
    args.push("--download-retry-count", String(config.retryCount));
  }
  if (config.timeout) {
    args.push("--http-request-timeout", String(config.timeout));
  }
  if (config.maxSpeed && config.maxSpeed !== "0") {
    args.push("-R", config.maxSpeed);
  }

  // 流选择
  if (config.autoSelect) {
    args.push("--auto-select");
  }
  if (config.selectVideo) {
    args.push("-sv", config.selectVideo);
  }
  if (config.selectAudio) {
    args.push("-sa", config.selectAudio);
  }
  if (config.selectSubtitle) {
    args.push("-ss", config.selectSubtitle);
  }

  // 混流
  if (config.muxFormat && config.muxAfterDone) {
    const muxOptions = buildMuxOptions(settings.mux);
    args.push("-M", muxOptions);
  }

  // 网络设置
  if (settings.network.useSystemProxy) {
    args.push("--use-system-proxy");
  } else if (settings.network.customProxy) {
    args.push("--custom-proxy", settings.network.customProxy);
  }

  // 请求头
  for (const header of settings.network.headers.filter((h) => h.enabled)) {
    args.push("-H", `${header.key}: ${header.value}`);
  }

  // 其他选项
  if (config.skipMerge) {
    args.push("--skip-merge");
  }
  if (!config.delAfterDone) {
    args.push("--del-after-done", "false");
  }
  if (!config.checkSegmentsCount) {
    args.push("--check-segments-count", "false");
  }

  // 解密
  if (config.key) {
    args.push("--key", config.key);
  }

  return args;
}

function buildMuxOptions(muxSettings: Settings["mux"]): string {
  const parts = [`format=${muxSettings.format}`, `muxer=${muxSettings.muxer}`];

  if (muxSettings.ffmpegPath && muxSettings.muxer === "ffmpeg") {
    parts.push(`bin_path=${muxSettings.ffmpegPath}`);
  }
  if (muxSettings.mkvmergePath && muxSettings.muxer === "mkvmerge") {
    parts.push(`bin_path=${muxSettings.mkvmergePath}`);
  }
  if (muxSettings.keepOriginal) {
    parts.push("keep=true");
  }
  if (muxSettings.skipSubtitles) {
    parts.push("skip_sub=true");
  }

  return parts.join(":");
}
```

---

## 状态管理设计

### stores/taskStore.ts

```typescript
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Task, TaskStatus } from "@/types";
import { generateId } from "@/utils/helpers";

export const useTaskStore = defineStore("task", () => {
  // 状态
  const tasks = ref<Task[]>([]);
  const maxConcurrent = ref(3);

  // 计算属性
  const activeTasks = computed(() =>
    tasks.value.filter((t) =>
      ["downloading", "analyzing", "merging", "muxing"].includes(t.status),
    ),
  );

  const pendingTasks = computed(() =>
    tasks.value.filter((t) => t.status === "pending"),
  );

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "completed"),
  );

  const canStartMore = computed(
    () => activeTasks.value.length < maxConcurrent.value,
  );

  // 操作
  function addTask(url: string, name?: string): Task {
    const task: Task = {
      id: generateId(),
      url,
      name: name || extractNameFromUrl(url),
      status: "pending",
      createdAt: new Date(),
      progress: createEmptyProgress(),
      config: {} as TaskConfig,
    };

    tasks.value.push(task);
    return task;
  }

  function removeTask(taskId: string): void {
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index !== -1) {
      tasks.value.splice(index, 1);
    }
  }

  function updateTaskStatus(taskId: string, status: TaskStatus): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.status = status;

      if (status === "downloading" && !task.startedAt) {
        task.startedAt = new Date();
      }
      if (status === "completed") {
        task.completedAt = new Date();
      }
    }
  }

  function updateTaskProgress(
    taskId: string,
    progress: Partial<TaskProgress>,
  ): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.progress = { ...task.progress, ...progress };
    }
  }

  function clearCompleted(): void {
    tasks.value = tasks.value.filter((t) => t.status !== "completed");
  }

  return {
    // 状态
    tasks,
    maxConcurrent,

    // 计算属性
    activeTasks,
    pendingTasks,
    completedTasks,
    canStartMore,

    // 操作
    addTask,
    removeTask,
    updateTaskStatus,
    updateTaskProgress,
    clearCompleted,
  };
});
```

### stores/settingsStore.ts

```typescript
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import type { Settings } from "@/types";
import { defaultSettings } from "@/utils/constants";
import { invoke } from "@tauri-apps/api/core";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>(defaultSettings);
  const isLoading = ref(false);

  // 加载设置
  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    try {
      const loaded = await invoke<Settings>("load_settings");
      settings.value = { ...defaultSettings, ...loaded };
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      isLoading.value = false;
    }
  }

  // 保存设置
  async function saveSettings(): Promise<void> {
    try {
      await invoke("save_settings", { settings: settings.value });
    } catch (error) {
      console.error("Failed to save settings:", error);
      throw error;
    }
  }

  // 重置设置
  function resetSettings(): void {
    settings.value = defaultSettings;
  }

  // 更新部分设置
  function updateSettings(partial: Partial<Settings>): void {
    settings.value = { ...settings.value, ...partial };
  }

  // 自动保存
  watch(
    settings,
    () => {
      saveSettings();
    },
    { deep: true },
  );

  return {
    settings,
    isLoading,
    loadSettings,
    saveSettings,
    resetSettings,
    updateSettings,
  };
});
```

---

## Tauri 命令设计

### src-tauri/src/commands/download.rs

```rust
use tauri::{command, AppHandle, Emitter, State};
use std::sync::Mutex;
use crate::process::ProcessManager;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub downloaded_segments: u32,
    pub total_segments: u32,
    pub percentage: f32,
    pub speed: String,
    pub speed_bytes: u64,
}

#[command]
pub async fn start_download(
    task: crate::types::Task,
    app: AppHandle,
    manager: State<'_, Mutex<ProcessManager>>,
) -> Result<(), String> {
    let task_id = task.id.clone();

    // 构建命令参数
    let args = build_args(&task);

    // 启动子进程
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;

    mgr.start_process(
        task_id.clone(),
        "N_m3u8DL-RE.exe",
        args,
        move |line: String| {
            // 解析输出并发送事件
            if let Some(progress) = parse_progress(&line) {
                let _ = app.emit(&format!("download:{}", task_id), progress);
            }
        }
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn stop_download(
    task_id: String,
    manager: State<'_, Mutex<ProcessManager>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.stop_process(&task_id).map_err(|e| e.to_string())
}

#[command]
pub async fn parse_url(url: String) -> Result<crate::types::StreamInfo, String> {
    // 使用 N_m3u8DL-RE 的 --skip-download 模式解析
    // 返回流信息
    todo!()
}

fn build_args(task: &crate::types::Task) -> Vec<String> {
    let mut args = vec![task.url.clone()];

    // 根据任务配置构建参数
    if let Some(ref name) = task.config.save_name {
        args.push("--save-name".to_string());
        args.push(name.clone());
    }

    // ... 更多参数

    args
}

fn parse_progress(line: &str) -> Option<DownloadProgress> {
    // 解析 N_m3u8DL-RE 的输出
    // 示例: "Downloaded: 156/234 (67%) Speed: 5.2MB/s"
    todo!()
}
```

---

## 数据流图

```
用户操作                    前端处理                   后端处理
   │                          │                          │
   │  1. 输入 URL             │                          │
   ├─────────────────────────▶│                          │
   │                          │                          │
   │                          │  2. 调用 parse_url       │
   │                          ├─────────────────────────▶│
   │                          │                          │
   │                          │                          │  3. 执行解析
   │                          │                          ├──────▶
   │                          │                          │
   │                          │  4. 返回 StreamInfo      │
   │                          │◀─────────────────────────┤
   │                          │                          │
   │  5. 显示流选择           │                          │
   │◀─────────────────────────┤                          │
   │                          │                          │
   │  6. 确认下载             │                          │
   ├─────────────────────────▶│                          │
   │                          │                          │
   │                          │  7. 创建 Task            │
   │                          ├──────▶                   │
   │                          │                          │
   │                          │  8. start_download       │
   │                          ├─────────────────────────▶│
   │                          │                          │
   │                          │                          │  9. 启动子进程
   │                          │                          ├──────▶
   │                          │                          │
   │                          │  10. 进度事件 (持续)     │
   │                          │◀═════════════════════════┤
   │                          │                          │
   │  11. 更新 UI             │                          │
   │◀═════════════════════════┤                          │
   │                          │                          │
   │                          │  12. 完成事件            │
   │                          │◀─────────────────────────┤
   │                          │                          │
   │  13. 显示完成            │                          │
   │◀─────────────────────────┤                          │
```

---

## 配置存储设计

配置文件存储在用户目录下：

```
Windows: %APPDATA%/m3u8-downloader-pro/
macOS: ~/Library/Application Support/m3u8-downloader-pro/
Linux: ~/.config/m3u8-downloader-pro/

文件结构:
├── config.json        # 主配置文件
├── history.json       # 历史记录
├── templates.json     # 配置模板
└── logs/
    └── app.log        # 应用日志
```

