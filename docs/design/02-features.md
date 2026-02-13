# 功能规格说明

## 功能模块总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        功能模块架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐        │
│  │  输入   │   │  解析   │   │  下载   │   │  处理   │        │
│  │  模块   │──▶│  模块   │──▶│  模块   │──▶│  模块   │        │
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘        │
│       │             │             │             │              │
│       └─────────────┴─────────────┴─────────────┘              │
│                           │                                     │
│                    ┌──────┴──────┐                             │
│                    │   管理模块   │                             │
│                    └─────────────┘                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 一、输入模块

### 1.1 URL 输入

| 功能           | 描述                         | 优先级 |
| -------------- | ---------------------------- | ------ |
| 单链接输入     | 文本框直接粘贴 URL           | P0     |
| 多链接批量输入 | 换行分隔多个 URL             | P0     |
| 从文件导入     | 读取 TXT 文件中的 URL 列表   | P1     |
| 剪贴板自动检测 | 自动识别剪贴板中的 M3U8 链接 | P2     |
| 拖拽输入       | 拖拽链接或文件到窗口         | P2     |

### 1.2 URL 格式支持

```
支持格式:
├── M3U8 (HLS)
│   ├── https://example.com/video.m3u8
│   └── https://example.com/master.m3u8 (多码率)
│
├── MPD (DASH)
│   └── https://example.com/video.mpd
│
├── MSS (Smooth Streaming)
│   └── https://example.com/video.ism/Manifest
│
└── 本地文件
    └── C:\path\to\playlist.m3u8
```

### 1.3 任务信息输入

| 字段   | 说明                       | 必填           |
| ------ | -------------------------- | -------------- |
| URL    | 视频流地址                 | 是             |
| 文件名 | 保存的文件名（不含扩展名） | 否（自动生成） |
| 分组   | 任务分组标签               | 否             |
| 备注   | 任务备注信息               | 否             |

---

## 二、解析模块

### 2.1 流信息解析

解析 M3U8/MPD 后获取的信息：

```typescript
interface StreamInfo {
  // 视频流
  videos: VideoStream[];

  // 音频流
  audios: AudioStream[];

  // 字幕流
  subtitles: SubtitleStream[];

  // 元数据
  duration: number; // 总时长（秒）
  segmentCount: number; // 分片数量
  isLive: boolean; // 是否直播
  isEncrypted: boolean; // 是否加密
}

interface VideoStream {
  id: string;
  resolution: string; // "1920x1080"
  bandwidth: number; // 比特率 bps
  codecs: string; // "avc1.64001f"
  frameRate: number; // 帧率
  hdr: string; // "SDR" | "HDR10" | "DV"
  language: string; // 语言代码
  name: string; // 显示名称
}

interface AudioStream {
  id: string;
  language: string;
  name: string;
  codecs: string;
  channels: string; // "2" | "6"
  sampleRate: number;
  bandwidth: number;
}

interface SubtitleStream {
  id: string;
  language: string;
  name: string;
  format: string; // "vtt" | "srt"
  isDefault: boolean;
}
```

### 2.2 流选择策略

| 策略         | 描述                        | 适用场景 |
| ------------ | --------------------------- | -------- |
| 自动选择最佳 | 选择最高码率视频 + 默认音频 | 普通用户 |
| 手动选择     | 用户逐个勾选流              | 进阶用户 |
| 正则匹配     | 使用正则表达式匹配          | 专业用户 |
| 模板选择     | 使用预设模板选择            | 批量下载 |

### 2.3 预览功能

- 显示所有可用流及其详细信息
- 预计下载大小
- 预计下载时间
- 是否需要解密

---

## 三、下载模块

### 3.1 下载配置

| 配置项             | 类型    | 默认值    | 说明                   |
| ------------------ | ------- | --------- | ---------------------- |
| threadCount        | number  | CPU核心数 | 下载线程数             |
| retryCount         | number  | 3         | 分片下载失败重试次数   |
| retryDelay         | number  | 5         | 重试间隔（秒）         |
| timeout            | number  | 100       | HTTP 请求超时（秒）    |
| maxSpeed           | string  | "0"       | 最大下载速度（0=不限） |
| concurrentDownload | boolean | false     | 并发下载音视频         |

