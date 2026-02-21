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

StreamGrab 是 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 和 [FFmpeg](https://ffmpeg.org/) 的图形化界面封装，支持 HLS(m3u8)、DASH(mpd)、MSS 流媒体协议以及普通 HTTP 视频链接下载。适用于下载网络视频课程、直播回放、直链视频等场景，提供任务管理、模板配置、代理设置等便捷功能。

## ✨ 功能特性

### 核心功能

- 🚀 **高性能下载** - 基于 N_m3u8DL-RE / FFmpeg 引擎，多线程并发下载
- 📋 **多任务管理** - 支持多个下载任务并行处理
- 📊 **实时进度** - 下载速度、进度条、剩余时间实时显示
- 🎬 **流选择器** - 自动解析并选择不同清晰度/码率
- 🔗 **HTTP 直链** - 支持 FFmpeg 下载普通 HTTP 视频链接

### 进阶功能

- 📁 **模板管理** - 保存常用配置，一键复用
- 🔧 **高级设置** - 自定义下载参数、代理、Headers 等
- 🌙 **深色主题** - 现代化暗色 UI，护眼舒适
- 🖥️ **系统托盘** - 最小化到托盘，后台运行
- 🌐 **多语言** - 支持中文/英文界面

## 📸 截图

<!-- 在此处添加应用截图 -->
<!-- ![主界面](docs/screenshots/main.png) -->
<!-- ![下载中](docs/screenshots/downloading.png) -->

## 📥 安装

### 下载安装包

前往 [Releases](../../releases) 页面下载对应平台的安装包：

| 平台     | 格式                      |
| -------- | ------------------------- |
| Windows  | `.msi` / `.exe`           |
| macOS    | `.dmg`                    |
| Linux    | `.AppImage` / `.deb`      |

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

| 格式  | 引擎        | 说明                               |
| ----- | ----------- | ---------------------------------- |
| HLS   | N_m3u8DL-RE | `.m3u8` 流媒体链接                 |
| DASH  | N_m3u8DL-RE | `.mpd` 流媒体链接                  |
| MSS   | N_m3u8DL-RE | Smooth Streaming 链接              |
| HTTP  | FFmpeg      | 普通视频直链（.mp4/.mkv/.avi 等）  |

## 🛠️ 开发

### 环境要求

| 工具    | 版本     |
| ------- | -------- |
| Node.js | >= 18    |
| Rust    | >= 1.70  |
| npm     | >= 9     |

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
```

### 构建

```bash
# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 🏗️ 项目结构

```
StreamGrab/
├── src/                        # Vue 前端源码
│   ├── components/             # UI 组件
│   │   ├── common/             # 通用组件
│   │   ├── task/               # 任务相关组件
│   │   └── settings/           # 设置组件
│   ├── composables/            # 组合式函数
│   ├── stores/                 # Pinia 状态管理
│   ├── services/               # 服务层 (Tauri 命令封装)
│   ├── types/                  # TypeScript 类型定义
│   ├── utils/                  # 工具函数
│   └── views/                  # 页面视图
│
├── src-tauri/                  # Tauri 后端源码
│   └── src/
│       ├── app/                # 应用层
│       ├── domain/             # 领域层
│       ├── infrastructure/     # 基础设施层
│       └── shared/             # 共享模块
│
└── docs/                       # 文档
    ├── design/                 # 设计文档
    └── releases/               # 发行说明
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

| 项目                                                   | 说明                                                                   |
| ------------------------------------------------------ | ---------------------------------------------------------------------- |
| [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE)  | 流媒体下载引擎，感谢 [nilaoda](https://github.com/nilaoda) 的杰出贡献  |
| [FFmpeg](https://ffmpeg.org/)                          | 强大的多媒体处理框架，用于 HTTP 直链下载                               |
| [Tauri](https://tauri.app/)                            | 构建更小、更快、更安全的桌面应用                                       |
| [Vue.js](https://vuejs.org/)                           | 渐进式 JavaScript 框架                                                 |
| [TailwindCSS](https://tailwindcss.com/)                | 实用优先的 CSS 框架                                                    |
| [shadcn-vue](https://www.shadcn-vue.com/)              | 精美的 UI 组件库                                                       |

---

<div align="center">

如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！

</div>
