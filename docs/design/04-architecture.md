# 项目架构设计

> 2026-07 完全重构后的架构。引擎策略与配置体系的详细设计见 `07-tool-config-architecture.md`。

## 整体架构

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                        StreamGrab 应用架构                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                      渲染进程 (WebView)                          │   │
│   │                                                                   │   │
│   │   views/ ─▶ components/ ─▶ composables/ ─▶ stores/ ─▶ services/  │   │
│   │                                    │                      │       │   │
│   │                            domain/ (类型唯一来源)     invoke()    │   │
│   └────────────────────────────────────────────────────┼──────────────┘   │
│                                                        │ IPC              │
│                                                        ▼                  │
│   ┌──────────────────────────────────────────────────────────────────┐    │
│   │                    主进程 (Rust · 四层架构)                       │    │
│   │                                                                   │    │
│   │   ┌─── app/ ─────────────────────────────────────────────────┐   │    │
│   │   │  commands/ (tasks · download · settings · presets ·      │   │    │
│   │   │             history · tools · system)                    │   │    │
│   │   │  tray.rs                                                 │   │    │
│   │   └──────────────────────┬───────────────────────────────────┘   │    │
│   │                          │                                       │    │
│   │   ┌─── domain/ ──────────┴───────────────────────────────────┐   │    │
│   │   │  config.rs     — 工具配置类型 (Nm3u8dlConfig/FfmpegConfig)│   │    │
│   │   │  task/         — 状态机 (TaskStatus + can_transition_to) │   │    │
│   │   │  download/     — DownloadEngine trait + EngineRegistry   │   │    │
│   │   │  media.rs      — 媒体信息领域类型                        │   │    │
│   │   └──────────────────────┬───────────────────────────────────┘   │    │
│   │                          │                                       │    │
│   │   ┌─── infrastructure/ ──┴───────────────────────────────────┐   │    │
│   │   │  engines/nm3u8dl/ — args.rs + parser.rs (策略实现)       │   │    │
│   │   │  engines/ffmpeg/  — args.rs + parser.rs (策略实现)       │   │    │
│   │   │  db/            — schema v4 + repository/ (6 repos)      │   │    │
│   │   │  process/       — ProcessManager (State 注入)            │   │    │
│   │   │  tools/         — ToolDetector (二进制检测/版本/下载)    │   │    │
│   │   │  media/         — ffprobe.rs (媒体探测)                  │   │    │
│   │   │  platform/ fs/  — 平台适配 / 文件系统                    │   │    │
│   │   └──────────────────────────────────────────────────────────┘   │    │
│   │                                                                   │    │
│   │   ┌─── shared/ ──────────────────────────────────────────────┐   │    │
│   │   │  error.rs — AppError (thiserror) + AppResult<T>          │   │    │
│   │   └──────────────────────────────────────────────────────────┘   │    │
│   └──────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 后端四层架构

### app/（应用层）

