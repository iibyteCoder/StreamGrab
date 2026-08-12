# 工具架构与配置体系设计

> 2026-07 完全重构的设计沉淀。本次重构不保留向后兼容（schema v4 全量重建），
> 前端 `src/utils/commandBuilder.ts` 已删除——前端不再持有任何工具的 CLI 知识。

## 一、三层配置模型

```
全局默认（设置中心 → SQLite：app_settings + tool_settings，按工具各自管理）
        ↓  提供默认值
TaskOverrides（添加任务对话框 → tasks.overrides_json 列，随任务持久化）
        ↓  后端引擎合并（任务级非空覆盖 > 全局默认）
命令行参数（引擎 args 模块构建 → N_m3u8DL-RE / FFmpeg 进程）
```

- **全局层**：`app_settings`（应用级，单行 JSON）+ `tool_settings(tool_id, config_json)`（按工具分行，新增工具零 DDL）
- **任务层**：`TaskOverrides` 全字段可选，`null/undefined` = 沿用全局默认；持久化于 `tasks.overrides_json`，历史记录同时快照（`history.overrides_json`），支持「重新下载携带原参数」
- **合并点**：后端引擎的 `build_download_args(spec, tools, app)`，前端不参与参数构建

## 二、下载引擎策略（Strategy）

### 领域契约（`domain/download/engine.rs`）

```rust
pub trait DownloadEngine: Send + Sync {
    fn id(&self) -> ToolId;                                  // nm3u8dl | ffmpeg
    fn handles(&self, url_type: UrlType) -> bool;            // 分派谓词
    fn build_download_args(&self, spec, tools, app) -> Vec<String>;
    fn build_parse_args(&self, url, tools, app) -> Vec<String>;
    fn parse_streams(&self, stdout: &str) -> StreamInfo;     // 解析模式输出 → 流信息
    fn new_session(&self) -> Box<dyn EngineSession>;         // 逐任务解析会话（跨行状态）
}
```

- **引擎自包含**：每个工具的参数构建、输出解析、进度模型全部内聚于 `infrastructure/engines/<tool>/`（`args.rs` + `parser.rs`）
- **会话模式**：引擎实例全局共享无状态；逐任务的跨行解析状态（FFmpeg 进度缓冲、N_m3u8DL-RE 视频/音频双流进度聚合）由 `EngineSession` 持有
- **自动分派**：`EngineRegistry::for_url(UrlType)` 按 URL 类型分派，`Unknown` 回退到 N_m3u8DL-RE（格式覆盖最广）。用户无需选择工具

### 引擎实现

| 引擎 | 处理类型 | 参数构建来源 | 输出解析 |
| --- | --- | --- | --- |
| `Nm3u8dlEngine` | HLS/DASH/MSS/Unknown | Nm3u8dlConfig（含网络/解密子配置）+ FfmpegConfig.mux_*（-M 混流参数）+ 任务覆盖 | `OutputParser`（日志行/进度行/状态标记）+ 流列表解析 |
| `FfmpegEngine` | HTTP 直链视频 | FfmpegConfig 直链下载默认值 | `-progress pipe:2` 块解析（Duration → 百分比，bitrate → 速度） |

### 新增工具五步清单（扩展契约）

1. `infrastructure/tools/` 注册 `ToolDefinition`（二进制检测 / 版本获取 / GitHub 下载）
2. `domain/config.rs` 新增工具配置类型并加入 `ToolConfigs`（`tool_settings` 表自动支持）
3. `infrastructure/engines/<tool>/` 实现 `DownloadEngine`（args.rs + parser.rs）
4. `domain/download/url_type.rs` 增加分派规则
5. 前端增加一个设置标签页组件

刻意不引入插件框架——静态注册 + trait 对象对此规模已足够。

### 工具下载与平台适配（实测结论，2026-08）

