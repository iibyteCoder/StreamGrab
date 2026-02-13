# M3U8 下载管理器

一个简洁高效的 M3U8 视频批量下载工具，基于 PowerShell 7.2+ 构建。

## 功能特性

- 批量下载 M3U8 视频流
- 自动合并音视频文件
- 实时进度显示和日志记录
- 失败自动重试机制
- 模块化架构，易于扩展
- 交互式菜单 + 命令行两种使用方式

## 快速开始

### 方式一：交互式菜单（推荐）

```powershell
pwsh -File run.ps1
```

### 方式二：命令行参数

```powershell
# 下载视频
pwsh -File run.ps1 download

# 合并音视频
pwsh -File run.ps1 merge

# 清理临时文件
pwsh -File run.ps1 clean

# 一键执行（下载 + 合并）
pwsh -File run.ps1 all
```

### 方式三：使用旧版脚本

旧版脚本仍保留在根目录，可以继续使用：

```powershell
pwsh -File DownloadVideo.ps1
pwsh -File merge_audio_video.ps1
pwsh -File run_all.ps1
```

## 目录结构

```
./
├── run.ps1                     # 主入口脚本（新版）
│
├── src/                        # 源码目录（新版）
│   ├── Download-Video.ps1      # 下载模块
│   ├── Merge-AudioVideo.ps1    # 合并模块
│   └── Clear-Workspace.ps1     # 清理模块
│
├── modules/                    # 共享模块
│   ├── UI.psm1                 # UI 组件（颜色、进度条等）
│   ├── Config.psm1             # 配置管理
│   └── Logger.psm1             # 日志记录
│
├── config/                     # 配置文件
│   └── settings.json           # 主配置文件
│
├── logs/                       # 日志目录
├── output/                     # 输出目录
│
├── DownloadVideo.ps1           # 旧版下载脚本（保留）
├── merge_audio_video.ps1       # 旧版合并脚本（保留）
├── clean_all.ps1               # 旧版清理脚本（保留）
├── run_all.ps1                 # 旧版一键运行（保留）
│
├── m3u8.txt                    # 任务列表
├── N_m3u8DL-RE.exe             # 下载器
├── ffmpeg.exe                  # 视频处理工具
│
├── .gitignore
├── .editorconfig
└── README.md
```

## 配置说明

首次运行会自动创建 `config/settings.json`：

```json
{
  "version": "1.0.0",
  "paths": {
    "inputFile": "./m3u8.txt",
    "tempDir": "./video",
    "outputDir": "./output",
    "logDir": "./logs"
  },
  "tools": {
    "downloader": "./N_m3u8DL-RE.exe",
    "ffmpeg": "./ffmpeg.exe"
  },
  "download": {
    "autoSelect": true,
    "retryCount": 3,
    "retryDelay": 5
  },
  "merge": {
    "codec": "copy",
    "overwrite": true
  },
  "ui": {
    "showProgress": true,
    "beepOnComplete": true
  }
}
```

### 配置项说明

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `paths.inputFile` | 任务列表文件 | `./m3u8.txt` |
| `paths.tempDir` | 临时下载目录 | `./video` |
| `paths.outputDir` | 最终输出目录 | `./output` |
| `download.retryCount` | 失败重试次数 | `3` |
| `download.retryDelay` | 重试间隔（秒） | `5` |
| `merge.codec` | 合并编码方式 | `copy`（无损） |
| `ui.beepOnComplete` | 完成时播放音效 | `true` |

## m3u8.txt 格式

支持两种格式：

```
# 纯 URL（自动命名）
https://example.com/video1.m3u8

# URL + 自定义文件名（空格分隔）
https://example.com/video2.m3u8 我的视频
```

以 `#` 开头的行会被忽略。

## 命令行参数

### run.ps1

```powershell
./run.ps1 [-Action] <action> [-Config <path>] [-NoBeep]

参数:
  -Action    操作类型: download, merge, clean, all, menu, config
  -Config    配置文件路径
  -NoBeep    禁用音效提示

示例:
  ./run.ps1                    # 交互式菜单
  ./run.ps1 download           # 下载视频
  ./run.ps1 all -NoBeep        # 一键执行，静默模式
```

### Download-Video.ps1

```powershell
./src/Download-Video.ps1 [-InputFile <path>] [-OutputDir <path>] [-RetryCount <n>]
```

### Clear-Workspace.ps1

```powershell
./src/Clear-Workspace.ps1 [-Targets <dirs>] [-WhatIf] [-Force]

参数:
  -WhatIf     预览模式，不实际删除
  -Force      跳过确认提示

示例:
  ./src/Clear-Workspace.ps1 -WhatIf              # 预览将删除的内容
  ./src/Clear-Workspace.ps1 -Targets "Logs,temp" -Force
```

## 系统要求

- **PowerShell**: 7.2 或更高版本
- **操作系统**: Windows 10/11 或 Windows Server
- **终端**: Windows Terminal（推荐）或支持 ANSI 转义序列的终端

### 安装 PowerShell 7

```powershell
# 使用 winget 安装
winget install Microsoft.PowerShell

# 或从 Microsoft Store 安装
```

## 常见问题

### Q: 进度条显示异常？

A: 请使用支持 ANSI 转义序列的现代终端，如：
- Windows Terminal
- VS Code 终端
- JetBrains Terminal

### Q: 下载失败？

A: 检查以下项目：
1. `N_m3u8DL-RE.exe` 是否存在
2. URL 是否有效且可访问
3. 查看日志文件获取详细错误信息

### Q: 合并失败？

A: 确保：
1. `ffmpeg.exe` 存在
2. 视频和音频文件名匹配（仅扩展名不同）
3. 视频文件为 `.mp4`，音频文件为 `.m4a`

### Q: 如何回退到旧版本？

A: 直接使用根目录下的旧版脚本即可：
```powershell
pwsh -File DownloadVideo.ps1
```

## 更新日志

### v1.0.0 (当前版本)

- 重构为模块化架构
- 添加统一配置文件
- 添加交互式菜单
- 修复安全漏洞（移除 Invoke-Expression）
- 添加失败重试机制
- 添加日志文件记录
- 统一 UI 组件和颜色主题

## 许可证

本项目仅供个人学习和研究使用。

## 致谢

- [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) - 强大的 M3U8 下载器
- [FFmpeg](https://ffmpeg.org/) - 多媒体处理框架