### 3.2 下载状态

```typescript
type TaskStatus =
  | "pending" // 等待中
  | "analyzing" // 解析中
  | "downloading" // 下载中
  | "paused" // 已暂停
  | "merging" // 合并中
  | "muxing" // 混流中
  | "completed" // 已完成
  | "failed" // 失败
  | "cancelled"; // 已取消
```

### 3.3 进度信息

```typescript
interface DownloadProgress {
  // 基础进度
  downloadedSegments: number;
  totalSegments: number;
  percentage: number;

  // 速度信息
  speed: number; // B/s
  speedFormatted: string; // "5.2 MB/s"

  // 大小信息
  downloadedBytes: number;
  totalBytes: number; // 可能为 0（未知）

  // 时间信息
  elapsedTime: number; // 已用时间（秒）
  estimatedTime: number; // 预计剩余时间（秒）

  // 当前状态
  currentAction: string; // "正在下载分片 123/456"
}
```

### 3.4 下载控制

| 操作 | 说明                   |
| ---- | ---------------------- |
| 开始 | 开始/继续下载          |
| 暂停 | 暂停当前下载           |
| 取消 | 取消下载并清理临时文件 |
| 重试 | 失败后重新下载         |
| 优先 | 提升队列优先级         |

### 3.5 范围下载

```
支持的范围格式:
├── 分片序号
│   ├── 0-10      (下载前 11 个分片)
│   ├── 10-       (从第 11 个分片开始)
│   └── -99       (下载前 100 个分片)
│
└── 时间范围
    └── 05:00-20:00  (下载 5 分钟到 20 分钟的内容)
```

---

## 四、处理模块

### 4.1 解密功能

| 配置项   | 说明                                 |
| -------- | ------------------------------------ |
| 解密引擎 | FFmpeg / MP4Decrypt / Shaka Packager |
| 密钥格式 | KID:KEY 或纯 KEY                     |
| 密钥文件 | 从文件读取密钥                       |
| 实时解密 | 下载时实时解密                       |

### 4.2 合并功能

| 配置项       | 说明                        |
| ------------ | --------------------------- |
| 合并方式     | 自动合并 / 二进制合并       |
| 合并程序     | FFmpeg / 内置               |
| 合并协议     | concat 协议 / concat 分离器 |
| 删除临时文件 | 合并后删除分片              |

### 4.3 混流功能

```typescript
interface MuxOptions {
  format: "mp4" | "mkv"; // 输出格式
  muxer: "ffmpeg" | "mkvmerge"; // 混流程序
  binPath?: string; // 程序路径
  skipSubtitles: boolean; // 跳过字幕
  keepOriginal: boolean; // 保留原文件
}

// 外部媒体导入
interface MuxImport {
  path: string; // 文件路径
  lang?: string; // 语言代码
  name?: string; // 描述信息
}
```

### 4.4 字幕处理

| 配置项   | 说明               |
| -------- | ------------------ |
| 字幕格式 | SRT / VTT          |
| 自动修正 | 自动修正字幕时间轴 |
| 嵌入字幕 | 混流时嵌入字幕轨道 |
| 外挂字幕 | 保留独立字幕文件   |

---

## 五、直播模块

### 5.1 直播配置

| 配置项            | 类型    | 默认值 | 说明               |
| ----------------- | ------- | ------ | ------------------ |
| livePerformAsVod  | boolean | false  | 以点播方式处理     |
| liveRealTimeMerge | boolean | false  | 实时合并           |
| liveKeepSegments  | boolean | true   | 保留分片           |
| livePipeMux       | boolean | false  | 管道实时混流       |
| liveRecordLimit   | string  | ""     | 录制时长限制       |
| liveWaitTime      | number  | 0      | 刷新间隔（0=自动） |
| liveTakeCount     | number  | 16     | 首次获取分片数     |

### 5.2 直播特性

- 自动检测直播流类型
- 支持定时录制
- 支持时长限制
- 支持实时混流输出
- 断流自动重连

---

## 六、网络模块

### 6.1 代理设置