- **按平台选择资产**：`Platform::is_platform_asset_for` 以「排除其他平台关键字 → 组合关键字（如 `win-x64`/`osx-arm64`/`linux64`）→ 平台关键字+架构关键字」三级匹配；`Arch` 枚举区分 x64/arm64，防止 `android-bionic-x64`、`linuxarm64` 等资产被误选。
- **按平台选择下载源**：

  | 工具 | Windows | macOS | Linux |
  | --- | --- | --- | --- |
  | N_m3u8DL-RE（GitHub nilaoda） | `win-x64.zip` | `osx-*.tar.gz` | `linux-*.tar.gz` |
  | FFmpeg | BtbN `win64-gpl-shared.zip` | evermeet.cx（ffmpeg+ffprobe 为两个独立 zip，ffprobe 经 `extraAssets` 随附下载） | BtbN `linux64-gpl-shared.tar.xz` |

  BtbN 不提供 macOS 构建，故 macOS 改走 evermeet.cx（`fetch_ffmpeg_evermeet`）。
- **压缩格式**：`extract_archive` 统一分派 `.zip`（zip crate）/ `.tar.gz`（flate2+tar）/ `.tar.xz`（lzma-rs+tar）；魔数校验防错误页；Unix 平台恢复可执行权限。
- **版本比较**（前端 `utils/version.ts`）：提取全部数字段逐位比较 + 预发布标识（`-beta` 等视为更旧），兼容 `v0.6.0-beta` vs `0.6.0`、日期版本 `latest-2026-08-09`、`0.6.0+hash` 构建元数据；BtbN 滚动 tag `latest` 在后端归一化为发布名中的日期（`2026-08-09`）。
- **「已是最新」状态**：`ToolManagerCard` 在已安装且检查最新版本后无更新时隐藏下载按钮、显示绿色徽章——修复旧实现中按钮常驻导致「已下载最新仍可重复下载」的误导。

## 三、数据层（schema v4，单表聚合）

| 表 | 设计 |
| --- | --- |
| `tasks` | 单表聚合：基础列 + `progress_json` / `media_info_json` / `overrides_json` 三个 JSON 列，消灭旧版 4 表 JOIN（34 列映射 → 单行映射） |
| `progress_history` | 速率曲线时序数据（独立表，图表消费） |
| `history` | 任务终态快照（含 overrides 快照），清除任务不删除历史 |
| `app_settings` | 单行 JSON |
| `tool_settings` | `(tool_id TEXT PK, config_json TEXT)`——通用工具配置表 |
| `task_presets` | 命名的 TaskOverrides 组合（取代旧 localStorage 模板） |

版本低于 4 时全量重建（DROP + CREATE，不保留旧数据）。

## 四、状态机（`TaskStatus`）

`pending → analyzing → downloading → merging → muxing → completed`，任意活跃态可到 `failed`/`cancelled`，活跃态可 `paused`（暂停 = 终止进程，恢复 = 重启，受 N_m3u8DL-RE 能力所限），终态可回 `pending`（重试/重新下载）。迁移经 `can_transition_to` 校验，命令层 `update_task_status` 强制检查；进入终态自动写入历史快照。

## 五、任务添加闭环（前端）

1. URL 输入（多行批量 / 拖拽 TXT / 剪贴板监控）→ 防抖 300ms 本地检测类型徽章
2. 引擎自动分支：流媒体 → `parse_url` + StreamSelector 流选择；直链 → ffprobe 元数据，无流选择
3. 任务级选项：文件名（自动建议）、保存位置覆盖（placeholder 显示全局默认）、预设选择器、定时开始（原生 `datetime-local`）
4. 高级折叠：限速 / 范围 / 容器格式 / 字幕选项
5. 提交 → TaskOverrides 随任务落库 → 按 auto_start_download 启动或交由调度器
6. **定时开始 = 前端调度器**：`useDownloader` 30s 轮询 pending 任务的 `scheduledStartAt`，到期启动（引擎无关），应用启动时补发到期任务

## 六、设置中心（9 → 4 标签页，按工具管理）

| 标签页 | 内容 |
| --- | --- |
| 常规·界面 | 语言/主题/通知/剪贴板/托盘/更新（含检查/下载/安装 UI）/日志 |
| N_m3u8DL-RE | ToolManagerCard（路径/版本/下载更新）+ 下载默认 + 网络 + 解密 + 直播 |
| FFmpeg | ToolManagerCard + 混流默认 + 直链下载默认 |
| 任务预设 | TaskPreset 管理（DB 持久化） |

