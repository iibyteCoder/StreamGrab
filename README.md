# StreamGrab

> 基于 Tauri 2.0 + Vue 3 的现代视频流下载器

## 功能特性

- 支持 HLS/DASH/MSS 流媒体下载
- 基于 N_m3u8DL-RE 的高性能下载
- 现代化深色主题界面
- 多任务并行下载
- 实时下载进度显示

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

MIT