| 配置项     | 说明             |
| ---------- | ---------------- |
| 不使用代理 | 直连             |
| 系统代理   | 使用系统代理设置 |
| 自定义代理 | HTTP/SOCKS 代理  |

### 6.2 请求头设置

```typescript
interface HeaderConfig {
  key: string; // "Cookie"
  value: string; // "session=abc123"
}

// 示例
const headers = [
  { key: "Cookie", value: "session=abc123" },
  { key: "User-Agent", value: "Mozilla/5.0..." },
  { key: "Referer", value: "https://example.com" },
];
```

---

## 七、管理模块

### 7.1 任务队列

| 功能     | 说明                 |
| -------- | -------------------- |
| 队列管理 | 添加、删除、排序任务 |
| 并发控制 | 同时下载任务数限制   |
| 优先级   | 调整任务执行顺序     |
| 批量操作 | 批量开始、暂停、删除 |

### 7.2 历史记录

```typescript
interface HistoryRecord {
  id: string;
  url: string;
  fileName: string;
  savePath: string;
  fileSize: number;
  duration: number;
  completedAt: Date;
  config: TaskConfig; // 使用的配置
}
```

### 7.3 配置模板

```typescript
interface ConfigTemplate {
  id: string;
  name: string; // "B站 1080P"
  description: string;
  settings: Partial<Settings>;
  createdAt: Date;
  updatedAt: Date;
}
```

### 7.4 定时任务

```typescript
interface ScheduledTask {
  id: string;
  taskId: string;
  scheduledTime: Date; // 计划执行时间
  repeat: "none" | "daily" | "weekly";
  enabled: boolean;
}
```

---

## 八、系统集成

### 8.1 系统托盘

- 最小化到托盘
- 托盘菜单快速操作
- 下载完成通知

### 8.2 文件关联

- 关联 .m3u8 文件
- 关联 .mpd 文件
- 右键菜单"使用 M3U8 Downloader 下载"

### 8.3 剪贴板监控

- 监控剪贴板 URL
- 自动识别 M3U8 链接
- 弹窗提示添加任务

---

## 九、命名模板系统

### 9.1 功能概述

支持使用变量模板自定义输出文件名，避免多流下载时的文件名冲突。

### 9.2 支持的变量

| 变量           | 说明               | 示例值            |
| -------------- | ------------------ | ----------------- |
| `<SaveName>`   | 用户指定的保存名称 | "my_video"        |
| `<Id>`         | 流的任务ID         | "video_1"         |
| `<Codecs>`     | 编解码器信息       | "avc1.64001f"     |
| `<Language>`   | 语言代码           | "zh-CN", "en"     |
| `<Resolution>` | 视频分辨率         | "1920x1080"       |
| `<Bandwidth>`  | 流的带宽/比特率    | "5000000"         |
| `<MediaType>`  | 媒体类型           | "VIDEO", "AUDIO"  |
| `<Channels>`   | 音频声道配置       | "2", "6"          |
| `<FrameRate>`  | 帧率               | "30", "60"        |
| `<VideoRange>` | 视频色域/HDR信息   | "SDR", "HDR10"    |
| `<GroupId>`    | 流组标识符         | "group_720p"      |
| `<Ext>`        | 文件扩展名         | "mp4", "m4a"      |

### 9.3 预设模板

| 模板名称   | 模板字符串                                      | 适用场景     |
| ---------- | ----------------------------------------------- | ------------ |
| 基础       | `<SaveName>`                                    | 单流下载     |
| 包含分辨率 | `<SaveName>_<Resolution>`                       | 多清晰度视频 |
| 包含带宽   | `<SaveName>_<Resolution>_<Bandwidth>kbps`       | 专业用户     |
| 多音轨     | `<SaveName>_<Language>_<Channels>ch`            | 多音轨下载   |
| 完整信息   | `<MediaType>_<Resolution>_<Codecs>_<Language>`  | 调试/分析    |

### 9.4 配置类型

```typescript
interface SavePatternSettings {
  enabled: boolean;
  template: string;
  presetId: string | 'custom';
}
```

### 9.5 UI 设计

**位置**: 设置 > 基础设置 > 命名模板
**快速访问**: 任务详情 > 保存选项

### 9.6 CLI 映射

