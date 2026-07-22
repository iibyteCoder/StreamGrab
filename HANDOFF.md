# StreamGrab 发布前审计 — 交接文档

> 生成于 2026-07-22。本会话已完成大规模 UI 重设计 + 多轮 bug 修复，并完成一次三层架构审计（前端 / Rust 后端 / 横切面安全·配置·可测性）。本文件是新会话的起点——**先读完本文件，再向用户确认范围，然后按"建议修复顺序"动手**。

---

## 0. 项目快照

- **是什么**：Tauri 2 + Vue 3 + TS + Pinia + Tailwind/shadcn-vue 的桌面视频流下载器；后端 Rust 四层（app/commands → domain → infrastructure → shared），引擎策略（N_m3u8DL-RE + FFmpeg），SQLite 单表聚合 schema v4。
- **分支**：`refactor/tool-architecture`（大量未提交改动；本会话所有修复均未 commit，等用户决定切分粒度）。
- **当前验证状态（全绿，实跑过）**：
  - 前端 `type-check` ✅ / `lint` ✅ / `vitest` 47/47 ✅ / `vite build` ✅（⚠️ vendor chunk 872KB，见 P1）
  - Rust `cargo test` 99/99 ✅ / `cargo clippy -- -D warnings` ✅
- **GUI 无法被自动测试**（headless）：功能验证靠自动门禁 + 下面的"GUI 回归清单"由用户手动跑。

---

## 1. 本会话【已修】清单（新会话勿重复）

### 功能性 bug（同源：空/相对路径无统一保障）
- **下载保存目录为空 → 文件落 CWD/找不到**：[download.rs](src-tauri/src/app/commands/download.rs) 启动时按 `任务覆盖 > 全局默认 > 系统 Downloads/StreamGrab` 解析 `save_dir` 并 `create_dir_all`，同时作 `--save-dir` 与子进程 CWD、完成回调查找目录。
- **工具下载目标为空 → 误解压到 CWD**：[tools.rs](src-tauri/src/app/commands/tools.rs) `download_tool` 空目标时回退 `<app_data_dir>/tools`（绝对、可写，dev/打包都稳）。
- **ffmpeg 找不到（解析失败）**：[nm3u8dl/mod.rs](src-tauri/src/infrastructure/engines/nm3u8dl/mod.rs) 新增 `resolve_ffmpeg_bin()`（绝对化 + 存在性校验），解析与下载**一致**注入 `--ffmpeg-binary-path`（[args.rs](src-tauri/src/infrastructure/engines/nm3u8dl/args.rs) `build_parse_args` 增 `ffmpeg_bin` 形参 + 测试）。修前存的是相对路径 `ffmpeg-master-.../bin\ffmpeg.exe`，N_m3u8DL-RE 以自身 CWD 解析必然找不到。

### UI 重设计
- 删除"下载历史"页（前端 `HistoryView`/`historyStore`/`historyService` + 路由/导航/locales 全清；后端历史表保留但**只写不读**——见 P1）。
- 设置页重设计：双栏导航壳 [SettingsView.vue](src/views/SettingsView.vue)；`SettingsGroup` 单卡片 + `divide-y` 行模型 [SettingsGroup.vue](src/components/settings/SettingsGroup.vue)；行组件加 `padded` prop；4 个 tab + sections + ToolManagerCard 全量 token 语义化。
- **根因修复**：[tailwind.config.js](tailwind.config.js) 颜色 token 补 `<alpha-value>`（修前所有 `bg-card/50` 等透明度修饰符被静默忽略）；[style.css](src/style.css) 补定义 `--accent-primary/success/error`（修前三色变量从未定义、渲染为继承色）。
- AddTaskDialog 两层渐进披露 + 裁切修复（[AddTaskDialog.vue](src/components/task/AddTaskDialog.vue)）：一级仅 URL 不滚动；二级"更多选项"折叠（grid-rows 过渡）；checkbox→Switch；宽度 `max-w-[min(600px,calc(100vw-2rem))]` + 滚动区 `px-2 -mx-2` 焦点环留白。
- 自定义应用图标：矢量主源 [public/logo.svg](public/logo.svg)（深色石板 + 三段蓝色级联流 + 下载箭头），`npx tauri icon` 从矢量生成全套位图（ico/icns/32×32 含托盘）；favicon 与标题栏 logo 指向 `/logo.svg`；`index.html` 标题 `temp-vue`→`StreamGrab`。

