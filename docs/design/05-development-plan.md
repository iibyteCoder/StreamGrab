# 开发计划

## 开发阶段总览

```
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

### 目标

搭建完整的项目框架，实现基础 UI 布局

### 任务清单

#### 1.1 环境准备

- [ ] 安装 Node.js 18+
- [ ] 安装 pnpm
- [ ] 安装 Rust (rustup)
- [ ] 安装 VS Code 及扩展
  - [ ] Vue - Official
  - [ ] Tauri
  - [ ] rust-analyzer
  - [ ] ESLint
  - [ ] Prettier
  - [ ] Tailwind CSS IntelliSense

#### 1.2 创建项目

- [ ] 使用 `pnpm create tauri-app` 创建项目
- [ ] 配置 Vue 3 + TypeScript + Vite
- [ ] 安装依赖包
  ```bash
  pnpm add vue-router pinia @vueuse/core
  pnpm add -D tailwindcss postcss autoprefixer
  pnpm add -D @tauri-apps/plugin-shell @tauri-apps/plugin-fs
  ```

#### 1.3 配置开发工具

- [ ] 配置 TailwindCSS
- [ ] 配置 ESLint + Prettier
- [ ] 配置 TypeScript (tsconfig.json)
- [ ] 配置路径别名 (@/)

#### 1.4 项目结构搭建

- [ ] 创建目录结构 (按 04-architecture.md)
- [ ] 配置 Vue Router 路由
- [ ] 配置 Pinia Store
- [ ] 创建基础布局组件

#### 1.5 UI 基础组件

- [ ] Button 组件
- [ ] Input 组件
- [ ] Select 组件
- [ ] Switch 组件
- [ ] Modal 组件
- [ ] Toast 组件

#### 1.6 主题系统

- [ ] CSS 变量定义
- [ ] 深色主题样式
- [ ] 浅色主题样式
- [ ] 主题切换功能

#### 1.7 布局组件

- [ ] TitleBar 标题栏
- [ ] MainLayout 主布局
- [ ] Sidebar 侧边栏（可选）

### 里程碑

- [ ] 项目可正常启动
- [ ] 基础布局显示正常
- [ ] 主题切换功能正常

---

## Phase 2: 核心功能

### 目标

实现下载核心功能，包括 URL 输入、下载执行、进度显示、任务管理

### 任务清单

#### 2.1 URL 输入模块

- [ ] UrlInput 组件
  - [ ] 输入框基础功能
  - [ ] URL 验证
  - [ ] 剪贴板粘贴
- [ ] BatchImport 批量导入组件
  - [ ] TXT 文件导入
  - [ ] 多行文本输入
- [ ] DropZone 拖放区域（可选）

#### 2.2 任务数据模型

- [ ] 定义 Task 类型 (types/task.ts)
- [ ] 定义 TaskProgress 类型
- [ ] 定义 TaskConfig 类型
- [ ] 创建 taskStore

#### 2.3 Rust 后端 - 下载命令

- [ ] 创建 `src-tauri/src/commands/` 目录
- [ ] 实现 `start_download` 命令
  ```rust
  #[tauri::command]
  async fn start_download(task: Task, app: AppHandle) -> Result<(), String>
  ```
- [ ] 实现 `stop_download` 命令
- [ ] 实现进程管理器 (ProcessManager)
- [ ] 实现输出解析器

#### 2.4 前端下载服务

- [ ] 创建 downloaderService
- [ ] 封装 Tauri invoke 调用
- [ ] 实现事件订阅机制

#### 2.5 任务队列 UI

- [ ] TaskQueue 组件
- [ ] TaskCard 组件
  - [ ] 显示任务信息
  - [ ] 显示下载进度
  - [ ] 显示下载速度
  - [ ] 显示状态
- [ ] ProgressBar 组件
- [ ] TaskActions 操作按钮
  - [ ] 开始/暂停
  - [ ] 取消
  - [ ] 打开目录

#### 2.6 任务控制逻辑

- [ ] useDownloader composable
- [ ] useTasks composable
- [ ] 任务添加/删除
- [ ] 任务暂停/继续
- [ ] 任务取消
- [ ] 并发控制

#### 2.7 基础设置

- [ ] 定义 Settings 类型 (types/settings.ts)
- [ ] 创建 settingsStore
- [ ] Rust 配置读写命令
  - [ ] `load_settings`
  - [ ] `save_settings`
- [ ] QuickSettings 快速设置面板
  - [ ] 保存目录选择
  - [ ] 格式选择
  - [ ] 自动选择开关

#### 2.8 命名模板系统 (新增)

- [ ] SavePatternSettings 类型定义
- [ ] 预设模板列表
- [ ] 命名模板 UI 组件
- [ ] 与下载任务集成

#### 2.9 广告过滤基础 (新增)

- [ ] AdFilterSettings 类型定义
- [ ] 预设规则列表
- [ ] 广告过滤开关 UI

### 里程碑

- [ ] 可以输入 URL 并开始下载
- [ ] 进度实时更新显示
- [ ] 可以暂停/继续/取消任务
- [ ] 基础设置可以保存
- [ ] 命名模板功能正常
- [ ] 广告过滤功能正常

---

## Phase 3: 完善功能

### 目标

完善所有下载相关功能，包括流选择、高级设置、历史记录等

### 任务清单

#### 3.1 URL 解析功能

- [ ] Rust `parse_url` 命令
- [ ] 解析结果显示
- [ ] StreamInfo 类型定义

#### 3.2 流选择器

- [ ] StreamSelector 组件
  - [ ] 视频流列表
  - [ ] 音频流列表
  - [ ] 字幕流列表
- [ ] 流信息展示
  - [ ] 分辨率
  - [ ] 编码
  - [ ] 码率
  - [ ] 语言
- [ ] 流选择逻辑
  - [ ] 单选
  - [ ] 多选
  - [ ] 正则匹配

#### 3.3 高级设置面板

- [ ] SettingsPanel 组件
- [ ] BasicSettings 基础设置
  - [ ] 线程数
  - [ ] 重试次数
  - [ ] 超时时间
  - [ ] 限速设置
- [ ] MuxSettings 混流设置
  - [ ] 输出格式
  - [ ] 混流程序
  - [ ] 程序路径
- [ ] ProxySettings 代理设置
  - [ ] 系统代理
  - [ ] 自定义代理
  - [ ] 请求头管理
- [ ] SettingsModal 设置弹窗

#### 3.4 命令行参数构建

- [ ] commandBuilder 工具函数
- [ ] 支持所有参数映射
- [ ] 参数验证

#### 3.5 历史记录

- [ ] HistoryRecord 类型定义
- [ ] Rust 历史记录存储
- [ ] HistoryView 页面
- [ ] 历史记录列表
- [ ] 重新下载功能
- [ ] 清除历史功能

#### 3.6 配置模板

- [ ] ConfigTemplate 类型定义
- [ ] 模板存储
- [ ] 模板管理 UI
  - [ ] 保存当前配置为模板
  - [ ] 应用模板
  - [ ] 删除模板

#### 3.7 统计面板

- [ ] 下载统计
  - [ ] 总任务数
  - [ ] 完成数
  - [ ] 失败数
  - [ ] 总大小
- [ ] 会话统计
- [ ] 全局统计

#### 3.8 流排除功能 (新增)

- [ ] StreamExclusionSettings 类型定义
- [ ] 流排除 UI 组件
- [ ] 与流选择器集成
- [ ] 排除正则表达式支持

#### 3.9 高级解密选项 (新增)

- [ ] CustomHlsDecryption 类型定义
- [ ] HLS 加密方法选择
- [ ] 自定义 Key/IV 输入
- [ ] 密钥格式支持 (HEX/Base64/文件)

#### 3.10 日志系统 (新增)

- [ ] LogSettings 类型定义
- [ ] 日志级别设置 UI
- [ ] Rust 日志输出处理
- [ ] 日志查看器组件

#### 3.11 帮助系统 (新增)

- [ ] 内置帮助文档
- [ ] 上下文工具提示
- [ ] CLI 参数说明页面
- [ ] FAQ 页面

### 里程碑

- [ ] 可以预览和选择流
- [ ] 所有设置功能正常
- [ ] 历史记录功能正常
- [ ] 配置模板功能正常
- [ ] 流排除功能正常
- [ ] 高级解密功能正常
- [ ] 日志系统正常
- [ ] 帮助系统正常

---

## Phase 4: 高级功能

### 目标

实现直播、解密、系统托盘等高级功能，完善用户体验

### 任务清单

#### 4.1 直播录制

- [ ] LiveSettings 组件
  - [ ] 实时合并选项
  - [ ] 录制时长限制
  - [ ] 刷新间隔设置
- [ ] 直播模式识别
- [ ] 直播状态展示
- [ ] 定时录制功能

#### 4.2 解密功能

- [ ] DecryptSettings 组件
  - [ ] 密钥输入
  - [ ] 密钥文件选择
  - [ ] 解密引擎选择
- [ ] 解密参数传递

#### 4.3 系统托盘

- [ ] Tauri 系统托盘配置
- [ ] 托盘菜单
  - [ ] 显示/隐藏窗口
  - [ ] 快速添加任务
  - [ ] 退出
- [ ] 最小化到托盘
- [ ] 下载完成通知

#### 4.4 剪贴板监控

- [ ] 监听剪贴板变化
- [ ] 自动识别 M3U8 链接
- [ ] 弹窗提示添加

#### 4.5 定时任务

- [ ] ScheduledTask 类型定义
- [ ] 定时任务管理
- [ ] 定时执行逻辑

#### 4.6 自动更新

- [ ] 版本检查
- [ ] 更新提示
- [ ] 自动下载更新

#### 4.7 其他优化

- [ ] 错误处理优化
- [ ] 性能优化
- [ ] 崩溃报告

#### 4.8 URL 处理增强 (新增)

- [ ] BaseURL 设置 UI
- [ ] URL 参数附加功能
- [ ] URL Processor 参数支持

#### 4.9 直播字幕修正 (新增)

- [ ] 直播字幕修正选项
- [ ] VTT 时间轴校正

#### 4.10 定时开始 (新增)

- [ ] 任务定时开始功能
- [ ] 日期时间选择器
- [ ] 快速时间选择

#### 4.11 高级混流选项 (新增)

- [ ] 不写入日期选项
- [ ] concat 分离器选项
- [ ] 外部媒体导入功能

#### 4.12 实验性功能 (新增)

- [ ] 实验性功能开关
- [ ] HLS 多 EXT-X-MAP 支持
- [ ] 警告提示 UI

#### 4.13 多语言支持 (新增)

- [ ] i18n 框架集成
- [ ] 中文语言包
- [ ] 英文语言包
- [ ] 语言切换 UI

### 里程碑

- [ ] 直播录制功能正常
- [ ] 解密功能正常
- [ ] 系统托盘功能正常
- [ ] 自动更新功能正常
- [ ] URL 处理增强功能正常
- [ ] 定时开始功能正常
- [ ] 高级混流功能正常
- [ ] 多语言支持正常

---

## Phase 5: 打包发布

### 目标

完善应用打包，准备发布

### 任务清单

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

- [ ] 功能测试
- [ ] 兼容性测试
- [ ] 性能测试
- [ ] 安装测试

#### 5.4 文档

- [ ] 用户使用手册
- [ ] README 完善
- [ ] 更新日志

#### 5.5 发布

- [ ] GitHub Release
- [ ] 自动构建 CI/CD

### 里程碑

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

### 2. 安装 pnpm

```bash
npm install -g pnpm
```

### 3. 安装 Rust

```bash
# Windows: 下载并运行 rustup-init.exe
# https://rustup.rs/

