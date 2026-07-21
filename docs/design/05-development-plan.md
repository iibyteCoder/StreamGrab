# 开发计划

> **2026-07 已完成完全重构**，架构细节与配置体系设计详见 `07-tool-config-architecture.md`。
> 本文档为初始开发计划，大部分任务已完成。当前架构状态以 `06-feature-status.md` 为准。

## 开发阶段总览

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          开发阶段路线图                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Phase 1          Phase 2          Phase 3          Phase 4            │
│  项目搭建         核心功能         完善功能         高级功能            │
│  ────────         ────────         ────────         ────────            │
│                                                                         │
│  ├─ 环境配置      ├─ 下载功能      ├─ 流选择        ├─ 直播录制        │
│  ├─ 项目结构      ├─ 进度显示      ├─ 代理设置      ├─ 解密功能        │
│  ├─ 基础 UI       ├─ 任务控制      ├─ 混流设置      ├─ 定时任务        │
│  ├─ 路由配置      ├─ 任务队列      ├─ 历史记录      ├─ 系统托盘        │
│  └─ 主题系统      └─ 基础设置      └─ 配置模板      └─ 自动更新        │
│                                                                         │
│  [████████]       [████████]       [████████]       [████████]          │
│   基础可用         核心完成         功能完善         生产就绪            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: 项目搭建

### Phase 1 目标

搭建完整的项目框架，实现基础 UI 布局

### Phase 1 任务清单

#### 1.1 环境准备

- [x] 安装 Node.js 18+
- [x] 安装 npm
- [x] 安装 Rust (rustup)
- [x] 安装 VS Code 及扩展
  - [x] Vue - Official
  - [x] Tauri
  - [x] rust-analyzer
  - [x] ESLint
  - [x] Prettier
  - [x] Tailwind CSS IntelliSense

#### 1.2 创建项目

- [x] 使用 `npm create tauri-app` 创建项目
- [x] 配置 Vue 3 + TypeScript + Vite
- [x] 安装依赖包

  ```bash
  npm install vue-router pinia @vueuse/core
  npm install -D tailwindcss postcss autoprefixer
  npm install @tauri-apps/plugin-shell @tauri-apps/plugin-fs
  ```

#### 1.3 配置开发工具

- [x] 配置 TailwindCSS
- [x] 配置 ESLint + Prettier
- [x] 配置 TypeScript (tsconfig.json)
- [x] 配置路径别名 (@/)

#### 1.4 项目结构搭建

- [x] 创建目录结构 (按 04-architecture.md)
- [x] 配置 Vue Router 路由
- [x] 配置 Pinia Store
- [x] 创建基础布局组件

#### 1.5 UI 基础组件

- [x] Button 组件（shadcn-vue）
- [x] Input 组件（shadcn-vue）
- [x] Select 组件（shadcn-vue）
- [x] Switch 组件（shadcn-vue）
- [x] Modal 组件（shadcn-vue Dialog）
- [x] Toast 组件（shadcn-vue）

#### 1.6 主题系统

- [x] CSS 变量定义
- [x] 深色主题样式
- [x] 浅色主题样式
- [x] 主题切换功能

#### 1.7 布局组件

- [x] LayoutView 布局
- [x] MainLayout 主布局

### Phase 1 里程碑

- [x] 项目可正常启动
- [x] 基础布局显示正常
- [x] 主题切换功能正常

---

## Phase 2: 核心功能

### Phase 2 目标

实现下载核心功能，包括 URL 输入、下载执行、进度显示、任务管理

### Phase 2 任务清单

#### 2.1 URL 输入模块

- [x] AddTaskDialog 组件
  - [x] 输入框基础功能
  - [x] URL 验证 + 类型徽章
  - [x] 剪贴板粘贴
- [x] 多行文本输入（换行分隔批量）
- [x] TXT 文件导入
- [x] 拖放支持（文本链接或 TXT 文件）

#### 2.2 任务数据模型

