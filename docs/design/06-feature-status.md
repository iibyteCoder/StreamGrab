# 功能实现状态

本文档追踪 StreamGrab 各功能模块的实现进度。

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
| 项目结构 | `[x]` | `src/`, `src-tauri/` | Tauri 2.0 + Vue 3 + Vite |
| TypeScript 类型 | `[x]` | `src/types/index.ts` | 完整类型定义 |
| 路由配置 | `[x]` | `src/router/index.ts` | Vue Router |
| Pinia Store | `[x]` | `src/stores/` | taskStore, settingsStore |
| TailwindCSS | `[x]` | `tailwind.config.js` | 样式系统 |
| Tauri 命令框架 | `[x]` | `src-tauri/src/commands/` | config, download, task, keys |
| 进程管理器 | `[x]` | `src-tauri/src/process/manager.rs` | 完整实现，支持启停/进度推送 |
| 服务层封装 | `[x]` | `src/services/` | tauri.ts, downloadService.ts, taskService.ts |
| 命令行参数构建器 | `[x]` | `src/utils/commandBuilder.ts` | 完整支持所有 N_m3u8DL-RE 参数 |
| **SQLite 统一数据库** | `[x]` | `src-tauri/src/db/` | settings, keys, tasks, history 表 |
| **Store 缓存层架构** | `[x]` | `src/stores/taskStore.ts` | 数据来源于后端，Store 为内存缓存 |

---

## 一、输入模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 单链接输入 | P0 | `[x]` | `src/views/HomeView.vue` | 统一的多行输入框 |
| URL 格式验证 | P0 | `[x]` | `src/utils/validate.ts` | M3U8/MPD/MSS |
| 多链接批量输入 | P0 | `[x]` | `src/views/HomeView.vue` | 换行分隔，统一输入框 |
| 从文件导入 | P1 | `[x]` | `src/views/HomeView.vue` | TXT 文件导入 |
| 剪贴板自动检测 | P2 | `[x]` | `src/composables/useClipboardWatcher.ts` | 监控剪贴板，自动检测 M3U8/MPD/MSS 链接 |
| 拖拽输入 | P2 | `[x]` | `src/views/HomeView.vue` | 支持拖放文本链接或 TXT 文件 |

---

## 二、解析模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 解析命令框架 | P0 | `[x]` | `src-tauri/src/commands/download.rs:parse_url` | 调用 N_m3u8DL-RE |
| M3U8 解析 | P0 | `[x]` | `src-tauri/src/commands/download.rs` | 通过 N_m3u8DL-RE |
| MPD 解析 | P0 | `[x]` | `src-tauri/src/commands/download.rs` | 通过 N_m3u8DL-RE |
| MSS 解析 | P1 | `[x]` | `src-tauri/src/commands/download.rs` | 通过 N_m3u8DL-RE |
| 视频流信息提取 | P0 | `[x]` | `src-tauri/src/commands/download.rs:parse_meta_json` | 分辨率/编码/帧率 |
| 音频流信息提取 | P0 | `[x]` | `src-tauri/src/commands/download.rs:parse_meta_json` | 语言/声道 |
| 字幕流信息提取 | P1 | `[x]` | `src-tauri/src/commands/download.rs:parse_meta_json` | 语言/格式 |
| 加密检测 | P1 | `[x]` | `src-tauri/src/commands/download.rs` | is_encrypted 字段 |
| 直播检测 | P1 | `[x]` | `src-tauri/src/commands/download.rs` | is_live 字段 |

---

## 三、流选择

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 自动选择最佳 | P0 | `[x]` | `src/stores/settingsStore.ts` | 默认行为 |
| 手动选择 | P1 | `[x]` | `src/components/stream/StreamSelector.vue` | 流选择器 UI |
| 正则匹配选择 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 选择器输入框 |
| 预设模板 | P1 | `[x]` | `src/stores/templateStore.ts` | 预设模板系统 |
| 流排除 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 流排除卡片 |
| 广告过滤 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 广告过滤卡片 |

---