`ToolManagerCard` 按 toolId 参数化，两页共用；设置更新走 `patch_*` 递归合并（后端深合并，前端只发增量）。

## 七、进程管理

- `ProcessManager` 经 Tauri State 注入（`Arc<tokio::sync::Mutex<ProcessManager>>`）
- 孤儿进程双保险：`impl Drop` + `RunEvent::Exit` hook 均调用 `stop_all_sync()`（PID + taskkill /T 终止进程树）
- 子进程被杀后管道关闭，stdout/stderr 读取线程随 EOF 自然退出
- **排序保证**：等待线程在 `child.wait()` 后先 join 两个读取线程，再触发 `on_complete`——保证退出瞬间倾泻的输出全部到达回调（见下节）；`on_complete` 开头调用 `EngineSession::finalize()` 冲刷无 `\n` 结尾的残余缓冲

## 七-附、N_m3u8DL-RE 输出特性（实测结论，2026-08）

结合 N_m3u8DL-RE 源码（Spectre.Console 0.57.1）与真实二进制（0.6.0+df70f0b，20260628）管道抓包实测：

1. **重定向检测**：`Console.IsOutputRedirected` 时工具自动置 `ForceAnsiConsole + NoAnsiColor`（`Program.cs`），用 `NonAnsiWriter` 剥 ANSI——这是它为 GUI 包装设计的管道模式。
2. **进度来自 Spectre live 显示**：下载进度是 `Progress().AutoRefresh`（刷新率 100ms，`LogLevel != OFF` 时启用）渲染的表格帧（`Vid <desc> ------ N/M x% size speed eta`，ASCII 横线、GBK 编码）。
3. **关键缺陷（实测）**：`progress.StartAsync()` 启动 live 显示后，Spectre 的 RenderHook 管线接管控制台，**此后所有输出（Markup 日志 + 进度帧）积压在进程内，仅进程退出瞬间一次性倾泻**；`NonAnsiWriter` 的 `[\r\n] +` 正则又把倾泻内容的换行剥光，变成**无 `\n` 的单个粘连块**。普通日志行（`Logger.Info/Warn` 的消息体走 `Console.WriteLine`）不受影响、实时到达。
4. **StreamGrab 的应对**：`parse_stream` 解析粘连块（已验证格式）；`finalize()` 冲刷退出倾泻；因此**下载完成后**进度历史/图表数据完整。局限：下载**过程中**进度条不动——根治需工具侧改造（候选方案：重定向时不强制 Interactive，改由工具节流输出普通日志行；或 StreamGrab 侧 ConPTY 伪终端使工具进入真交互模式），当前暂缓。
5. **使用注意**：`--log-level OFF` 会关闭进度自动刷新（`progress.AutoRefresh = false`），设置中心勿提供/勿默认该级别；工具自身日志 `<工具目录>/Logs/*.log`（UTF-8、逐行实时落盘）是可用的旁路信号源。

## 八、错误处理

`AppError`（thiserror）：Database/Process/ToolNotFound/Config/Io/Parse/Serialization/Http/Other，`From<rusqlite::Error>`/`io::Error`/`serde_json::Error`/`reqwest::Error`。基础设施与领域层全部使用 `AppResult<T>`，仅命令层边界转换为 `Result<T, String>`（Tauri 前端契约）。

## 九、本次重构移除/降级的功能

| 功能 | 处理 | 原因 |
| --- | --- | --- |
| 广告过滤 UI | 移除 | 旧实现未接入参数构建（空壳 UI）；如需可用 `--urlprocessor-args` 实现 |
| 外部媒体导入（mux imports） | 移除 | 同上，旧实现未接入 |
| 命名模板（save pattern） | 移除 | 同上 |
| 暂停=断点续传 | 明确为终止/重启语义 | N_m3u8DL-RE 能力所限，UI 文案如实 |
| `get_n_m3u8dl_version` 硬编码桩 | 替换为真实检测 | 委托 ToolDetector |