- [x] 定义 Task 类型 (src/domain/task.ts)
- [x] 定义 TaskProgress 类型
- [x] 定义 TaskConfig / TaskOverrides 类型
- [x] 创建 taskStore

#### 2.3 Rust 后端 - 下载命令

- [x] 创建 `src-tauri/src/app/commands/` 目录
- [x] 实现 `start_download` 命令（引擎策略分派）
- [x] 实现 `stop_download` 命令
- [x] 实现进程管理器 (infrastructure/process/manager.rs)
- [x] 实现输出解析器 (infrastructure/engines/*/parser.rs)

#### 2.4 前端下载服务

- [x] 创建 downloadService
- [x] 封装 Tauri invoke 调用
- [x] 实现事件订阅机制

#### 2.5 任务队列 UI

- [x] TaskList 组件
- [x] TaskCard 组件
  - [x] 显示任务信息
  - [x] 显示下载进度
  - [x] 显示下载速度
  - [x] 显示状态
- [x] ProgressChart 速率曲线组件
- [x] TaskActionButtons 操作按钮
  - [x] 开始/暂停
  - [x] 取消
  - [x] 打开目录

#### 2.6 任务控制逻辑

- [x] useDownloader composable（含队列 + 定时调度器）
- [x] useTasks composable
- [x] 任务添加/删除
- [x] 任务暂停/继续（终止/重启语义）
- [x] 任务取消
- [x] 并发控制

#### 2.7 基础设置

- [x] 定义 Settings 类型 (src/domain/config.ts)
- [x] 创建 settingsStore
- [x] Rust 配置读写命令
  - [x] `load_settings` / `patch_app_settings` / `patch_tool_settings`
- [x] 设置页面（4 标签页）
  - [x] 保存目录选择
  - [x] 格式选择
  - [x] 自动选择开关

#### 2.8 命名模板系统

- [-] ~~命名模板 UI~~ — 旧实现未接入参数构建（空壳），重构中移除

#### 2.9 广告过滤基础

- [-] ~~广告过滤 UI~~ — 旧实现未接入参数构建（空壳），重构中移除；可用 `--urlprocessor-args` 实现

### Phase 2 里程碑

- [x] 可以输入 URL 并开始下载
- [x] 进度实时更新显示
- [x] 可以暂停/继续/取消任务
- [x] 基础设置可以保存

---

## Phase 3: 完善功能

### Phase 3 目标

完善所有下载相关功能，包括流选择、高级设置、历史记录等

### Phase 3 任务清单

#### 3.1 URL 解析功能

- [x] Rust `parse_url` 命令（引擎策略分派）
- [x] 解析结果显示
- [x] StreamInfo 类型定义 (src/domain/stream.ts)

#### 3.2 流选择器

- [x] StreamSelector 组件
  - [x] 视频流列表
  - [x] 音频流列表
  - [x] 字幕流列表
- [x] 流信息展示
  - [x] 分辨率
  - [x] 编码
  - [x] 码率
  - [x] 语言
- [x] 流选择逻辑
  - [x] 单选
  - [x] 正则匹配

#### 3.3 高级设置面板

- [x] SettingsView（4 标签页）
- [x] Nm3u8dlTab — N_m3u8DL-RE 设置（含 ToolManagerCard）
- [x] FfmpegTab — FFmpeg 设置（含 ToolManagerCard）
- [x] GeneralTab — 常规·界面设置
- [x] PresetsTab — 任务预设管理

#### 3.4 命令行参数构建

- [x] 参数构建已移入后端引擎 (`infrastructure/engines/*/args.rs`)
- [x] 前端 `commandBuilder.ts` 已删除

#### 3.5 历史记录

- [x] HistoryRecord 类型定义 (src/domain/task.ts)
- [x] Rust 历史记录存储 (infrastructure/db/repository/history_repo.rs)
- [x] HistoryView 页面
- [x] 历史记录列表
- [x] 重新下载功能（携带原 overrides 快照）
- [x] 清除历史功能

#### 3.6 任务预设

- [x] TaskPreset 类型定义 (src/domain/config.ts)
- [x] 预设 Store（DB 持久化，src/stores/presetStore.ts）
- [x] 预设管理 UI (PresetsTab)
  - [x] 保存当前配置为预设
  - [x] 应用预设
  - [x] 删除预设

#### 3.7 统计面板

- [ ] 下载统计（暂未实现）
- [ ] 会话统计
- [ ] 全局统计

#### 3.8 流排除功能

- [x] 流排除 UI（Nm3u8dlTab 内）
- [x] 排除正则表达式支持

#### 3.9 高级解密选项

- [x] DecryptionSettings 组件
- [x] 密钥配置 (KID:KEY)
- [x] 解密引擎选择（FFmpeg/MP4Decrypt/Shaka）
- [x] 自定义 HLS 解密

#### 3.10 日志系统

- [x] LogViewer 组件
- [x] 实时日志显示

#### 3.11 帮助系统

- [ ] 内置帮助文档（暂未实现）
- [ ] 上下文工具提示
- [ ] FAQ 页面

### Phase 3 里程碑

- [x] 可以预览和选择流
- [x] 所有设置功能正常
- [x] 历史记录功能正常
- [x] 预设功能正常
- [x] 流排除功能正常
- [x] 高级解密功能正常
- [x] 日志系统正常
- [ ] 帮助系统待实现

---

## Phase 4: 高级功能

### Phase 4 目标

实现直播、解密、系统托盘等高级功能，完善用户体验

### Phase 4 任务清单

#### 4.1 直播录制

- [x] LiveSettings 组件
  - [x] 实时合并选项
  - [x] 录制时长限制
  - [x] 刷新间隔设置
- [x] 直播模式识别
- [x] 直播状态展示

#### 4.2 解密功能

- [x] DecryptionSettings 组件
  - [x] 密钥输入
  - [x] 密钥文件选择
  - [x] 解密引擎选择
- [x] 解密参数传递（后端引擎 args.rs）

#### 4.3 系统托盘

- [x] Tauri 系统托盘配置 (app/tray.rs)
- [x] 托盘菜单
  - [x] 显示/隐藏窗口
  - [x] 退出
- [x] 最小化到托盘
- [x] 下载完成通知

#### 4.4 剪贴板监控

- [x] 监听剪贴板变化 (useClipboardWatcher)
- [x] 自动识别 M3U8/MPD/MSS 链接

#### 4.5 定时任务

- [x] TaskOverrides.scheduledStartAt 类型定义
- [x] datetime-local UI (AddTaskDialog)
- [x] 定时执行逻辑（useDownloader 30s 轮询调度器）

#### 4.6 自动更新

- [x] 版本检查 (useUpdateChecker + GitHub API)
- [x] 更新提示
- [x] 自动下载更新

#### 4.7 其他优化

- [x] 错误处理优化（AppError + 命令层边界转换）
- [ ] 性能优化（待评估）
- [ ] 崩溃报告（待实现）

#### 4.8 URL 处理增强

- [x] BaseURL 设置 UI (NetworkSettings)
- [x] URL Processor 参数支持（后端引擎）

#### 4.9 直播字幕修正

- [x] 直播字幕修正选项（LiveSettings）

#### 4.10 定时开始

- [x] 任务定时开始功能（AddTaskDialog + useDownloader 调度器）
- [x] 日期时间选择器（原生 datetime-local）

#### 4.11 高级混流选项

- [-] ~~外部媒体导入~~ — 旧实现未接入参数构建（空壳），重构中移除
- [x] concat 分离器选项（后端引擎）

#### 4.12 实验性功能

- [ ] 实验性功能开关（暂未实现）
- [ ] HLS 多 EXT-X-MAP 支持
- [ ] 警告提示 UI

#### 4.13 多语言支持

- [x] i18n 框架集成（vue-i18n）
- [x] 简体中文语言包
- [x] 繁体中文语言包
- [x] 英文语言包
- [x] 语言切换 UI

### Phase 4 里程碑

- [x] 直播录制功能正常
- [x] 解密功能正常
- [x] 系统托盘功能正常
- [x] 自动更新功能正常
- [x] URL 处理增强功能正常
- [x] 定时开始功能正常
- [x] 多语言支持正常
- [ ] 实验性功能待实现

---

## Phase 5: 打包发布

### Phase 5 目标

完善应用打包，准备发布

### Phase 5 任务清单

#### 5.1 图标和资源

- [ ] 应用图标设计
- [ ] 各尺寸图标生成
- [ ] 启动画面（可选）

#### 5.2 打包配置

- [ ] 配置 tauri.conf.json
- [ ] Windows 打包配置
  - [ ] NSIS 安装程序
  - [ ] MSI 安装程序
- [ ] macOS 打包配置（可选）
  - [ ] DMG
  - [ ] App Bundle
- [ ] Linux 打包配置（可选）
  - [ ] AppImage
  - [ ] deb

#### 5.3 测试

- [x] Rust 单元测试（cargo test，96 个测试）
- [x] 前端单元测试（vitest，47 个测试）
- [ ] 功能测试（手动）
- [ ] 兼容性测试
- [ ] 性能测试
- [ ] 安装测试

#### 5.4 文档

- [ ] 用户使用手册
- [x] README 完善
- [ ] 更新日志

#### 5.5 发布

- [ ] GitHub Release
- [ ] 自动构建 CI/CD

### Phase 5 里程碑

- [ ] 应用可正常安装
- [ ] 所有功能正常
- [ ] 文档完善

---

## 开发环境搭建指南

### 1. 安装 Node.js

```bash
# 使用 nvm-windows 安装
nvm install 20
nvm use 20

# 或直接下载安装
# https://nodejs.org/
```

### 2. 安装 Rust

```bash
# Windows: 下载并运行 rustup-init.exe
# https://rustup.rs/

# 验证安装
rustc --version
cargo --version
```

### 3. 安装 VS Code 扩展

创建 `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "Vue.volar",
    "Vue.vscode-typescript-vue-plugin",
    "tauri-apps.tauri-vscode",
    "rust-lang.rust-analyzer",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "bradlc.vscode-tailwindcss"
  ]
}
```

### 4. 安装依赖

```bash
cd StreamGrab

