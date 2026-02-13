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

| 优先级 | 功能                                                     |
| ------ | -------------------------------------------------------- |
| **P0** | URL输入、单任务下载、基本进度显示、基础设置、任务队列    |
| **P1** | 批量导入、流选择、混流设置、代理设置、历史记录、配置模板 |
| **P2** | 解密功能、直播录制、剪贴板监控、系统托盘、定时任务       |
| **P3** | 文件关联、主题切换、多语言支持、自动更新                 |

