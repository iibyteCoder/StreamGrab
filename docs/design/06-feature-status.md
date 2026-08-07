# 功能实现状态

本文档追踪 StreamGrab 各功能模块的实现进度。

> **2026-07 完全重构**：架构细节与配置体系设计详见 `07-tool-config-architecture.md`。
> 本次重构 schema v4 全量重建，不保留旧数据；前端 `commandBuilder.ts` 已删除（参数构建移入后端引擎）。

## 状态说明

| 状态 | 符号 | 说明 |
| --- | --- | --- |
| 已完成 | `[x]` | 功能已实现并测试 |
| 进行中 | `[/]` | 正在开发中 |
| 计划中 | `[ ]` | 已规划，待开发 |
| 暂不实现 | `[-]` | 暂不纳入开发计划 |

---

## 零、基础设施 (Foundation)

| 功能 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- |
| 项目结构 | `[x]` | `src/`, `src-tauri/` | Tauri 2.0 + Vue 3 + Vite；后端四层架构（app/domain/infrastructure/shared） |
| 领域类型定义 | `[x]` | `src/domain/` (task.ts, config.ts, stream.ts, url.ts) | 前端类型唯一来源，与后端 JSON 契约一一对应 |
| 路由配置 | `[x]` | `src/router/index.ts` | Vue Router |
| Pinia Store | `[x]` | `src/stores/` | taskStore, settingsStore, presetStore |
| TailwindCSS | `[x]` | `tailwind.config.js` | 样式系统 |
| Tauri 命令框架 | `[x]` | `src-tauri/src/app/commands/` | tasks, download, settings, presets, history, tools, system |
| 进程管理器 | `[x]` | `src-tauri/src/infrastructure/process/manager.rs` | State 注入 + Drop/Exit 双保险清理 |
| 服务层封装 | `[x]` | `src/services/` | task, download, settings, preset, history, tools, system, clipboard, update |
| SQLite 数据库 | `[x]` | `src-tauri/src/infrastructure/db/` | schema v4 单表聚合 + tool_settings 通用表 + repository 模式 |
| Store 缓存层架构 | `[x]` | `src/stores/taskStore.ts` | 数据来源于后端，Store 为内存缓存 |
| 引擎策略架构 | `[x]` | `src-tauri/src/infrastructure/engines/` | DownloadEngine trait + EngineRegistry 自动分派，详见 07 |
| 错误处理体系 | `[x]` | `src-tauri/src/shared/error.rs` | AppError (thiserror) + AppResult\<T\>，命令层边界转 String |
| 前端测试设施 | `[x]` | vitest (src/domain/url.test.ts, src/utils/*.test.ts, src/components/task/*.test.ts, src/composables/recentDirs.test.ts) | 77 个测试：url 检测、format、validate、id + 添加任务向导（parseLinks / resolveLinkToTask / linkOptionVisibility / recentDirs） |
| 后端测试设施 | `[x]` | cargo test (src-tauri/src/) | 117 个测试：引擎参数/解析器（含 N_m3u8DL-RE 真实粘连进度流/退出倾泻冲刷）、状态机、仓储 CRUD；另有 `tests/nm3u8dl_live_pipeline.rs` 实跑集成测试（#[ignore]，需本地工具+网络） |

---

## 一、输入模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 单链接输入 | P0 | `[x]` | `src/components/task/AddTaskDialog.vue` + `LinkConfigCard.vue` | 三段式向导：粘贴步单链接直达单条配置卡（无页码/全部添加/跳过），完成即入库；选项可见性由引擎类型驱动（`linkOptionVisibility.ts`） |
| URL 格式验证 | P0 | `[x]` | `src/utils/validate.ts` + 后端 `domain/url.rs` | M3U8/MPD/MSS/HTTP，前端本地检测与后端对照 |
| 多链接逐条配置 | P0 | `[x]` | `src/composables/useAddTaskWizard.ts` + `src/components/task/LinkConfigCard.vue` | 三段式向导：粘贴解析→逐条配置（页码 i/N、添加/跳过/全部添加）→完成提交；流媒体按类型显隐专属选项，直链仅通用件 |
| 多链接批量输入 | P0 | `[x]` | `src/components/task/AddTaskDialog.vue` + `src/components/task/parseLinks.ts` | 多行粘贴/文件导入，换行分隔；`parseLinks` 纯函数分类（流媒体/直链/无效）并剔除无效项 + toast |
| 从文件导入 | P1 | `[x]` | `src/components/task/AddTaskDialog.vue` | TXT 文件导入 |
| 剪贴板自动检测 | P2 | `[x]` | `src/composables/useClipboardWatcher.ts` | 监控剪贴板，自动检测 M3U8/MPD/MSS 链接 |
| 拖拽输入 | P2 | `[x]` | `src/views/HomeView.vue` | 支持拖放文本链接或 TXT 文件 |
| URL 重复检测 | P1 | `[x]` | `src/components/common/UrlDuplicateDialog.vue` | 添加已存在 URL 时弹窗确认；向导逐条配置弹窗确认，批量添加静默跳过并在结束 toast 汇报 |

---

## 二、解析模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 解析命令框架 | P0 | `[x]` | `src-tauri/src/app/commands/download.rs` | 引擎策略分派解析 |
| M3U8 解析 | P0 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | 通过 N_m3u8DL-RE |
| MPD 解析 | P0 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | 通过 N_m3u8DL-RE |
| MSS 解析 | P1 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | 通过 N_m3u8DL-RE |
| HTTP 直链探测 | P1 | `[x]` | `infrastructure/media/ffprobe.rs` | ffprobe 元数据提取 |
| 视频流信息提取 | P0 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` + `infrastructure/media/ffprobe.rs` | 分辨率/编码/帧率 |
| 音频流信息提取 | P0 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` + `infrastructure/media/ffprobe.rs` | 语言/声道 |
| 字幕流信息提取 | P1 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | 语言/格式 |
| 加密检测 | P1 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | is_encrypted 字段 |
| 直播检测 | P1 | `[x]` | `infrastructure/engines/nm3u8dl/parser.rs` | is_live 字段 |

---

## 三、流选择

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 自动选择最佳 | P0 | `[x]` | `src/composables/useStreamSelector.ts` | 默认行为 |
| 手动选择 | P1 | `[x]` | `src/components/stream/StreamSelector.vue` | 流选择器 UI |
| 正则匹配选择 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 选择器输入框 |
| 预设模板 | P1 | `[x]` | `src/stores/presetStore.ts` + `src/components/settings/tabs/PresetsTab.vue` | DB 持久化预设（取代旧 localStorage 模板） |
| 流排除 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 流排除配置 |
| 广告过滤 | P1 | `[x]` | `src/components/settings/AdKeywordManager.vue` → `--ad-keyword` | 正则列表增删启停，过滤分片 URL 匹配的广告 |

---

## 四、下载模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 下载任务管理 | P0 | `[x]` | `src/stores/taskStore.ts` | CRUD 完整 |
| 任务添加 | P0 | `[x]` | `src/composables/useTasks.ts:addTask` | AddTaskDialog 闭环 |
| 任务删除 | P0 | `[x]` | `src/composables/useTasks.ts:removeTask` | |
| 进度显示 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 组件完成，数据已连接 |
| 速度显示 | P0 | `[x]` | `src/domain/task.ts` (TaskProgressData) | 后端引擎 parser 解析输出并推送 |
| 剩余时间估算 | P0 | `[x]` | `src/domain/task.ts` (TaskProgressData) | 后端引擎 parser 解析输出并推送 |
| 暂停/继续 | P0 | `[x]` | `src/composables/useDownloader.ts` | 暂停=终止进程、恢复=重启（N_m3u8DL-RE 不支持断点续传） |
| 取消下载 | P0 | `[x]` | `src/composables/useDownloader.ts:stopDownload` | 后端实现 |
| 重试机制 | P0 | `[x]` | `src/stores/taskStore.ts:retryTask` | |
| 并发控制 | P0 | `[x]` | `src/composables/useDownloader.ts` | 队列管理+自动启动下一任务 |
| 实际执行下载 | P0 | `[x]` | `src-tauri/src/app/commands/download.rs` | 引擎策略构建参数 → 子进程 |
| 范围下载 | P1 | `[x]` | TaskOverrides → `infrastructure/engines/nm3u8dl/args.rs` | 参数构建在后端引擎 |
| 限速下载 | P1 | `[x]` | TaskOverrides → `infrastructure/engines/nm3u8dl/args.rs` | 参数构建在后端引擎 |
| N_m3u8DL-RE 路径配置 | P1 | `[x]` | `src/components/settings/ToolManagerCard.vue` | 参数化共用，支持自定义路径/下载更新 |
| 双引擎自动分派 | P0 | `[x]` | `infrastructure/engines/` (EngineRegistry) | 按 URL 类型分派，Unknown 回退 N_m3u8DL-RE |
| TaskOverrides 任务级覆盖 | P0 | `[x]` | `src/domain/task.ts` (TaskOverrides) + `tasks.overrides_json` | 全字段可选，null=沿用全局默认 |
| 命名模板 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` → `--save-pattern` | 支持 `<SaveName>/<Resolution>/<Bandwidth>...` 变量 |
| 混流导入外部文件 | P2 | `[x]` | `src/components/settings/MuxImportManager.vue` → `--mux-import` | 混流时导入外部音视频/字幕，path/lang/name |

---

## 五、处理模块

### 5.1 解密

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 密钥配置 | P2 | `[x]` | `src/components/settings/sections/DecryptionSettings.vue` | KID:KEY 格式密钥管理 |
| 密钥文件读取 | P2 | `[x]` | `src/components/settings/sections/DecryptionSettings.vue` | 密钥文本文件路径 |
| 解密引擎选择 | P2 | `[x]` | `src/components/settings/sections/DecryptionSettings.vue` | FFmpeg/MP4Decrypt/Shaka |
| 实时解密 | P2 | `[x]` | `src/components/settings/sections/DecryptionSettings.vue` | 下载时实时解密 |
| HLS 自定义方法 | P2 | `[x]` | `src/components/settings/sections/DecryptionSettings.vue` | 自定义 HLS 解密配置 |

### 5.2 合并

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 自动合并 | P0 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 任务完成后自动混流 |
| 二进制合并 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 选项开关 |
| 删除临时文件 | P0 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 选项开关 |

### 5.3 混流

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | MP4/MKV |
| 混流器选择 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | FFmpeg/MKVMerge |
| 自定义程序路径 | P1 | `[x]` | `src/components/settings/ToolManagerCard.vue` | FFmpeg 路径管理 |
| 保留原文件 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | 选项开关 |
| 外部媒体导入 | P1 | `[-]` | — | 旧实现未接入参数构建（空壳），重构中移除 |

### 5.4 字幕

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | SRT/WebVTT |
| 自动修正时间轴 | P1 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 选项开关 |
| 仅下载字幕 | P2 | `[x]` | `src/components/settings/tabs/Nm3u8dlTab.vue` | 选项开关 |

---

## 六、直播模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 直播设置 | P2 | `[x]` | `src/components/settings/sections/LiveSettings.vue` | 直播录制设置 UI |
| 实时合并 | P2 | `[x]` | `src/components/settings/sections/LiveSettings.vue` | 选项开关 |
| 保留分片 | P2 | `[x]` | `src/components/settings/sections/LiveSettings.vue` | 选项开关 |
| 录制时长限制 | P2 | `[x]` | `src/components/settings/sections/LiveSettings.vue` | 时间格式输入 |
| 刷新间隔设置 | P2 | `[x]` | `src/components/settings/sections/LiveSettings.vue` | 等待时间和分片数 |

---

## 七、网络模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 系统代理 | P1 | `[x]` | `src/components/settings/sections/NetworkSettings.vue` | 选项开关 |
| 自定义代理 | P1 | `[x]` | `src/components/settings/sections/NetworkSettings.vue` | 代理地址输入 |
| 请求头管理 | P1 | `[x]` | `src/components/settings/sections/NetworkSettings.vue` | 支持添加/删除/启用/禁用 |
| BaseURL 设置 | P3 | `[x]` | `src/components/settings/sections/NetworkSettings.vue` | URL 替换 |

---

## 八、管理模块

### 8.1 任务队列

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 队列管理 | P0 | `[x]` | `src/stores/taskStore.ts` | |
| 并发限制 | P0 | `[x]` | `src/composables/useDownloader.ts` | 队列控制+自动启动 |
| 优先级调整 | P1 | `[x]` | `src/stores/taskStore.ts:reorderTasks` | |
| 批量操作 | P1 | `[x]` | `src/stores/taskStore.ts:clearCompleted,clearAll` | |

### 8.2 历史记录

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P1 | `[x]` | `src/domain/task.ts` (HistoryRecord) | 保留：镜像后端 JSON 契约 |
| SQLite 数据库 | P1 | `[x]` | `src-tauri/src/infrastructure/db/repository/history_repo.rs` | rusqlite 持久化 |
| 后端命令 | P1 | `[x]` | `src-tauri/src/app/commands/history.rs` | load_history / delete_history_record / clear_history |
| 记录保存 | P1 | `[-]` | — | 2026-07 前端 historyStore/historyService 移除（后端仍自动快照）；首页已完成分类即历史 |
| 历史列表 | P1 | `[-]` | — | 2026-07 HistoryView 移除：与首页「进行中/已完成」分类冗余，后者分类更清晰 |

### 8.3 任务预设

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P1 | `[x]` | `src/domain/config.ts` (TaskPreset) | |
| 预设 Store | P1 | `[x]` | `src/stores/presetStore.ts` | DB 持久化（取代旧 templateStore localStorage） |
| 预设管理 UI | P1 | `[x]` | `src/components/settings/tabs/PresetsTab.vue` | 命名预设管理 |
| 预设模板 | P1 | `[x]` | `src/stores/presetStore.ts` | 最佳质量/1080P/720P 预设 |

### 8.4 定时任务

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P2 | `[x]` | `src/domain/task.ts` (TaskOverrides.scheduledStartAt) | |
| 定时开始 | P2 | `[x]` | `src/components/task/LinkAdvancedSection.vue` + `src/composables/useDownloader.ts` | datetime-local + 30s 轮询调度，应用需运行 |

---

## 九、系统集成

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 系统托盘 | P2 | `[x]` | `src-tauri/src/app/tray.rs` | 最小化到托盘，托盘菜单；创建失败浮出 UI 警告（`get_tray_status`） |
| 最小化到托盘 | P2 | `[x]` | `src-tauri/src/lib.rs` + `resolve_close_behavior` | 关闭决策抽纯函数 + 关闭日志；DB 空表回默认（旧版升级被重置） |
| 下载完成通知 | P2 | `[x]` | `src/composables/useNotification.ts` + `tauri-plugin-notification` | **改用 Tauri 通知插件**（修复 WebView2 浏览器 Notification 恒 denied） |
| 剪贴板监控 | P2 | `[x]` | `src/composables/useClipboardWatcher.ts` | 选项开关；读权限已补 |
| 自动更新 | P3 | `[x]` | `src/composables/useUpdateChecker.ts` + `src/services/updateService.ts` | GitHub API 版本检查，自动下载安装；**App.vue 启动时检查**（原仅设置页挂载触发） |
| 最大并发任务数 | P0 | `[x]` | `src/composables/useDownloader.ts` + `AppSettings.max_concurrent_tasks` | 替代硬编码 `MAX_CONCURRENT_TASKS=5`，设置页可调 |

---

## 十、UI/UX

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 深色主题 | P0 | `[x]` | `src/style.css` | CSS 变量已定义 |
| 浅色主题 | P3 | `[x]` | `src/style.css` | CSS 变量已定义，主题切换已实现 |
| 主题切换 | P3 | `[x]` | `src/components/settings/tabs/GeneralTab.vue` | 主题选择器 UI |
| 多语言支持 | P3 | `[x]` | `src/locales/` | vue-i18n，简体中文/繁体中文/英文三语 |
| 主页布局 | P0 | `[x]` | `src/views/HomeView.vue` | 基本布局完成 |
| 任务卡片 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 渐进式披露（紧凑→悬停→点击详情）；状态单一来源（左侧圆形图标，已移除徽章与信息区重复文案） |
| 任务卡片右键菜单 | P2 | `[x]` | `src/components/task/TaskContextMenu.vue` | 右键菜单收纳复制链接/文件名/路径、以此链接重新下载（预填添加对话框）、打开详情 |
| 任务列表 | P0 | `[x]` | `src/components/task/TaskList.vue` | 组件完成 |
| 详情链接复制 | P2 | `[x]` | `src/components/task/TaskDetailPanel.vue` + `src/services/clipboardService.ts` | 标题行复制按钮：图标态切换 + toast 反馈；同期补齐 clipboard 读/写权限（修复剪贴板监控静默失败） |
| 设置页面 | P0 | `[x]` | `src/views/SettingsView.vue` | 2026-07 重设计：左侧导航栏 + 右侧单列内容（4 分区），SettingsGroup 单卡片 + divide-y 行模型，内联样式全量替换为语义化 token（浅色主题修复） |
| 添加任务弹窗 | P0 | `[x]` | `src/components/task/AddTaskDialog.vue` + `LinkConfigCard.vue` + `LinkAdvancedSection.vue` + `src/composables/useAddTaskWizard.ts`、`useRecentDirs.ts`（`recentDirs.ts`） + `parseLinks.ts` / `resolveLinkToTask.ts` / `addTaskTypes.ts` | 2026-08 重设计为三段式向导：①粘贴步（多行/文件导入，`parseLinks` 分类+剔除无效并 toast）②逐条配置步（页码 i/N、添加/跳过/全部添加；L1 字段 + 最近保存目录记忆下拉（useStorage，上限 5、去重、最新在前）+ 高级手风琴；按 UrlType 动态显隐引擎专属选项，内联 StreamPickerInline + 解析失败可重试）③完成提交（`resolveLinkToTask` 两层映射 + 提交调度；重复 URL 逐条弹 UrlDuplicateDialog / 批量静默跳过并结束 toast 汇报）；旧暂存外壳 TaskStagingList/LinkConfigPanel/staging-types 已删除；后端契约零改动 |
| Toast 提示 | P0 | `[x]` | `src/composables/useToast.ts` | |
| 日志查看器 | P2 | `[x]` | `src/components/task/LogViewer.vue` | 实时日志显示 |
| 进度图表 | P2 | `[x]` | `src/components/task/ProgressChart.vue` | Chart.js 下载速率曲线，实时更新 |
| 任务筛选 | P2 | `[x]` | `src/components/task/TaskFilterBar.vue` + `src/composables/useTaskFilter.ts` | 状态/搜索筛选 |

---

## 十一、通用组件

| 组件 | 状态 | 文件 | 备注 |
| --- | --- | --- | --- |
| AppIcon | `[x]` | `src/components/common/AppIcon.vue` | |
| AppProgress | `[x]` | `src/components/common/AppProgress.vue` | |
| UrlDuplicateDialog | `[x]` | `src/components/common/UrlDuplicateDialog.vue` | URL 重复检测弹窗 |
| RestoreTasksDialog | `[x]` | `src/components/common/RestoreTasksDialog.vue` | 启动时恢复中断任务 |
| shadcn-vue 组件库 | `[x]` | `src/components/ui/` | Textarea, Button, Dialog, Select, Switch 等 |

---

## 统计汇总

### 按状态

| 状态 | 数量 | 说明 |
| --- | --- | --- |
| `[x]` 已完成 | 115 | 基础设施（含引擎策略/测试体系/错误处理）+ 输入（含 URL 重复检测）+ 解析（含 ffprobe）+ 流选择（含广告过滤 `--ad-keyword`）+ 下载（含双引擎分派/TaskOverrides/命名模板 `--save-pattern`/混流导入 `--mux-import`）+ 处理 + 直播 + 网络 + 管理（含历史后端/预设 DB/定时调度）+ 系统集成（含通知插件修复/托盘状态浮出/启动检查更新/最大并发数配置化）+ UI/UX + 通用组件 |
| `[-]` 暂不实现 | 1 | 历史列表/记录保存前端（2026-07 移除，与首页分类冗余） |
| `[/]` 进行中 | 0 | - |
| `[ ]` 计划中 | 0 | - |
| **总计** | **116** | |

### 核心待实现 (P0 优先)

所有 P0 优先级功能已完成，无待实现项。

---

## 更新日志

| 日期 | 更新内容 |
| --- | --- |
| 2025-02-13 | 初始化功能状态文档 |
| 2025-02-13 | 完成基础设施、类型系统、Store、组件框架 |
| 2025-02-13 | 清理 Rust 警告，修复前端类型错误 |
| 2025-02-13 | 集成 Shadcn-Vue 组件库 (75+ 组件) |
| 2025-02-13 | 完成设置页面 UI (8 个标签页: 常规/下载/混流/网络/解密/直播/高级/界面) |
| 2025-02-13 | 更新 Toast 系统使用 Shadcn-Vue |
| 2025-02-13 | **核心功能实现**: 完善进程管理器，实现实际下载执行、进度事件推送、暂停/取消功能 |
| 2025-02-14 | **并发控制**: 实现任务队列管理、并发限制、任务完成后自动启动下一任务 |
| 2025-02-14 | **批量输入**: 实现多链接批量输入，支持可展开的多行文本框 |
| 2025-02-14 | **流解析功能**: 实现调用 N_m3u8DL-RE 解析 URL，提取视频/音频/字幕流信息 |
| 2025-02-14 | **所有 P0 优先级功能已完成！** |
| 2025-02-14 | **文件导入 / UI 简化 / 历史记录持久化 / SQLite / 历史记录列表 UI** |
| 2025-02-14 | **流选择器 UI / 配置模板管理 / 流排除 / 广告过滤 / 混流 / 网络 / 字幕** |
| 2026-02-14 | **请求头管理 / 解密设置 / 直播设置 / 外部媒体导入 / 定时开始 / 日志查看器** |
| 2026-02-14 | **拖拽输入 / 浅色主题 / 系统托盘 / 剪贴板检测 / 自动更新 / 多语言支持** |
| 2026-02-19 | **设置修复 / 文件信息增强 / 解析器改进 / 媒体信息存储 / 进度图表 / commandBuilder 架构重构** |
| 2026-02-20 | **自动更新下载安装功能** |
| 2026-07-21 | **完全重构**——引擎策略架构（DownloadEngine + EngineRegistry 自动分派）、三层配置模型（全局默认 + TaskOverrides + 引擎 args）、schema v4 单表聚合（tasks JSON 列 + tool_settings 通用表 + history 快照）、设置中心 9→4 标签页（常规·界面 / N_m3u8DL-RE / FFmpeg / 任务预设）+ ToolManagerCard 参数化共用、添加任务闭环（URL 类型徽章 / 流选择 / 预设选择器 / 定时开始 / 高级折叠）、历史记录与定时开始真实实现、移除 3 个空壳功能（广告过滤 / 外部媒体导入 / 命名模板）、前端 commandBuilder 删除（参数构建移入后端引擎 `infrastructure/engines/*/args.rs`）、测试体系建立（Rust 96 + vitest 47）；schema v4 全量重建不保留旧数据。详见 `07-tool-config-architecture.md` |
| 2026-07-21 | **UI 整修**——移除下载历史前端（HistoryView/historyStore/historyService 删除，与首页「进行中/已完成」分类冗余；后端历史快照保留）；设置页重设计（双栏导航栏 + 单卡片 SettingsGroup + divide-y 行模型 + 语义化 token，修复 tailwind alpha token 缺失与 `--accent-*` 变量未定义导致的浅色主题破图）；添加任务弹窗两层化（一级仅 URL，二级「更多选项」折叠，grid-rows 过渡，checkbox→Switch，宽度钳制 + 焦点环裁切修复） |
| 2026-08-05 | **工具能力最大化 + 设置生效性 + 全面测试**——① FFmpeg 直链 5 个死设置全部修复：`retry_count→-reconnect_max_retries`、`timeout→-rw_timeout`、`connection_timeout→-timeout`、`preserve_timestamps→-copyts` 真实接线，`max_speed` 移除（ffmpeg 无原生限速）；新增直链能力（`-http_proxy`/`-max_redirects`/`-cookies`/basic 认证/`-reconnect_on_http_error`/`-reconnect_delay_total_max`）；② N_m3u8DL-RE 补齐 4 项缺失功能（命名模板 `--save-pattern`、广告过滤 `--ad-keyword` 复活 AdKeywordManager、混流导入 `--mux-import`、恒定 `--disable-update-check`）；③ 应用配置项修复：桌面通知改用 `tauri-plugin-notification`（修复 WebView2 浏览器 Notification 恒 denied）、启动时检查更新（原仅设置页触发）、应用日志级别由 `log_level` 驱动、`max_concurrent_tasks` 配置化替代硬编码；④ 最小化到托盘可观测性（关闭决策抽 `resolve_close_behavior` 纯函数 + 关闭日志 + 托盘创建失败浮出 UI + `get_tray_status`）；⑤ 测试：Rust 139（args 契约矩阵全覆盖 + 纯命令 + 纯函数）、vitest 106（settingsStore/useUpdateChecker/useNotification/GeneralTab/SettingSwitch + 原 93）、CI 门禁补 `npm test`/`cargo test`。详见 `02a-cli-mapping.md` |
| 2026-08-01 | **进度解析修复**——N_m3u8DL-RE 20260628 在非 TTY（piped stdout）下进度块零分隔粘连且单条格式与旧正则不符，导致 `EngineEvent::Progress` 从不产出（DB `progress_history` 0 行、任务 `progress_json` 全 0、进度条/速度/图表全空）。`EngineSession` 由逐行 `parse_line→Option` 改为流式 `parse_chunk→Vec`；`OutputParser::parse_stream` 在累积文本上扫描核心进度块 + 手动定位尾部边界（标准 regex 不支持前瞻）；`Nm3u8dlSession` 改为缓冲 + 流式 + 双流聚合；`spawn_reader` 传原始行（含 `\n`）便于会话按 `\n` 排水。附带修复 `useDownloader` 多实例化导致任一消费者卸载（关闭弹窗/切标签页）会 `unsubscribeFromAll()` 全局清订阅的潜在缺陷——状态提升为模块级单例、移除 `onUnmounted(cleanup)`。真实捕获格式补 5 个后端单测，全量 116 后端 + 52 前端测试通过 |
| 2026-08-03 | **进度数据抢救（退出倾泻冲刷）**——实测定位上一轮修复后进度仍为 0 的根因：N_m3u8DL-RE 重定向下强制 Spectre 交互模式 + live Progress 显示，`StartAsync` 后全部输出（Markup 日志 + 100ms 进度帧）被 Spectre RenderHook 管线积压在进程内，仅进程退出瞬间一次性倾泻；且 `NonAnsiWriter` 的 `[\r\n] +` 正则把整块倾泻内容的换行剥光 → `parse_chunk` 按 `\n` 排水永远等不到边界，数据滞留会话缓冲直至丢弃。修复：`EngineSession` 新增 `finalize()`（`Nm3u8dlSession` 冲刷残余缓冲）；`ProcessManager` 等待线程先 join 两个读取线程再触发 `on_complete`（排序保证）；`download.rs` 在完成回调开头分派 finalize 事件、随后再 `flush_progress` + 发 `download:complete`。实测集成测试（真实二进制 + 管道 + GBK 解码）从 0 事件 → 53 个进度事件、0→100% 完整。局限：下载**过程中**进度条仍不动（工具输出机制决定，根治需工具侧改造——方案已评估：去 Interactive 强制 + 节流日志行输出；暂缓）。补 `finalize_drains_newline_less_exit_dump` 单测 + `tests/nm3u8dl_live_pipeline.rs` 实跑集成测试（#[ignore]）。全量 117 后端 + 77 前端测试、clippy、type-check 通过 |
| 2026-08-02 | **添加任务弹窗重设计**——主从详情式暂存层重写为三段式向导（粘贴→逐条配置→完成）：`AddTaskDialog` 降为薄壳，编排逻辑迁入 `useAddTaskWizard`（状态机+导航+提交调度）；新增 `LinkConfigCard`（L1 字段 + 最近保存目录记忆下拉）与 `LinkAdvancedSection`（引擎驱动动态项 + 内联流选择 + 解析重试）；纯函数 `parseLinks`（分类+剔除无效）/ `resolveLinkToTask`（两层映射，移除预设播种）/ `addTaskTypes`（类型徽章集中映射）；最近保存目录记忆（`useRecentDirs`/`recentDirs`，useStorage 上限 5、去重、最新在前）；键盘流（粘贴框 Enter 提交 / Shift+Enter 换行，配置步 Enter 添加/完成，Esc 关闭）；删除旧外壳 `TaskStagingList` / `LinkConfigPanel` / `staging-types`；前端门禁全绿（type-check / lint / vitest 77，9 文件）；后端契约零改动（`src-tauri/` diff 为空） |
| 2026-08-04 | **v0.6.0 发布**——完全重构 + 添加向导三段式 + 进度修复 + 右键菜单/复制链接汇总为 0.5.2 后的首个发布版；门禁全绿（Rust 117 + 前端 93 + clippy + type-check），发行说明见 `docs/releases/v0.6.0.md` |
| 2026-08-04 | **v0.6.1 发布**——下载失败时透传工具详细错误（403/exception/failed 等）替代笼统的「进程退出码」，`ProcessManager` 输出行环形缓冲 + `extract_error_hint` 纯函数（6 单测，Rust 共 123）；发行说明见 `docs/releases/v0.6.1.md` |
| 2026-08-07 | **最小化到托盘恢复路径修复**——根因：`tray-icon` crate 的 `menu_on_left_click` 默认即为 `true`，加上 `tray.rs` 显式 `.show_menu_on_left_click(true)`，Windows 上左键点击托盘图标只弹出菜单，`on_tray_icon_event` 的「单击显示窗口」逻辑永远不触发，最小化到托盘后无法单击恢复。修复：改 `.show_menu_on_left_click(false)`（左键单击触发 Click 事件→显示窗口；右键仍弹菜单）；补充缺失的窗口权限 `core:window:allow-hide` / `allow-show` / `allow-unminimize`。实测：关闭→隐藏到托盘、`show()` 恢复均正常 |