| UI 选项  | CLI 参数       |
| -------- | -------------- |
| 命名模板 | --save-pattern |

---

## 十、广告过滤系统

### 10.1 功能概述

通过正则表达式匹配广告分片的 URL 关键字，自动跳过广告内容。

### 10.2 配置选项

```typescript
interface AdFilterSettings {
  enabled: boolean;
  keywords: string[];      // 正则表达式列表
  action: 'skip' | 'mark'; // 跳过或标记
}
```

### 10.3 预设规则

| 平台 | 正则表达式示例                  | 说明         |
| ---- | -------------------------------- | ------------ |
| 通用 | `ad\.domain\.com`               | 匹配广告域名 |
| 通用 | `\/ad\/|\/ads\/|\/advert`       | 匹配广告路径 |
| 特定 | `doubleclick\.net`              | Google广告   |

### 10.4 UI 设计

**位置**: 设置 > 高级设置 > 广告过滤
**交互**:
- 启用/禁用开关
- 正则表达式列表（可增删）
- 预设规则快速选择
- 测试工具（输入URL验证匹配）

### 10.5 CLI 映射

| UI 选项   | CLI 参数     |
| --------- | ------------ |
| 广告关键字 | --ad-keyword |

---

## 十一、流排除功能

### 11.1 功能概述

通过正则表达式排除不符合需求的视频、音频或字幕流。

### 11.2 排除选项

```typescript
interface StreamExclusionSettings {
  dropVideo: string;    // 视频流排除正则
  dropAudio: string;    // 音频流排除正则
  dropSubtitle: string; // 字幕流排除正则
}
```

### 11.3 使用场景

| 场景         | 参数 | 正则示例                    |
| ------------ | ---- | --------------------------- |
| 排除低画质   | -dv  | `res="480.*"|res="360.*"`   |
| 排除特定语言 | -da  | `lang="ja"`                 |
| 排除强制字幕 | -ds  | `name="forced"`             |

### 11.4 与流选择的关系

```
选择逻辑:
1. 首先应用选择规则 (-sv/-sa/-ss)
2. 然后应用排除规则 (-dv/-da/-ds)
3. 最终确定要下载的流

示例:
-sv best        -> 选择最佳视频
-dv codecs=av01 -> 排除 AV1 编码
结果: 选择最佳非AV1视频
```

### 11.5 UI 设计

**位置**: 流选择器 > 高级选项
**交互**: 与流选择器集成，提供"排除"标签页

### 11.6 CLI 映射

| UI 选项   | CLI 参数 |
| --------- | -------- |
| 排除视频   | -dv      |
| 排除音频   | -da      |
| 排除字幕   | -ds      |

---

## 十二、高级解密选项

### 12.1 功能概述

支持自定义 HLS 加密方法和密钥，适用于非标准加密场景。

### 12.2 HLS 自定义解密

```typescript
interface CustomHlsDecryption {
  enabled: boolean;
  method: 'AES_128' | 'AES_128_ECB' | 'CENC' | 'CHACHA20' |
          'NONE' | 'SAMPLE_AES' | 'SAMPLE_AES_CTR' | 'UNKNOWN';
  key: {
    type: 'file' | 'hex' | 'base64';
    value: string;
  };
  iv: {
    type: 'file' | 'hex' | 'base64';
    value: string;
  };
}
```

### 12.3 加密方法说明

| 方法            | 说明                   |
| --------------- | ---------------------- |
| AES_128         | 标准 AES-128 CBC       |
| AES_128_ECB     | AES-128 ECB 模式       |
| CENC            | 通用加密 (Common Enc)  |
| CHACHA20        | ChaCha20 加密          |
| SAMPLE_AES      | 采样 AES               |
| SAMPLE_AES_CTR  | 采样 AES CTR 模式      |
| NONE            | 无加密                 |
| UNKNOWN         | 未知加密方式           |

### 12.4 密钥输入方式

| 方式     | 说明               | 示例                     |
| -------- | ------------------ | ------------------------ |
| 文件路径 | 从文件读取密钥     | `C:\keys\key.bin`        |
| HEX      | 十六进制字符串     | `0123456789ABCDEF...`    |
| Base64   | Base64 编码字符串  | `ASNFZ4mrze8=`           |