### 弹窗/控制台噪音
- 预设编辑/删除弹窗崩溃：`<SelectItem value="">`（reka 禁用空串）→ 哨兵 `__default__` + 双向映射；弹窗 `max-h-[85vh] flex flex-col` + 可滚动 + 固定页脚（修前上下被截、无法关闭）。
- GitHub 更新检查 403 刷屏：[updateService.ts](src/services/updateService.ts) 条件请求缓存（ETag/If-Modified-Since，304 零配额，403/离线降级用缓存）+ [useUpdateChecker.ts](src/composables/useUpdateChecker.ts) 24h 跨会话节流（localStorage）。
- 通知权限被拒刷屏：[useNotification.ts](src/composables/useNotification.ts) 缓存"已拒绝"静默短路；icon `/favicon.ico`→`/logo.svg`。
- 剪贴板 watcher 日志 `console.log`→`console.debug`（[useClipboardWatcher.ts](src/composables/useClipboardWatcher.ts)）。
- 无障碍：给 AddTaskDialog / 预设编辑·删除 / LogViewer 补 `<DialogDescription class="sr-only">`（消除 reka `Missing Description` 警告）。

---

## 2. 审计总清单（P0→P2，三路合并去重）

### 🔴 P0 — 发布阻塞

| # | 位置 | 问题 | 修复 |
|---|---|---|---|
| P0-1 | [schema.rs:94-105](src-tauri/src/infrastructure/db/schema.rs#L94-L105) | schema 版本不符即**整库删除重建**，零迁移/零备份/零提示 → 升级即丢全部任务/设置/预设 | 改迁移链 `migrate_v4_to_v5...`；最少在破坏性操作前复制 `streamgrab.db.bak.v{N}` + 首启弹窗 |
| P0-2 | [ffmpeg/args.rs:51](src-tauri/src/infrastructure/engines/ffmpeg/args.rs#L51) + [manager.rs:117-126](src-tauri/src/infrastructure/process/manager.rs#L117-L126) + [nm3u8dl/args.rs:34](src-tauri/src/infrastructure/engines/nm3u8dl/args.rs#L34) | 路径"空/相对→CWD"这族 bug **无统一保障**，仍潜伏两处（ffmpeg args 不判空；manager 工作目录不存在时 warn 后静默回退继承 CWD）。本会话是三处各打补丁 | 引入 `ResolvedPath` 新类型（非空+绝对+存在，编译期保证），命令层构造一次往下传；`ProcessManager::start_process` 收 `ResolvedPath`。一次根除全族 |
| P0-3 | [tauri.conf.json:29](src-tauri/tauri.conf.json#L29) | `security.csp: null` → webview 零脚本源限制；XSS→可调任意 Tauri 命令（download_tool/run_installer/shell）= 本地代码执行链 | 设严格 CSP：`default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https://api.github.com`（注意 updateService 在 webview fetch api.github.com） |
| P0-4 | [tauri.conf.json:52-57](src-tauri/tauri.conf.json#L52-L57) + [lib.rs:32](src-tauri/src/lib.rs#L32) + Cargo.toml | updater 插件**已注册**但 `pubkey:""` 空、前端从不调用（自研 updateService 走下载安装包）；若谁误激活 check() = 不验签的供应链风险；现状是误导性死配置 | **建议删除**整块 updater 配置 + lib.rs 注册 + Cargo/package 依赖（自研流已覆盖一切）；或正确生成密钥对填 pubkey + capabilities 加 `updater:default` + 前端接线 |
| P0-5 | [tools.rs:269-297](src-tauri/src/app/commands/tools.rs#L269-L297) + [system.rs:196-256](src-tauri/src/app/commands/system.rs#L196-L256) | 工具 ZIP 与应用安装包下载**无任何完整性校验**（只查 content-length 与 ZIP magic）→ 供应链投毒 | SHA-256 校验：工具从 release 的 `.sha256` 资产取或按版本硬编码；安装包发布时附 checksum 下载前校验 |
| P0-6 | [taskStore.ts:157,195](src/stores/taskStore.ts#L157) + AddTaskDialog | 前端仍把空 `saveDir` 写库（对话框从不合并全局默认）——后端兜底已挡住，但类型系统允许空串、前端无校验（同族根因的前端侧） | 在 `taskService` 边界加 `resolveTaskDefaults()` 规范化（saveDir←defaultSaveDir、fileName←extractFileName(url)）再入库；taskStore 不应向语义可选路径写 `""` |
| P0-7 | [ToolManagerCard.vue:181](src/components/settings/ToolManagerCard.vue#L181) | `targetDir = dirPath \|\| configPath \|\| ""` 仍可能传空给 downloadTool（后端已兜底） | 服务边界 `toolsService.downloadTool` 拒绝空 targetDir，或前端解析为有效目录后再发 |

> P0-6/P0-7 后端已兜底，当前不炸；列为 P0 是因为"空串合法地穿过类型系统"是同族根因，`ResolvedPath`/`resolveTaskDefaults` 才是根治。

### 🟠 P1 — 上线前应修

| # | 位置 | 问题 | 修复 |
|---|---|---|---|
| P1-1 | [history_repo.rs](src-tauri/src/infrastructure/db/repository/history_repo.rs) + [tasks.rs:68-71](src-tauri/src/app/commands/tasks.rs#L68-L71) | 每个任务完成都写历史快照，前端**从不读**（已删历史页）→ 死写入 + 无界增长 + 持锁竞争 | 删写入 + schema 删 history 表；或实现前端历史 UI；或加 TTL/上限 |
| P1-2 | [lib.rs:48-50](src-tauri/src/lib.rs#L48-L50) | `setup()` 里 config_dir/DB init 用 `.expect()` → 启动期可崩且无 UI（只读盘/AV 锁库/漫游配置即崩） | 改 `?` + `map_err` 从 `setup` 返回错误，至少落日志/弹窗 |
| P1-3 | [config.rs:231,236,241](src-tauri/src/infrastructure/tools/config.rs#L231) | `ToolRegistry::downloader()/ffmpeg()/ffprobe()` 热路径 `.unwrap()`（OnceLock 现总注册，安全但脆） | 改 `.expect("… 必须在 ToolRegistry::global() 注册")` 或返回 Option |
| P1-4 | download.rs:49,110-115 等见下 | **关键路径零测试**=bug 反复的根因。需补：`default_save_dir`、save_dir 三级兜底、`resolve_ffprobe_bin`、`none_if_empty`、`ProcessManager` 生命周期、`extract_zip` 路径穿越、`ToolDetector::detect_single_tool` | 逐项补单测（红色先行） |
| P1-5 | [vite.config.ts](vite.config.ts) | 单 `index` chunk 872KB，无 `manualChunks`，触发 >500KB 警告 | 拆 vendor/ui/i18n/charts/tauri 分块 |
| P1-6 | [LayoutView.vue:9,32,55,70](src/views/LayoutView.vue#L9) | 直接用 `@tauri-apps/api/window`（`getCurrentWindow()` 等）——违反 CLAUDE.md"组件不经 services" | 抽 `windowService`（或扩 systemService） |
| P1-7 | [updateService.ts:86-97](src/services/updateService.ts#L86) + [ToolManagerCard.vue:211-221](src/components/settings/ToolManagerCard.vue#L211) | 两份重复 `compareVersions`；updateService 私有 `formatFileSize` 重复 [utils/format.ts:10](src/utils/format.ts#L10) | 抽 `utils/version.ts`（带测试）共用；删私有副本 |
| P1-8 | [AppIcon.vue:34](src/components/common/AppIcon.vue#L34) + ~30 调用点 | `name: keyof typeof Icons` 不满足 `<component :is>`，每个调用点都 `as any`；存了非法图标名→运行时空白无报错 | AppIcon 内部一次性正确类型化（`markRaw`/typed Component 查找），消除全部调用点 `as any` |
| P1-9 | [PresetsTab.vue:182](src/components/settings/tabs/PresetsTab.vue#L182) | `preset.icon as keyof typeof import('lucide-vue-next')` 无校验无兜底 | `usePresetManager.savePreset` 校验图标名；组件未知图标回退 `Bookmark` |
| P1-10 | [capabilities/default.json:16-17](src-tauri/capabilities/default.json#L16) | `shell:allow-open`+`shell:allow-execute` 无 scope；open 可开任意 scheme（file://、javascript:…） | open 限定 `https://` scope；`allow-execute` 前端未用→删（后端走 `std::process::Command`） |
| P1-11 | 见 §3 错误策略 | 大量 `console.error` 无 toast（[taskStore.ts:205](src/stores/taskStore.ts#L205) createTask 吞掉、[useDownloader.ts:101-202](src/composables/useDownloader.ts#L101) 多处、TaskCard/TaskDetailPanel 打开/删除、[App.vue:52,68](src/App.vue#L52)）= 静默失败 | 立约定：用户发起动作必须 toast；中央 invokeTauri 对未处理错误统一提示 |
| P1-12 | [tauri.conf.json:42-46](src-tauri/tauri.conf.json#L42) | `certificateThumbprint: null` → 未签名，SmartScreen 警告 | 上线前配代码签名证书；或 README 说明 SmartScreen |

### 🟡 P2 — 清理/打磨

- [nm3u8dl/args.rs:41-45](src-tauri/src/infrastructure/engines/nm3u8dl/args.rs#L41)：`tmp_dir` 兜底依赖两帧外的隐式前置；`ResolvedPath` 可使之显式。
- [fs.rs:37-84](src-tauri/src/infrastructure/fs.rs#L37)：`find_output_file` "目录内最新媒体"兜底，多任务并发可能返回他人文件 → 删该兜底，前缀失败直接走期望路径。
- 进度跟踪器 `OnceCell<Mutex>` 与 ProcessManager 外层 `tokio::Mutex`、内层 `std::Mutex` 混用——当前正确但无类型级约束，维护陷阱。
- 死代码：[src/components/template/*](src/components/template)（A 确认零引用）、[AdKeywordManager.vue](src/components/settings/AdKeywordManager.vue)、[AddTaskDialog.vue:384](src/components/task/AddTaskDialog.vue#L384) `duplicateCount` 死变量、[constants.ts:93](src/utils/constants.ts#L93) `TEMPLATES_FILE_NAME`。
- [ProgressChart.vue:296-335](src/components/task/ProgressChart.vue#L296)：同任务重试时 `liveDataPoints` 不重置，混入上次数据 → watch `startedAt`/转 downloading 时重置。
- [LayoutView.vue](src/views/LayoutView.vue) 标题栏按钮用 `title`（中文）无 `aria-label`、无 `:focus-visible` 环。
- 硬编码中文散布：StreamSelector、UrlDuplicateDialog、TaskActionButtons、ProgressChart、SettingPath、[constants.ts](src/utils/constants.ts) `TASK_STATUS_*`。AddTaskDialog/HomeView 是**已知故意**保留；其余应逐步转 `t()`（en-US 用户最大的迁移成本项）。

### ✅ 做得好（勿为重构而重构）
- 条件请求缓存（updateService.ts）——生产级限流处理。
- 四层架构 + `AppError`/`AppResult` 边界转 String 规范、有文档且落实。
- 引擎策略模式 + args 构建器测试充分（nm3u8dl/ffmpeg/args.rs + fs.rs 6 测 + config.rs 5 测）。
- 三层配置合并（default>global>override）有文档且 `patch_typed` 深合并 + `serde(default)` 前向兼容。
- 服务层纪律强（invokeTauri 唯一 chokepoint，仅 LayoutView 一处越界）；domain 类型单一来源；composables 薄封装；进程清理双保险（Drop + Exit hook）；GBK 解码；三语 locale 键齐整。

---

## 3. 待用户拍板的决策（不要擅自改）

1. **安全策略**：CSP 的确切值（尤其 `connect-src` 放不放 `api.github.com`、是否还允许 inline style for shadcn）；updater 块**删**还是**接线+pubkey**；shell 权限 scope 细则；是否本期上代码签名。
2. **DB 升级策略**：做完整迁移链，还是"破坏前备份 + 首启确认弹窗"的最小改？
3. **ResolvedPath 重构**：现在做（中量改动、跨 download/parse/tools/manager/ffmpeg-args）根除全族，还是保留补丁、仅补测试？
4. **历史表**：删写入+删表，还是实现前端历史 UI，还是加 TTL？
5. **i18n 迁移**：是否本期把散布的硬编码中文转 `t()`（范围不小）。
6. **死代码**：确认可删 `template/` + `AdKeywordManager` 等。
7. **提交粒度**：本会话改动未 commit，要按主题切几个 commit？

---

## 4. 建议修复顺序（用户确认范围后）

1. **Phase 1 安全/发布阻塞（需用户定策略）**：P0-3 CSP、P0-4 updater 块、P1-10 shell scope、P0-5 下载完整性校验。
2. **Phase 2 数据安全**：P0-1 DB 迁移/备份。
3. **Phase 3 同族根因根除 + 测试**：P0-2 `ResolvedPath`、P0-6/P0-7 前端 `resolveTaskDefaults` + 服务边界校验、P1-4 补红色测试集。
4. **Phase 4 健壮性 P1**：P1-1 history、P1-2 启动 `.expect`、P1-3 ToolRegistry unwrap、P1-11 错误策略约定、P1-8 AppIcon 类型、P1-7 去重。
5. **Phase 5 清理/打磨 P2**：死代码、P1-5 manualChunks、无障碍（标题栏 aria/focus、剩余 DialogDescription）、i18n、find_output_file 兜底、ProgressChart 重试重置。
6. **Phase 6 GUI 回归 + 发布构建**：用户手测 + `npm run tauri build`。

---

## 5. GUI 回归清单（用户手动，自动化测不到）

- 设置页：左导航切 4 区正常；切浅色主题颜色正确（token 语义化验收点）；工具路径行对齐、"未安装"徽章内联不独占。
- 预设：新建/编辑弹窗能开、能滚、能关；混流/字幕下拉不再崩、选"沿用默认"正确保存。
- 添加任务：窗口拉到最小 720×480，一级不滚动；展开"更多选项"正常、Switch 可用。
- 图标：标题栏/任务栏/托盘显新矢量 logo。
- 下载：m3u8 解析不再报 `找不到 ffmpeg`；下载完成后文件落 `<下载目录>/StreamGrab/`（或配置的全局默认），能打开；日志无 `Save directory does not exist:` 空值、下载不再触发 dev `Rebuilding`。
- 更新检查：多次开设置页控制台不再刷 403。
- 一次性清理：手动删 `src-tauri/6fXBGpaUxaoeXTTt*.mp4`（误生成）；设置里重选 FFmpeg 目录成绝对路径，清掉历史脏数据。

## 6. 环境性注意

- 用户网络走代理 `127.0.0.1:7897`；GitHub `api.github.com` 未鉴权 60/h/IP（已用条件请求+24h 节流压到几乎零）。
- `cargo run` 下子进程 CWD = `src-tauri`（曾导致误生成文件在此 + 触发 Tauri watcher Rebuilding；已由保存目录兜底修复）。
- tauri dev 热更会重置 `useUpdateChecker` 的内存节流（localStorage 持久化部分不受影响）。

## 7. 新会话起步指令

1. 读本文件 + `CLAUDE.md` + `docs/design/06-feature-status.md`。
2. 用 CodeGraph（`.codegraph/` 已索引）做定位，别上来 grep。
3. 与用户确认 §3 的决策范围（尤其安全策略与 DB 迁移），再按 §4 顺序动手。
4. 每个 Phase 后跑全量门禁（§0 命令）再继续；安全/行为有 trade-off 的改动先对齐再改。
5. GUI 验证项交给用户手测（§5）。