## 四、下载模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 下载任务管理 | P0 | `[x]` | `src/stores/taskStore.ts` | CRUD 完整 |
| 任务添加 | P0 | `[x]` | `src/composables/useTasks.ts:addTask` | |
| 任务删除 | P0 | `[x]` | `src/composables/useTasks.ts:removeTask` | |
| 进度显示 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 组件完成，数据已连接 |
| 速度显示 | P0 | `[x]` | `src/types/index.ts:TaskProgressData` | 后端解析输出并推送 |
| 剩余时间估算 | P0 | `[x]` | `src/types/index.ts:TaskProgressData.eta` | 后端解析输出并推送 |
| 暂停/继续 | P0 | `[x]` | `src/composables/useDownloader.ts` | 后端实现，通过终止/重启进程 |
| 取消下载 | P0 | `[x]` | `src/composables/useDownloader.ts:stopDownload` | 后端实现 |
| 重试机制 | P0 | `[x]` | `src/stores/taskStore.ts:retryTask` | |
| 并发控制 | P0 | `[x]` | `src/composables/useDownloader.ts:processQueue` | 队列管理+自动启动下一任务 |
| 实际执行下载 | P0 | `[x]` | `src-tauri/src/commands/download.rs:start_download` | **核心已实现** |
| 范围下载 | P1 | `[x]` | `src/types/index.ts:TaskConfig.customRange` | 参数构建已实现 |
| 限速下载 | P1 | `[x]` | `src/types/index.ts:DownloadSettings.maxSpeed` | 参数构建已实现 |
| 命名模板 | P1 | `[x]` | `src/types/index.ts:SavePatternSettings` | 参数构建已实现 |
| N_m3u8DL-RE 路径配置 | P1 | `[x]` | `src/types/index.ts:AdvancedSettings.n_m3u8dlPath` | 支持自定义程序路径 |

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
| 自动合并 | P0 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 任务完成后自动混流 |
| 二进制合并 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 选项开关 |
| 删除临时文件 | P0 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 选项开关 |

### 5.3 混流

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | MP4/MKV |
| 混流器选择 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | FFmpeg/MKVMerge |
| 自定义程序路径 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | 混流器路径 |
| 保留原文件 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | 选项开关 |
| 外部媒体导入 | P1 | `[x]` | `src/components/settings/sections/MuxSettings.vue` | 外部音频/字幕导入 UI |

### 5.4 字幕

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | SRT/WebVTT |
| 自动修正时间轴 | P1 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 选项开关 |
| 仅下载字幕 | P2 | `[x]` | `src/components/settings/sections/DownloadSettings.vue` | 选项开关 |

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
| 类型定义 | P1 | `[x]` | `src/types/index.ts:HistoryRecord` | |
| SQLite 数据库 | P1 | `[x]` | `src-tauri/src/db/history.rs` | rusqlite 持久化 |
| 后端命令 | P1 | `[x]` | `src-tauri/src/commands/config.rs` | load/save/add/clear/delete |
| 记录保存 | P1 | `[x]` | `src/stores/historyStore.ts` | 任务完成时自动保存 |
| 历史列表 | P1 | `[x]` | `src/views/HistoryView.vue` | 历史记录页面 |

### 8.3 配置模板

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P1 | `[x]` | `src/types/index.ts:ConfigTemplate` | |
| 模板 Store | P1 | `[x]` | `src/stores/templateStore.ts` | 增删改查 + 应用 |
| 模板管理 UI | P1 | `[x]` | `src/components/template/TemplateManager.vue` | 预设模板 + 自定义模板 |
| 预设模板 | P1 | `[x]` | `src/stores/templateStore.ts` | 最佳质量/1080P/720P |

### 8.4 定时任务

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P2 | `[x]` | `src/types/index.ts:ScheduledTask` | |
| 定时开始 | P2 | `[x]` | `src/components/input/UrlInputPanel.vue` | 高级选项中的日期时间选择器 |

---

## 九、系统集成

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 系统托盘 | P2 | `[x]` | `src-tauri/src/tray.rs` | 最小化到托盘，托盘菜单 |
| 下载完成通知 | P2 | `[x]` | `src/components/settings/sections/UISettings.vue` | 选项开关 |
| 剪贴板监控 | P2 | `[x]` | `src/components/settings/sections/UISettings.vue` | 选项开关 |
| 自动更新 | P3 | `[x]` | `src/composables/useUpdateChecker.ts` | GitHub API 版本检查，手动/自动检查 |

