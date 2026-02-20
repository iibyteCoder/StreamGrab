# 发行说明模板

本文档定义了 StreamGrab 项目发行说明的格式规范。

## 格式规范

发行说明应包含以下部分（按顺序）：

### 1. 版本概述（可选）

简短描述本版本的主要主题。

```markdown
## 📝 版本概述

本版本主要...
```

### 2. 新增功能

列出新增的功能，使用 emoji 增强可读性。

```markdown
## ✨ 新增

- 🔧 功能描述
- 🎨 功能描述
```

### 3. 优化改进

列出优化和改进的内容。

```markdown
## 🚀 优化

- 优化项描述
- 改进项描述
```

### 4. Bug 修复

列出修复的 bug。

```markdown
## 🐛 修复

- 修复项描述
```

### 5. 代码重构

列出代码重构和架构改进。

```markdown
## 🔨 重构

- 重构项描述
```

### 6. 安装包

列出各平台的安装包。

```markdown
## 📦 安装包

| 平台 | 文件 |
|------|------|
| Windows | `StreamGrab_X.X.X_x64-setup.exe` |
| macOS | `StreamGrab_X.X.X_x64.dmg` |
| Linux | `StreamGrab_X.X.X_amd64.AppImage` |
```

### 7. 致谢

固定内容。

```markdown
## 致谢

感谢 [nilaoda](https://github.com/nilaoda) 开发的 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 下载引擎。
```

## 常用 Emoji 参考

| Emoji | 用途 |
|-------|------|
| ✨ | 新增功能 |
| 🐛 | Bug 修复 |
| 🚀 | 性能优化 |
| 🔨 | 代码重构 |
| 📝 | 文档更新 |
| 🎨 | UI/UX 改进 |
| 🔧 | 配置/设置 |
| 📦 | 依赖更新 |
| 🗄️ | 数据库变更 |
| 🌐 | 网络/国际化 |
| 🔒 | 安全相关 |
| 💅 | 样式改进 |
| ♻️ | 代码清理 |
| 📱 | 组件相关 |

## 示例

```markdown
# StreamGrab v0.4.0

## ✨ 新增

- 📊 进度图表组件，支持实时下载速率曲线显示
- 🔔 通知系统，下载完成/失败时发送系统通知
- 🎯 广告关键词管理组件

## 🚀 优化

- 优化任务详情面板布局
- 改进媒体信息存储逻辑

## 🐛 修复

- 修复并发下载时媒体信息混乱问题
- 修复文件大小不显示问题

## 🔨 重构

- 拆分大型组件为更小的可复用单元
- 重构后端下载命令模块

## 📦 安装包

| 平台 | 文件 |
|------|------|
| Windows | `StreamGrab_0.4.0_x64-setup.exe` |
| macOS | `StreamGrab_0.4.0_x64.dmg` |
| Linux | `StreamGrab_0.4.0_amd64.AppImage` |

## 致谢

感谢 [nilaoda](https://github.com/nilaoda) 开发的 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 下载引擎。
```