### 12.5 UI 设计

**位置**: 设置 > 解密设置 > 高级解密选项
**交互**:
- 折叠面板（默认收起）
- 加密方法下拉选择
- 密钥/IV 输入（支持文件选择或直接输入）
- 格式切换（HEX/Base64）

### 12.6 CLI 映射

| UI 选项      | CLI 参数            |
| ------------ | ------------------- |
| HLS 加密方法 | --custom-hls-method |
| HLS 解密 KEY | --custom-hls-key    |
| HLS 解密 IV  | --custom-hls-iv     |

---

## 十三、日志系统

### 13.1 功能概述

完整的日志记录和管理系统，支持日志级别设置、日志文件输出和日志查看。

### 13.2 日志级别

| 级别  | 说明                 | 使用场景 |
| ----- | -------------------- | -------- |
| DEBUG | 详细调试信息         | 开发调试 |
| INFO  | 一般信息（默认）     | 日常使用 |
| WARN  | 警告信息             | 问题排查 |
| ERROR | 错误信息             | 故障诊断 |
| OFF   | 关闭日志             | 性能优化 |

### 13.3 日志配置

```typescript
interface LogSettings {
  level: 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'OFF';
  enableFileOutput: boolean;
  logFilePath: string;      // 默认: 应用数据目录
  maxFileSize: number;      // MB
  maxFileCount: number;     // 保留文件数
}
```

### 13.4 日志查看界面

**功能**:
- 实时日志流显示
- 按级别过滤
- 关键字搜索
- 导出日志文件
- 清除日志

### 13.5 UI 设计

**位置**:
- 设置 > 高级设置 > 日志设置
- 主界面 > 帮助 > 查看日志

**交互**:
- 日志级别下拉选择
- 日志文件路径选择
- "查看日志"按钮打开日志查看器
- 日志查看器：终端风格，支持过滤

### 13.6 CLI 映射

| UI 选项     | CLI 参数        |
| ----------- | --------------- |
| 日志级别    | --log-level     |
| 日志文件路径 | --log-file-path |
| 关闭日志    | --no-log        |

---

## 十四、帮助系统

### 14.1 功能概述

内置帮助和参数说明系统，方便用户了解功能和 CLI 参数。

### 14.2 帮助内容

| 内容类型   | 来源             | 展示方式         |
| ---------- | ---------------- | ---------------- |
| 功能说明   | 内置文档         | 工具提示、帮助页 |
| CLI 参数   | N_m3u8DL-RE 帮助 | 设置页内嵌       |
| 常见问题   | 内置/在线        | FAQ 页面         |
| 快捷键     | 内置             | 快捷键列表       |

### 14.3 上下文帮助

- 设置项悬停显示详细说明
- 输入框显示格式提示
- 错误信息附带解决方案链接

### 14.4 UI 设计

**位置**:
- 标题栏 > 帮助图标
- 设置页 > 每个选项的"?"图标

**交互**:
- 点击帮助图标打开帮助面板
- 悬停设置项显示工具提示
- F1 快捷键打开上下文帮助

### 14.5 CLI 映射

| UI 选项     | CLI 参数   |
| ----------- | ---------- |
| 查看更多帮助 | --morehelp |
| 显示帮助    | -h, --help |

---

## 十五、URL 处理增强

### 15.1 BaseURL 设置

**功能**: 手动指定基础 URL，用于分片地址解析

```typescript
interface UrlProcessSettings {
  baseUrl: string;           // --base-url
  appendUrlParams: boolean;  // --append-url-params
}
```

**使用场景**:
- M3U8 文件中分片使用相对路径
- 需要覆盖默认的 BaseURL

### 15.2 URL 参数附加

**功能**: 将输入 URL 的查询参数附加到所有分片请求

**适用网站**: kakao.com 等需要传递 URL 参数的网站

### 15.3 UI 设计

**位置**: 设置 > 高级设置 > URL 处理

**交互**:
- BaseURL 输入框
- "附加URL参数"开关

### 15.4 CLI 映射

| UI 选项     | CLI 参数            |
| ----------- | ------------------- |
| BaseURL     | --base-url          |
| 附加URL参数 | --append-url-params |

---

