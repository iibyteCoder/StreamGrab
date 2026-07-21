<div align="center">

# StreamGrab

**现代视频流下载器 | Modern Video Stream Downloader**

基于 Tauri 2.0 + Vue 3 构建的跨平台流媒体下载 GUI 应用

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D8?logo=tauri)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.x-4FC08D?logo=vue.js)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-DEA584?logo=rust)](https://www.rust-lang.org/)

[功能特性](#-功能特性) • [安装](#-安装) • [使用](#-使用) • [开发](#-开发) • [致谢](#-致谢)

</div>

---

## 📖 简介

StreamGrab 是 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 和 [FFmpeg](https://ffmpeg.org/) 的图形化界面封装，支持 HLS(m3u8)、DASH(mpd)、MSS 流媒体协议以及普通 HTTP 视频链接下载。双引擎按 URL 类型自动分派（策略模式），用户无需选择工具。适用于下载网络视频课程、直播回放、直链视频等场景，提供任务管理、任务预设、历史记录、定时开始等便捷功能。

## ✨ 功能特性

### 核心功能

- 🚀 **双引擎自动分派** - N_m3u8DL-RE 处理流媒体（HLS/DASH/MSS），FFmpeg 处理 HTTP 直链，按 URL 类型自动选择
- 📋 **多任务管理** - 支持多个下载任务并行处理，队列并发控制
- 📊 **实时进度** - 下载速度、进度条、剩余时间、速率曲线图表
- 🎬 **流选择器** - 自动解析并选择不同清晰度/码率/音轨/字幕
- 🔗 **HTTP 直链** - 支持 FFmpeg 下载普通 HTTP 视频链接

### 进阶功能

- 📜 **历史记录** - 任务终态自动快照（含参数覆盖），支持查看/删除/重新下载
- ⏰ **定时开始** - 设置计划时间，到点自动启动下载
- 📁 **任务预设** - 保存常用配置组合（DB 持久化），一键复用
- 🔧 **任务级覆盖** - 每个任务可独立覆盖全局默认参数（TaskOverrides）
- 🌙 **深色/浅色主题** - 现代化 UI，主题切换
- 🖥️ **系统托盘** - 最小化到托盘，后台运行
- 🌐 **多语言** - 支持简体中文/繁体中文/英文三语界面
- 🔄 **自动更新** - GitHub API 版本检查，一键下载安装

## 📸 截图

<!-- 在此处添加应用截图 -->
<!-- ![主界面](docs/screenshots/main.png) -->
<!-- ![下载中](docs/screenshots/downloading.png) -->

## 📥 安装

### 下载安装包

前往 [Releases](../../releases) 页面下载对应平台的安装包：

| 平台    | 格式                 |
| ------- | -------------------- |
| Windows | `.msi` / `.exe`      |
| macOS   | `.dmg`               |
| Linux   | `.AppImage` / `.deb` |

### 系统要求

- **Windows**: Windows 10/11 (x64)
- **macOS**: macOS 10.15+ (Intel/Apple Silicon)
- **Linux**: 主流发行版 (x64)

## 🎯 使用

### 快速开始

1. 下载并安装 StreamGrab
2. 启动应用，首次运行会自动检测/下载 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE)
3. 粘贴视频流地址（m3u8/mpd 链接）
4. 点击开始下载

### 支持的链接格式

| 格式 | 引擎        | 说明                              |
| ---- | ----------- | --------------------------------- |
| HLS  | N_m3u8DL-RE | `.m3u8` 流媒体链接                |
| DASH | N_m3u8DL-RE | `.mpd` 流媒体链接                 |
| MSS  | N_m3u8DL-RE | Smooth Streaming 链接             |
| HTTP | FFmpeg      | 普通视频直链（.mp4/.mkv/.avi 等） |

## 🛠️ 开发

### 环境要求

| 工具    | 版本    |
| ------- | ------- |
| Node.js | >= 18   |
| Rust    | >= 1.70 |
| npm     | >= 9    |

### 本地开发

```bash
# 克隆仓库
git clone https://github.com/your-username/StreamGrab.git
cd StreamGrab

# 安装依赖
npm install

# 启动开发模式
npm run tauri dev

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 前端单元测试（vitest）
npm test

# 后端测试 + clippy
cd src-tauri && cargo test && cargo clippy -- -D warnings
```

### 构建

```bash
# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 🏗️ 项目结构

```text
StreamGrab/
├── src/                        # Vue 前端源码
│   ├── domain/                 # 领域类型唯一来源（task/config/stream/url）
│   ├── components/             # UI 组件
│   │   ├── task/               # 任务组件（TaskCard/AddTaskDialog/ProgressChart...）
│   │   ├── settings/           # 设置组件（tabs/ 4 标签页 + ToolManagerCard）
│   │   ├── stream/             # 流选择器
│   │   ├── common/             # 通用组件
│   │   └── ui/                 # shadcn-vue 基础组件
│   ├── composables/            # 组合式函数（useDownloader 含队列+调度器）
│   ├── stores/                 # Pinia 状态管理（task/settings/preset/history）
│   ├── services/               # 服务层（与后端命令组对应的 invoke 封装）
│   ├── utils/                  # 工具函数（format/validate/id）
│   ├── locales/                # i18n 三语（zh-CN/en-US/zh-TW）
│   └── views/                  # 页面视图（Home/Settings/History）
│
├── src-tauri/src/              # Rust 后端（四层架构）
│   ├── app/                    # 应用层：commands/（按域分组）+ tray.rs
│   ├── domain/                 # 领域层：config/task/download(策略契约)/media
│   ├── infrastructure/         # 基础设施：engines/db/process/tools/media/platform/fs
│   └── shared/                 # 共享错误类型（AppError）
│
└── docs/                       # 文档
    └── design/                 # 设计文档（00-07）
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目基于 [GPL-3.0 License](LICENSE) 开源。

## 🙏 致谢

本项目依赖以下优秀的开源项目：

| 项目                                                  | 说明                                                                  |
| ----------------------------------------------------- | --------------------------------------------------------------------- |
| [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) | 流媒体下载引擎，感谢 [nilaoda](https://github.com/nilaoda) 的杰出贡献 |
| [FFmpeg](https://ffmpeg.org/)                         | 强大的多媒体处理框架，用于 HTTP 直链下载                              |
| [Tauri](https://tauri.app/)                           | 构建更小、更快、更安全的桌面应用                                      |
| [Vue.js](https://vuejs.org/)                          | 渐进式 JavaScript 框架                                                |
| [TailwindCSS](https://tailwindcss.com/)               | 实用优先的 CSS 框架                                                   |
| [shadcn-vue](https://www.shadcn-vue.com/)             | 精美的 UI 组件库                                                      |

---

<div align="center">

如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！

</div>