# 验证安装
rustc --version
cargo --version
```

### 4. 安装 VS Code 扩展

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

### 5. 创建项目

```bash
# 创建 Tauri 项目
pnpm create tauri-app m3u8-downloader-pro

# 选择配置:
# - Package manager: pnpm
# - UI template: Vue
# - UI flavor: TypeScript
```

### 6. 安装依赖

```bash
cd m3u8-downloader-pro

# 前端依赖
pnpm add vue-router pinia @vueuse/core
pnpm add lucide-vue-next clsx tailwind-merge class-variance-authority

# 开发依赖
pnpm add -D tailwindcss postcss autoprefixer
pnpm add -D @types/node

# Tauri 插件
pnpm add @tauri-apps/plugin-shell
pnpm add @tauri-apps/plugin-fs
pnpm add @tauri-apps/plugin-dialog
pnpm add @tauri-apps/plugin-notification

# 初始化 TailwindCSS
pnpm dlx tailwindcss init -p
```

---

## 常用命令

```bash
# 开发模式
pnpm tauri dev

# 构建
pnpm tauri build

# 仅构建前端
pnpm build

# 类型检查
pnpm type-check

# 代码检查
pnpm lint

# 格式化
pnpm format
```

---

## 开发建议

### 代码规范

- 使用 Composition API + `<script setup>`
- 使用 TypeScript 严格模式
- 组件命名使用 PascalCase
- 文件命名使用 kebab-case
- 常量使用 UPPER_SNAKE_CASE

### Git 提交规范

```
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

```
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