- **commands/**：按域分组的瘦命令，仅做参数校验 → 调用领域/基础设施 → 错误转换（`AppResult` → `Result<T, String>`）
  - `tasks.rs` — CRUD + 状态迁移
  - `download.rs` — start/stop/parse_url
  - `settings.rs` — load/patch_app_settings/patch_tool_settings
  - `presets.rs` — CRUD
  - `history.rs` — load/delete/clear
  - `tools.rs` — detect/download_update
  - `system.rs` — open_dir/get_file_info
- **tray.rs**：系统托盘（最小化/菜单/单击恢复）

### domain/（领域层）

- **config.rs**：工具配置类型（`Nm3u8dlConfig`、`FfmpegConfig`、`AppSettings`、`ToolConfigs`）
- **task/**：`TaskStatus` 状态机 + `can_transition_to` 迁移校验
- **download/**：`DownloadEngine` trait（策略契约）+ `EngineRegistry`（分派）+ `UrlType`
- **media.rs**：`StreamInfo`、`MediaInfo` 等领域类型

### infrastructure/（基础设施层）

- **engines/**：策略实现（`Nm3u8dlEngine`、`FfmpegEngine`），每个引擎自包含 `args.rs`（参数构建）+ `parser.rs`（输出解析 + `EngineSession`）
- **db/**：SQLite schema v4 + `repository/`（task_repo / settings_repo / preset_repo / history_repo / progress_repo）
- **process/**：`ProcessManager`——`Arc<tokio::sync::Mutex<_>>` 经 Tauri State 注入
- **tools/**：`ToolDetector`（二进制存在性检测 / 版本获取 / GitHub Release 下载更新）
- **media/**：`ffprobe.rs`（HTTP 直链媒体探测）
- **platform/** / **fs/**：平台适配与文件系统操作

### shared/（共享层）

- **error.rs**：`AppError`（thiserror 枚举：Database/Process/ToolNotFound/Config/Io/Parse/Serialization/Http/Other）
- `From<rusqlite::Error>` / `io::Error` / `serde_json::Error` / `reqwest::Error` 自动转换
- 基础设施与领域层全部使用 `AppResult<T>`，仅命令层边界转 `Result<T, String>`

---

## 下载引擎策略（Strategy）

```text
URL 输入
    │
    ▼
EngineRegistry::for_url(UrlType)
    │
    ├── HLS/DASH/MSS/Unknown ──▶ Nm3u8dlEngine
    │                              ├── build_parse_args() → 解析流列表
    │                              ├── parse_streams()   → StreamInfo
    │                              ├── build_download_args() → 下载
    │                              └── new_session() → OutputParser (逐行解析)
    │
    └── HTTP 直链 ─────────────▶ FfmpegEngine
                                   ├── ffprobe 元数据探测
                                   ├── build_download_args() → 直链下载
                                   └── new_session() → ProgressParser (-progress pipe:2)
```

- **引擎自包含**：参数构建、输出解析、进度模型全部内聚于 `infrastructure/engines/<tool>/`
- **会话模式**：引擎实例全局共享无状态；逐任务跨行状态由 `EngineSession` 持有
- **自动分派**：用户无需选择工具，`Unknown` 回退 N_m3u8DL-RE（格式覆盖最广）
- **扩展**：新增工具五步清单详见 `07-tool-config-architecture.md`

---

## 三层配置模型

```text
全局默认（设置中心 → SQLite：app_settings + tool_settings）
        ↓  提供默认值
TaskOverrides（添加任务对话框 → tasks.overrides_json，随任务持久化）
        ↓  后端引擎合并（非空覆盖 > 全局默认）
命令行参数（引擎 args.rs 构建 → 子进程）
```

- 前端不持有任何工具的 CLI 知识
- `patch_*` 命令递归合并（后端深合并，前端只发增量）
- 详见 `07-tool-config-architecture.md` 第一节

---

## 数据层（schema v4）

| 表 | 设计 |
| --- | --- |
| `tasks` | 单表聚合：基础列 + `progress_json` / `media_info_json` / `overrides_json` 三个 JSON 列 |
| `progress_history` | 速率曲线时序数据（独立表，图表消费） |
| `history` | 任务终态快照（含 overrides 快照），清除任务不删除历史 |
| `app_settings` | 单行 JSON |
| `tool_settings` | `(tool_id TEXT PK, config_json TEXT)` — 通用工具配置表，新增工具零 DDL |
| `task_presets` | 命名的 TaskOverrides 组合（DB 持久化，取代旧 localStorage 模板） |

版本低于 4 时全量重建（DROP + CREATE，不保留旧数据）。

仓储层 `infrastructure/db/repository/`：

- `task_repo.rs` — tasks 表 CRUD + 状态迁移
- `settings_repo.rs` — app_settings + tool_settings 读写
- `preset_repo.rs` — task_presets CRUD
- `history_repo.rs` — history CRUD
- `progress_repo.rs` — progress_history 时序写入/查询

---

## 前端分层

```text
views/               页面视图（HomeView / SettingsView / HistoryView / LayoutView）
    │
components/          UI 组件
    ├── task/        任务组件（TaskCard / AddTaskDialog / ProgressChart / LogViewer ...）
    ├── settings/    设置组件（tabs/ 4 标签页 + ToolManagerCard + sections/ 子区块）
    ├── stream/      流选择器（StreamSelector / StreamList / StreamItem）
    ├── common/      通用组件（AppIcon / AppProgress / UrlDuplicateDialog / RestoreTasksDialog）
    └── ui/          shadcn-vue 基础组件
    │
composables/         组合式函数
    ├── useDownloader.ts    下载编排（队列 + 定时调度器）
    ├── useTasks.ts         任务操作
    ├── useStreamSelector.ts 流选择
    ├── usePresetManager.ts 预设管理
    ├── useClipboardWatcher.ts / useNotification.ts / useUpdateChecker.ts / useToast.ts
    │
stores/              Pinia 状态管理（缓存层）
    ├── taskStore.ts        任务列表（来源：后端 DB）
    ├── settingsStore.ts    全局设置（来源：app_settings + tool_settings）
    ├── presetStore.ts      预设（来源：task_presets）
    └── historyStore.ts     历史记录（来源：history）
    │
services/            Tauri invoke 封装（与后端命令组一一对应）
    ├── taskService.ts / downloadService.ts / settingsService.ts
    ├── presetService.ts / historyService.ts / toolsService.ts
    ├── systemService.ts / clipboardService.ts / updateService.ts
    └── tauri.ts           invokeTauri / subscribeToEvent 底层封装
    │
domain/              类型唯一来源（与后端 JSON 契约一一对应）
    ├── task.ts / config.ts / stream.ts / url.ts
    │
utils/               工具函数
    ├── format.ts / validate.ts / id.ts / cn.ts / constants.ts
```

---

## 进程管理

```text
ProcessManager (Arc<tokio::sync::Mutex<ProcessManager>>)
    │
    │  Tauri State 注入到所有命令
    │
    ├── start_process(task_id, bin, args, on_line)
    │       └── tokio::process::Command → stdout/stderr 读取线程
    │
    ├── stop_process(task_id)
    │       └── PID + taskkill /T 终止进程树
    │
    └── 孤儿进程双保险清理：
            ├── impl Drop → stop_all_sync()
            └── RunEvent::Exit hook → stop_all_sync()
```

- 子进程被杀后管道关闭，stdout/stderr 读取线程随 EOF 自然退出
- 逐任务进度推送通过 Tauri 事件（`download:<task_id>`）

---

## 状态机（TaskStatus）

```text
pending → analyzing → downloading → merging → muxing → completed
   │          │           │           │         │
   └──────────┴───────────┴───────────┴─────────┴──▶ failed
                                                   cancelled
                                                   paused (从活跃态)
终态 (completed/failed/cancelled) → pending (重试/重新下载)
```

- 迁移经 `can_transition_to` 校验
- 命令层 `update_task_status` 强制检查
- 进入终态自动写入历史快照（`history_repo`）

---

## 数据流图

```text
用户操作                    前端处理                   后端处理
   │                          │                          │
   │  1. 输入 URL             │                          │
   ├─────────────────────────▶│                          │
   │                          │                          │
   │                          │  2. 类型检测 (本地)      │
   │                          ├──────▶                   │
   │                          │                          │
   │                          │  3. parse_url (引擎分派) │
   │                          ├─────────────────────────▶│
   │                          │                          │
   │                          │                          │  4. EngineRegistry
   │                          │                          │     → Nm3u8dlEngine
   │                          │                          │       .build_parse_args()
   │                          │                          │     → 子进程 → parser
   │                          │                          ├──────▶
   │                          │                          │
   │                          │  5. StreamInfo           │
   │                          │◀─────────────────────────┤
   │                          │                          │
   │  6. 流选择 / 预设 /     │                          │
   │     TaskOverrides        │                          │
   │◀─────────────────────────┤                          │
   │                          │                          │
   │  7. 确认下载             │                          │
   ├─────────────────────────▶│                          │
   │                          │  8. start_download       │
   │                          ├─────────────────────────▶│
   │                          │                          │
   │                          │                          │  9. build_download_args()
   │                          │                          │     → 子进程
   │                          │                          ├──────▶
   │                          │                          │
   │                          │  10. 进度事件 (持续)     │
   │                          │◀═════════════════════════┤
   │                          │     (EngineSession 解析) │
   │  11. 更新 UI             │                          │
   │◀═════════════════════════┤                          │
   │                          │                          │
   │                          │  12. 终态 → 历史快照     │
   │                          │                          ├──────▶ history_repo
   │                          │                          │
   │  13. 完成                │                          │
   │◀─────────────────────────┤                          │
```

---

## 配置存储

SQLite 数据库存储在用户目录下：

```text
Windows: %APPDATA%/com.streamgrab.app/
macOS:   ~/Library/Application Support/com.streamgrab.app/
Linux:   ~/.config/com.streamgrab.app/

├── streamgrab.db       # SQLite 主数据库（schema v4，所有表）
└── logs/
    └── app.log         # 应用日志
```
