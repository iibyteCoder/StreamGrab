# 更新日志

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

- 首次发布

## [0.1.0] - 2025-02-14

### 新增

- M3U8/MPD/MSS 视频流下载支持
- 多线程下载，可配置线程数
- 实时进度显示，包含下载速度和剩余时间
- 批量导入 URL（支持从文本文件导入）
- 可自定义下载设置（线程数、重试次数、超时时间）
- 混流设置（输出格式、混流器选择）
- 直播流录制支持
- 代理配置（支持 HTTP/HTTPS/SOCKS）
- 自定义请求头支持
- 深色主题界面
- 任务管理（暂停、继续、重试、删除）
- 下载历史记录
- 基于 SQLite 的持久化存储

### 技术栈

- 使用 Tauri 2.0 + Vue 3 + TypeScript 构建
- 跨平台支持（Windows、macOS、Linux）

[未发布]: https://github.com/iibyteCoder/StreamGrab/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/iibyteCoder/StreamGrab/releases/tag/v0.1.0
