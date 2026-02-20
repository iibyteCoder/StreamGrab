# 更新日志

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

- 首次发布

## [0.5.2] - 2026-02-20

### 新增

- 软件更新下载和安装功能
  - 检测到新版本后可直接下载更新
  - 自动下载到临时目录并运行安装程序
  - 支持在设置页面查看下载文件位置
  - 支持重新运行安装程序

### 优化

- 改进 Windows 平台安装程序运行兼容性
- 改进文件管理器打开文件位置的准确性

## [0.5.1] - 2026-02-19

### 优化

- 改进 FFmpeg 版本解析，支持 BtbN 构建格式（日期版本号）
- 改进 N_m3u8DL-RE 版本解析，支持更多版本输出格式
- 改进平台资源匹配逻辑，支持组合关键字如 `win64`
- 增加工具检测日志输出，便于调试

### 修复

- 修复工具下载完整性验证
- 修复 ZIP 解压时文件名大小写匹配问题
- 修复部分 FFmpeg 发行版（如 latest 标签）版本检测失败的问题

## [0.5.0] - 2026-02-14

### 新增

- 进度图表组件，支持实时下载速率曲线显示
- 通知系统，下载完成/失败时发送系统通知
- 广告关键词管理组件
- 剪贴板自动检测 M3U8/MPD/MSS 链接
- 系统托盘支持
- 多语言支持（简体中文、繁体中文、英文）
- 浅色/深色主题切换

### 优化

- 优化任务详情面板布局
- 改进媒体信息存储逻辑
- 完善流选择器和配置模板系统

### 修复

- 修复并发下载时媒体信息混乱问题
- 修复文件大小不显示问题
- 修复 minimizeToTray 设置不生效问题

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

[未发布]: https://github.com/iibyteCoder/StreamGrab/compare/v0.5.2...HEAD
[0.5.2]: https://github.com/iibyteCoder/StreamGrab/releases/tag/v0.5.2
[0.5.1]: https://github.com/iibyteCoder/StreamGrab/releases/tag/v0.5.1
[0.5.0]: https://github.com/iibyteCoder/StreamGrab/releases/tag/v0.5.0
[0.1.0]: https://github.com/iibyteCoder/StreamGrab/releases/tag/v0.1.0
