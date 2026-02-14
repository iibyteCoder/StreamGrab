# StreamGrab

> 基于 Tauri 2.0 + Vue 3 的现代视频流下载器 GUI

StreamGrab 是 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 的图形化界面封装，提供友好的用户界面来管理流媒体下载任务。

## 功能特性

- 支持 HLS/DASH/MSS 流媒体下载
- 基于 N_m3u8DL-RE 的高性能下载引擎
- 现代化深色主题界面
- 多任务并行下载
- 实时下载进度显示
- 流选择器（选择不同清晰度）
- 模板管理（保存复用配置）
- 系统托盘支持
- 多语言支持（中文/英文）

## 技术栈

- **前端**: Vue 3 + TypeScript + TailwindCSS
- **桌面框架**: Tauri 2.0
- **后端**: Rust
- **下载引擎**: N_m3u8DL-RE

## 开发

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- npm >= 9

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建

```bash
npm run tauri:build
```

## 项目结构

```
src/
├── components/     # UI 组件
├── composables/    # 组合式函数
├── stores/         # Pinia 状态管理
├── services/       # 服务层
├── types/          # TypeScript 类型
├── utils/          # 工具函数
└── views/          # 页面视图

src-tauri/          # Tauri 后端 (Rust)
```

## 许可证

[MIT License](LICENSE)

## 致谢

本项目依赖以下开源项目：

- **[N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE)** - 核心下载引擎，感谢 [nilaoda](https://github.com/nilaoda) 的杰出贡献
- **[Tauri](https://tauri.app/)** - 跨平台桌面应用框架
- **[Vue.js](https://vuejs.org/)** - 渐进式 JavaScript 框架