# 前端依赖
npm install

# 验证 Tauri
npm run tauri dev
```

---

## 常用命令

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 前端单元测试
npm test

# 后端测试 + clippy
cd src-tauri && cargo test && cargo clippy -- -D warnings
```

---

## 开发建议

### 代码规范

- 使用 Composition API + `<script setup>`
- 使用 TypeScript 严格模式
- 组件命名使用 PascalCase
- 文件命名使用 camelCase（组件 PascalCase）
- 常量使用 UPPER_SNAKE_CASE

### Git 提交规范

```text
feat: 新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式
refactor: 重构
perf: 性能优化
test: 测试
chore: 构建/工具
```

### 分支策略

```text
main        # 主分支，稳定版本
develop     # 开发分支
feature/*   # 功能分支
fix/*       # 修复分支
release/*   # 发布分支
```

---

## 风险与应对

| 风险           | 影响 | 应对措施                       |
| -------------- | ---- | ------------------------------ |
| Rust 学习曲线  | 高   | 从简单功能开始，逐步深入       |
| Tauri 版本更新 | 中   | 锁定版本，谨慎升级             |
| 跨平台兼容性   | 中   | 优先保证 Windows，其他平台可选 |
| 依赖库问题     | 低   | 选择成熟的库，定期更新         |

---

## 参考资料

### 官方文档

- [Tauri 文档](https://tauri.app/v2/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Pinia 文档](https://pinia.vuejs.org/)
- [TailwindCSS 文档](https://tailwindcss.com/)
- [Rust 学习](https://www.rust-lang.org/learn)

### 推荐学习资源

- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)
- [Tauri + Vue 示例](https://github.com/tauri-apps/tauri/tree/dev/examples)
