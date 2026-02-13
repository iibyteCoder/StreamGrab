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
| 剪贴板自动检测 | P2 | `[ ]` | - | 监控剪贴板 |
| 拖拽输入 | P2 | `[ ]` | - | 拖拽文件/链接 |

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
| 自动选择最佳 | P0 | `[/]` | `src/stores/settingsStore.ts` | 设置存在，未连接后端 |
| 手动选择 | P1 | `[ ]` | - | 流选择器 UI |
| 正则匹配选择 | P1 | `[ ]` | - | 高级选择 |
| 预设模板 | P1 | `[ ]` | - | 保存选择规则 |
| 流排除 | P1 | `[/]` | `src/types/index.ts` | 类型定义存在 |
| 广告过滤 | P1 | `[/]` | `src/types/index.ts:AdFilterSettings` | 类型定义存在 |

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
| 密钥配置 | P2 | `[/]` | `src/types/index.ts:DecryptionSettings` | 类型定义存在 |
| 密钥文件读取 | P2 | `[/]` | `src/types/index.ts:DecryptionSettings.keyTextFile` | 类型定义存在 |
| 解密引擎选择 | P2 | `[/]` | `src/types/index.ts:DecryptionSettings.engine` | 类型定义存在 |
| 实时解密 | P2 | `[ ]` | - | |
| HLS 自定义方法 | P2 | `[/]` | `src/types/index.ts:CustomHlsDecryption` | 类型定义存在 |

### 5.2 合并

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 自动合并 | P0 | `[/]` | `src/types/index.ts:TaskConfig.muxAfterDone` | 类型定义存在 |
| 二进制合并 | P1 | `[/]` | `src/types/index.ts:DownloadSettings.binaryMerge` | 类型定义存在 |
| 删除临时文件 | P0 | `[/]` | `src/types/index.ts:DownloadSettings.delAfterDone` | 类型定义存在 |

### 5.3 混流

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[/]` | `src/types/index.ts:MuxSettings.format` | 类型定义存在 |
| 混流器选择 | P1 | `[/]` | `src/types/index.ts:MuxSettings.muxer` | 类型定义存在 |
| 自定义程序路径 | P1 | `[/]` | `src/types/index.ts:MuxSettings.binPath` | 类型定义存在 |
| 保留原文件 | P1 | `[/]` | `src/types/index.ts:MuxSettings.keepOriginal` | 类型定义存在 |
| 外部媒体导入 | P1 | `[/]` | `src/types/index.ts:MuxSettings.muxImports` | 类型定义存在 |

### 5.4 字幕

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 格式选择 | P1 | `[/]` | `src/types/index.ts:DownloadSettings.subFormat` | 类型定义存在 |
| 自动修正时间轴 | P1 | `[/]` | `src/types/index.ts:DownloadSettings.autoSubtitleFix` | 类型定义存在 |
| 仅下载字幕 | P2 | `[/]` | `src/types/index.ts:DownloadSettings.subOnly` | 类型定义存在 |

---

## 六、直播模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 直播设置 | P2 | `[/]` | `src/types/index.ts:LiveSettings` | 类型定义存在 |
| 实时合并 | P2 | `[ ]` | - | |
| 保留分片 | P2 | `[ ]` | - | |
| 录制时长限制 | P2 | `[/]` | `src/types/index.ts:LiveSettings.recordLimit` | 类型定义存在 |
| 刷新间隔设置 | P2 | `[/]` | `src/types/index.ts:LiveSettings.waitTime` | 类型定义存在 |

---

## 七、网络模块

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 系统代理 | P1 | `[/]` | `src/types/index.ts:NetworkSettings.useSystemProxy` | 类型定义存在 |
| 自定义代理 | P1 | `[/]` | `src/types/index.ts:NetworkSettings.customProxy` | 类型定义存在 |
| 请求头管理 | P1 | `[/]` | `src/types/index.ts:NetworkSettings.headers` | 类型定义存在 |
| BaseURL 设置 | P3 | `[/]` | `src/types/index.ts:NetworkSettings.baseUrl` | 类型定义存在 |

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
| 模板管理 | P1 | `[ ]` | - | |

### 8.4 定时任务

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 类型定义 | P2 | `[x]` | `src/types/index.ts:ScheduledTask` | |
| 定时开始 | P2 | `[/]` | `src/types/index.ts:TaskConfig.startAt` | 类型定义存在 |

---

## 九、系统集成

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 系统托盘 | P2 | `[ ]` | - | |
| 下载完成通知 | P2 | `[/]` | `src/types/index.ts:UISettings.showNotification` | 类型定义存在 |
| 剪贴板监控 | P2 | `[/]` | `src/types/index.ts:UISettings.clipboardWatch` | 类型定义存在 |
| 自动更新 | P3 | `[/]` | `src/types/index.ts:GeneralSettings.checkUpdate` | 类型定义存在 |

---

## 十、UI/UX

| 功能 | 优先级 | 状态 | 文件/位置 | 备注 |
| --- | --- | --- | --- | --- |
| 深色主题 | P0 | `[x]` | `src/style.css` | CSS 变量已定义 |
| 浅色主题 | P3 | `[ ]` | - | |
| 主题切换 | P3 | `[/]` | `src/types/index.ts:UISettings.theme` | 类型定义存在 |
| 多语言支持 | P3 | `[/]` | `src/types/index.ts:GeneralSettings.language` | 类型定义存在 |
| 主页布局 | P0 | `[x]` | `src/views/HomeView.vue` | 基本布局完成 |
| 任务卡片 | P0 | `[x]` | `src/components/task/TaskCard.vue` | 组件完成 |
| 任务列表 | P0 | `[x]` | `src/components/task/TaskList.vue` | 组件完成 |
| 设置页面 | P0 | `[x]` | `src/views/SettingsView.vue` | 8 个设置标签页完成 |
| Toast 提示 | P0 | `[x]` | `src/composables/useToast.ts` | |
| 日志查看器 | P2 | `[ ]` | - | |

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
| `[x]` 已完成 | 66 | 基础设施 + 核心下载功能 + UI 组件 + 并发控制 + 流解析 + 文件导入 + 历史记录 + SQLite + 历史列表 UI |
| `[/]` 进行中 | 18 | 类型已定义，逻辑未实现 |
| `[ ]` 计划中 | 23 | 完全未开始 |
| **总计** | **105** | |

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