---

## 十、UI/UX

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 深色主题 | P0 | `[x]` | `src/style.css` | CSS 变量已定义 |
| 浅色主题 | P3 | `[x]` | `src/style.css` | CSS 变量已定义，主题切换已实现 |
| 主题切换 | P3 | `[x]` | `src/components/settings/sections/UISettings.vue` | 主题选择器 UI |
| 多语言支持 | P3 | `[x]` | `src/locales/` | vue-i18n 实现，支持中/英/繁三语 |
| 主页布局 | P0 | `[x]` | `src/views/HomeView.vue` | 基本布局完成 |
| 任务卡片 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 组件完成 |
| 任务列表 | P0 | `[x]` | `src/components/task/TaskList.vue` | 组件完成 |
| 设置页面 | P0 | `[x]` | `src/views/SettingsView.vue` | 9 个设置标签页完成 |
| Toast 提示 | P0 | `[x]` | `src/composables/useToast.ts` | |
| 日志查看器 | P2 | `[x]` | `src/components/task/LogViewer.vue` | 实时日志显示 |

---

## 十一、通用组件

| 组件 | 状态 | 文件 | 备注 |
| --- | --- | --- | --- |
| AppButton | `[x]` | `src/components/common/AppButton.vue` | |
| AppInput | `[x]` | `src/components/common/AppInput.vue` | |
| AppProgress | `[x]` | `src/components/common/AppProgress.vue` | |
| AppCard | `[x]` | `src/components/common/AppCard.vue` | |
| Textarea | `[x]` | `src/components/ui/textarea/Textarea.vue` | Shadcn-Vue 多行输入 |

---

## 统计汇总

### 按状态

| 状态 | 数量 | 说明 |
| --- | --- | --- |
| `[x]` 已完成 | 107 | 基础设施 + 核心下载功能 + UI 组件 + 并发控制 + 流解析 + 文件导入 + 历史记录 + SQLite + 历史列表 UI + 流选择器 + 配置模板 + 流排除 + 广告过滤 + 混流设置 + 网络代理 + 字幕设置 + 解密设置 + 直播设置 + 外部媒体导入 + UI 设置 + 自动更新 + 多语言 + 定时开始 + 日志查看器 + 拖拽输入 + 浅色主题 + 系统托盘 + 剪贴板检测 + i18n 翻译 |
| `[/]` 进行中 | 0 | - |
| `[ ]` 计划中 | 0 | - |
| **总计** | **107** | |

### 核心待实现 (P0 优先)

| 功能 | 文件 | 状态 |
| --- | --- | --- |
| ~~实际下载执行~~ | ~~`src-tauri/src/commands/download.rs:start_download`~~ | `[x]` |
| ~~进度事件推送~~ | ~~`src-tauri/src/process/manager.rs`~~ | `[x]` |
| ~~设置页面 UI~~ | ~~`src/views/SettingsView.vue`~~ | `[x]` |
| ~~多链接批量输入~~ | ~~`src/views/HomeView.vue`~~ | `[x]` |
| ~~并发控制逻辑~~ | ~~`src/composables/useDownloader.ts`~~ | `[x]` |
| ~~流解析 (调用 N_m3u8DL-RE)~~ | ~~`src-tauri/src/commands/download.rs:parse_url`~~ | `[x]` |