## 十六、直播字幕修正

### 16.1 功能概述

通过读取音频文件的起始时间修正 VTT 字幕时间轴。

### 16.2 配置

```typescript
interface LiveSubtitleSettings {
  fixVttByAudio: boolean;  // --live-fix-vtt-by-audio
}
```

### 16.3 使用场景

- 直播录制的字幕与音视频不同步
- VTT 字幕时间轴需要根据音频校正

### 16.4 UI 设计

**位置**: 设置 > 直播设置 > 字幕修正

**交互**: 复选框开关

### 16.5 CLI 映射

| UI 选项     | CLI 参数                |
| ----------- | ----------------------- |
| 音频修正字幕 | --live-fix-vtt-by-audio |

---

## 十七、定时开始

### 17.1 功能概述

设置任务在指定时间后才开始执行。

### 17.2 配置

```typescript
interface ScheduleStartSettings {
  startAt: Date | null;  // --task-start-at
  format: 'yyyyMMddHHmmss';
}
```

### 17.3 UI 设计

**位置**: 任务详情 > 高级选项 > 定时开始

**交互**:
- 日期时间选择器
- 快速选择（1小时后、明天8点等）
- 启用/禁用开关

### 17.4 CLI 映射

| UI 选项 | CLI 参数        |
| ------- | --------------- |
| 定时开始 | --task-start-at |

---

## 十八、高级混流选项

### 18.1 新增选项

| 选项           | 说明                       | CLI 参数                    |
| -------------- | -------------------------- | --------------------------- |
| noDateInfo     | 混流时不写入日期信息       | --no-date-info              |
| concatDemuxer  | 使用 concat 分离器而非协议 | --use-ffmpeg-concat-demuxer |

### 18.2 配置更新

```typescript
interface MuxSettings {
  // 现有选项...
  noDateInfo: boolean;
  useConcatDemuxer: boolean;
}
```

### 18.3 UI 设计

**位置**: 设置 > 混流设置 > 高级选项

**交互**: 复选框，带工具提示说明

---

## 十九、实验性功能

### 19.1 功能概述

提供实验性功能开关，用于测试不稳定或高级特性。

### 19.2 当前实验性功能

| 功能              | CLI 参数                   | 说明                      |
| ----------------- | ------------------------- | ------------------------- |
| HLS 多 EXT-X-MAP  | --allow-hls-multi-ext-map | 允许多个 #EXT-X-MAP 标签  |

### 19.3 UI 设计

**位置**: 设置 > 高级设置 > 实验性功能

**警告**: 显示警告提示，说明这些功能可能不稳定

---

## 二十、自动更新功能

### 20.1 功能概述

检查并提示应用更新，可选择自动下载。

### 20.2 配置

```typescript
interface UpdateSettings {
  checkOnStartup: boolean;
  autoDownload: boolean;
  disableCheck: boolean;  // --disable-update-check
}
```

### 20.3 UI 设计

**位置**: 设置 > 通用设置 > 自动更新

**交互**:
- 启动时检查更新开关
- 手动检查更新按钮
- 更新提示弹窗

### 20.4 CLI 映射

| UI 选项     | CLI 参数                |
| ----------- | ----------------------- |
| 禁用更新检测 | --disable-update-check  |

---

## 功能优先级矩阵

```
         高价值
           ▲
           │
    P1     │     P0
  (重要但不紧急) │ (重要且紧急)
           │
───────────┼──────────▶ 高紧急
           │
    P2     │     P3
  (不重要但紧急) │ (不重要不紧急)
           │
           ▼
         低价值
```

| 优先级 | 功能                                                                                           |
| ------ | ---------------------------------------------------------------------------------------------- |
| **P0** | URL输入、单任务下载、基本进度显示、基础设置、任务队列                                          |
| **P1** | 批量导入、流选择、混流设置、代理设置、历史记录、配置模板、**命名模板**、**广告过滤**、**流排除** |
| **P2** | 解密功能、直播录制、剪贴板监控、系统托盘、定时任务、**高级解密**、**日志系统**、**定时开始**    |
| **P3** | 文件关联、主题切换、多语言支持、自动更新、**帮助系统**、**URL增强**、**实验性功能**             |