**所有 P0 优先级功能已完成！**

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
| 2025-02-14 | 新增 Textarea 组件 (Shadcn-Vue) |
| 2025-02-14 | **流解析功能**: 实现调用 N_m3u8DL-RE 解析 URL，提取视频/音频/字幕流信息，检测加密和直播状态 |
| 2025-02-14 | **所有 P0 优先级功能已完成！** |
| 2025-02-14 | **文件导入**: 实现从 TXT 文件导入 URL 列表 |
| 2025-02-14 | **UI 简化**: 统一单链接和多链接输入为一个多行输入框 |
| 2025-02-14 | **历史记录持久化**: 实现后端历史记录命令、historyStore、任务完成时自动保存 |
| 2025-02-14 | **SQLite 数据库**: 将历史记录持久化从 JSON 文件改为 SQLite 数据库 |
| 2025-02-14 | **历史记录列表 UI**: 实现历史记录页面，支持查看、删除、重新下载 |
| 2025-02-14 | **UI 重构**: 重构 SettingsView (999→189行) 和 HomeView (372→140行)，创建通用组件体系 |
| 2025-02-14 | **数据持久化架构重构**: 统一使用 SQLite 存储，重构 Store 为缓存层，支持任务恢复 |
| 2025-02-14 | **流选择器 UI**: 实现 StreamSelector 组件，支持视频/音频/字幕流选择，集成到下载流程 |
| 2025-02-14 | **配置模板管理**: 实现 templateStore 和 TemplateManager 组件，支持预设模板和自定义模板 |
| 2025-02-14 | 设置页面新增"模板"标签页，共 9 个标签页 |
| 2025-02-14 | **组件重构**: 将 StreamSelector 和 TemplateManager 拆分为 UI 组件 + Composable，遵循单一职责原则 |
| 2025-02-14 | **流排除 UI**: 在下载设置页面添加流排除卡片，支持正则表达式排除视频/音频/字幕流 |
| 2025-02-14 | **广告过滤 UI**: 在下载设置页面添加广告过滤卡片，支持动态添加/删除正则关键字 |
| 2025-02-14 | **混流设置 UI**: 完善混流设置组件，支持格式选择、混流器选择、程序路径、保留原文件等选项 |
| 2025-02-14 | **网络代理 UI**: 完善网络设置组件，支持系统代理、自定义代理、BaseURL 设置 |
| 2025-02-14 | **字幕设置 UI**: 在下载设置页面添加字幕设置卡片，支持格式选择、自动修正时间轴、仅下载字幕 |
| 2025-02-14 | **P1 优先级功能基本完成**：85/107 功能已实现 |
| 2026-02-14 | **请求头管理 UI**: 在网络设置页面添加请求头管理卡片，支持添加/删除/启用/禁用自定义 HTTP 请求头 |
| 2026-02-14 | **解密设置 UI**: 完善解密设置组件，支持密钥配置(KID:KEY)、解密引擎选择、实时解密、HLS 自定义解密 |
| 2026-02-14 | **直播设置 UI**: 完善直播设置组件，支持 VOD 模式、实时合并、保留分片、录制限制、等待时间等选项 |
| 2026-02-14 | **P2 优先级功能基本完成**：92/107 功能已实现 |
| 2026-02-14 | **外部媒体导入 UI**: 在混流设置页面添加外部媒体导入卡片，支持导入外部音频/字幕文件进行混流 |
| 2026-02-14 | **UI 设置完善**: 界面设置组件已包含通知开关、剪贴板监视、主题切换功能 |
| 2026-02-14 | **功能状态完善**: 更新自动更新、多语言支持状态，UI 已实现 |
| 2026-02-14 | **P1-P2 优先级功能基本完成**：97/107 功能已实现 |
| 2026-02-14 | **前后端联动修复**: 修复 commandBuilder 缺失参数(binaryMerge, writeMetaJson, concurrentDownload, ffmpegPath, keys 数组)，完善 useDownloader 任务配置合并逻辑 |
| 2026-02-14 | **定时开始 UI**: 在 URL 输入面板添加高级选项，支持选择定时开始日期时间 |
| 2026-02-14 | **日志查看器**: 创建 LogViewer 组件，支持实时查看任务日志、自动滚动、清除日志 |
| 2026-02-14 | **P2 优先级功能完成**：99/107 功能已实现 |
| 2026-02-14 | **拖拽输入**: 在 HomeView 添加拖放支持，可拖放文本链接或 TXT 文件到页面添加任务 |
| 2026-02-14 | **浅色主题**: 确认浅色主题 CSS 变量和主题切换功能已完整实现 |
| 2026-02-14 | **P3 优先级功能开始**: 101/107 功能已实现 |
| 2026-02-14 | **系统托盘**: 实现系统托盘功能，支持最小化到托盘、托盘菜单（显示窗口/退出）、单击托盘图标显示窗口 |
| 2026-02-14 | **剪贴板检测**: 实现剪贴板自动检测功能，监控剪贴板变化，自动检测 M3U8/MPD/MSS 链接并添加到输入框 |
| 2026-02-14 | **所有 P2 优先级功能完成**: 103/107 功能已实现 |
| 2026-02-14 | **自动更新检查**: 实现 GitHub API 版本检查功能，支持手动检查和启动时自动检查，显示当前版本和最新版本 |
| 2026-02-14 | **多语言支持**: 使用 vue-i18n 实现国际化，支持简体中文、繁体中文、英文三种语言，完成所有界面翻译 |
| 2026-02-14 | **所有功能实现完成**: 107/107 功能已实现 🎉 |
| 2026-02-19 | **设置项功能修复**: 修复 showNotification 设置未被使用的问题，创建 useNotification composable 实现系统通知功能，下载完成/失败时根据设置发送通知 |
| 2026-02-19 | **minimizeToTray 设置修复**: 修复后端总是最小化到托盘的问题，现在根据用户设置决定关闭窗口时的行为（最小化到托盘或退出应用） |
| 2026-02-19 | **文件存在检测修复**: 修复完成后文件总是显示"已删除"的问题，后端现在会扫描实际生成的文件路径，支持多种扩展名(.mp4/.mkv/.ts等)和文件名匹配 |
| 2026-02-19 | **文件信息增强**: 添加 get_file_info 后端命令，任务详情面板现在显示实际文件大小、文件名、格式、修改时间等信息 |
| 2026-02-19 | **解析器改进**: 改进 N_m3u8DL-RE 输出解析器，支持更多输出格式，包括 [DOWN]/[MERGE]/[MUX] 标签格式 |
| 2026-02-19 | **媒体信息存储**: 扩展数据库结构，添加 media_info_json 字段存储视频元数据（分辨率、编码、时长、帧率、HDR等），新增 update_task_media_info 后端命令 |
| 2026-02-19 | **任务详情面板优化**: 重新设计头部布局，将文件名和状态整合到标题栏，改进视觉层次，增加媒体信息显示（分辨率、时长、编码、帧率、色域、分片数、是否加密等） |
| 2026-02-19 | **媒体信息自动保存**: 下载开始时自动从解析的流信息中提取媒体元数据（分辨率、编码、帧率、时长等）并保存到任务 |
| 2026-02-19 | **媒体信息任务隔离修复**: 修复并发下载时媒体信息混乱的问题，改为每个任务启动时单独解析 URL 获取流信息，确保任务间数据完全隔离 |
| 2026-02-19 | **解析参数重构**: 新增 buildParseArgs 函数复用应用设置（网络、解密等），重构 parse_url 命令接收完整参数数组，修复解析时缺少 --auto-select 导致交互式提示失败的问题 |
| 2026-02-19 | **代码清理**: 删除旧的 parse_url 硬编码参数版本、删除 tauri.ts 中不存在的命令包装函数（loadConfig/saveConfig/getDefaultDownloadDir/checkFfmpegAvailable）、删除未使用的 HeaderItem 类型 |
| 2026-02-19 | **commandBuilder 架构重构**: 提取公共配置构建函数（addNetworkArgs/addDecryptionArgs/addLogArgs/addAdvancedArgs），确保 buildCommandArgs 和 buildParseArgs 使用相同的配置逻辑，提升代码一致性和可维护性 |
| 2026-02-19 | **进度图表功能**: 添加进度历史数据库表(progress_history)，实现 ProgressChart 组件使用 Chart.js 绘制下载速率曲线图（X轴=进度%, Y轴=速率），显示峰值/平均/当前速率统计 |
| 2026-02-19 | **ProgressChart 实时更新**: 重构 ProgressChart 组件支持实时数据更新，下载中从 taskStore 获取实时进度，下载完成后从数据库加载历史数据，合并显示完整曲线 |
| 2026-02-19 | **文件大小显示修复**: 修复文件大小在任务卡片和详情面板中不显示的问题，下载中显示"已下载/总大小"，已完成显示总大小 |
| 2026-02-19 | **进度条显示优化**: TaskCard 进度条在所有有进度时显示（不仅仅是下载中），新增暂停状态显示已下载大小和进度百分比 |
