# CLI 参数与 UI 功能映射表

本文档记录 N_m3u8DL-RE 命令行参数与 StreamGrab UI 功能的完整映射关系。

## 映射说明

| 字段 | 说明 |
| --- | --- |
| CLI 参数 | N_m3u8DL-RE 命令行参数 |
| UI 位置 | StreamGrab 界面中的对应位置 |
| 类型 | 参数类型（文本/数字/布尔/选择/复合） |
| 默认值 | CLI 工具的默认值 |

---

## 基础参数

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `<input>` | URL 输入框 | 必填 | - |
| `--tmp-dir` | 设置 > 基础 > 临时目录 | 文本 | 系统临时目录 |
| `--save-dir` | 设置 > 基础 > 保存目录 | 文本 | 当前目录 |
| `--save-name` | 任务详情 > 文件名 | 文本 | 自动生成 |
| `--save-pattern` | 设置 > 基础 > 命名模板 | 文本 | - |
| `--log-file-path` | 设置 > 高级 > 日志文件路径 | 文本 | - |
| `--base-url` | 设置 > 高级 > BaseURL | 文本 | - |
| `--thread-count` | 设置 > 下载 > 线程数 | 数字 | CPU核心数 |
| `--download-retry-count` | 设置 > 下载 > 重试次数 | 数字 | 3 |
| `--http-request-timeout` | 设置 > 下载 > 超时时间 | 数字 | 100 |
| `--force-ansi-console` | (内部使用) | 布尔 | false |
| `--no-ansi-color` | (内部使用) | 布尔 | false |

---

## 下载控制

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--auto-select` | 设置 > 下载 > 自动选择最佳 | 布尔 | false |
| `--skip-merge` | 设置 > 下载 > 跳过合并 | 布尔 | false |
| `--skip-download` | (仅解析模式) | 布尔 | false |
| `--check-segments-count` | 设置 > 下载 > 检查分片数量 | 布尔 | true |
| `--binary-merge` | 设置 > 下载 > 二进制合并 | 布尔 | false |
| `--use-ffmpeg-concat-demuxer` | 设置 > 混流 > concat分离器 | 布尔 | false |
| `--del-after-done` | 设置 > 下载 > 完成后删除临时 | 布尔 | true |
| `--no-date-info` | 设置 > 混流 > 不写入日期 | 布尔 | false |
| `--no-log` | 设置 > 高级 > 关闭日志 | 布尔 | false |
| `--write-meta-json` | 设置 > 下载 > 生成元数据JSON | 布尔 | true |
| `--append-url-params` | 设置 > 高级 > 附加URL参数 | 布尔 | false |
| `-mt`, `--concurrent-download` | 设置 > 下载 > 并发下载 | 布尔 | false |
| `-H`, `--header` | 设置 > 网络 > 请求头 | 键值对 | - |
| `--sub-only` | 设置 > 下载 > 仅字幕 | 布尔 | false |
| `--sub-format` | 设置 > 下载 > 字幕格式 | 选择 | SRT |
| `--auto-subtitle-fix` | 设置 > 下载 > 自动修正字幕 | 布尔 | true |
| `--ffmpeg-binary-path` | 设置 > 混流 > FFmpeg路径 | 文本 | 自动查找 |
| `--log-level` | 设置 > 高级 > 日志级别 | 选择 | INFO |
| `--ui-language` | 设置 > 界面 > 语言 | 选择 | 系统 |
| `--urlprocessor-args` | (高级用户) | 文本 | - |

---

## 解密参数

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--key` | 设置 > 解密 > 密钥 | 文本 | - |
| `--key-text-file` | 设置 > 解密 > 密钥文件 | 文本 | - |
| `--decryption-engine` | 设置 > 解密 > 解密引擎 | 选择 | MP4DECRYPT |
| `--decryption-binary-path` | 设置 > 解密 > 程序路径 | 文本 | 自动查找 |
| `--mp4-real-time-decryption` | 设置 > 解密 > 实时解密 | 布尔 | false |
| `--custom-hls-method` | 设置 > 解密 > HLS加密方法 | 选择 | - |
| `--custom-hls-key` | 设置 > 解密 > HLS密钥 | 文本 | - |
| `--custom-hls-iv` | 设置 > 解密 > HLS IV | 文本 | - |

### 解密引擎选项

| 值 | 说明 |
| --- | --- |
| `FFMPEG` | 使用 FFmpeg 解密 |
| `MP4DECRYPT` | 使用 mp4decrypt (默认) |
| `SHAKA_PACKAGER` | 使用 Shaka Packager |

### HLS 加密方法选项

| 值 | 说明 |
| --- | --- |
| `AES_128` | AES-128 CBC |
| `AES_128_ECB` | AES-128 ECB |
| `CENC` | 通用加密 |
| `CHACHA20` | ChaCha20 |
| `SAMPLE_AES` | 采样 AES |
| `SAMPLE_AES_CTR` | 采样 AES CTR |
| `NONE` | 无加密 |
| `UNKNOWN` | 未知 |

---

## 限速与混流

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `-R`, `--max-speed` | 设置 > 下载 > 限速 | 文本 | 0 (不限) |
| `-M`, `--mux-after-done` | 设置 > 混流 > 自动混流 | 复合 | - |

### 混流参数格式

```
-M format=mp4:muxer=ffmpeg:bin_path="C:\path":skip_sub=false:keep=false
```

| 参数 | 说明 |
| --- | --- |
| `format` | 输出格式 (mp4/mkv) |
| `muxer` | 混流程序 (ffmpeg/mkvmerge) |
| `bin_path` | 程序路径 |
| `skip_sub` | 跳过字幕 |
| `keep` | 保留原文件 |

---

## 流选择

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `-sv`, `--select-video` | 流选择器 > 视频选择 | 正则 | - |
| `-sa`, `--select-audio` | 流选择器 > 音频选择 | 正则 | - |
| `-ss`, `--select-subtitle` | 流选择器 > 字幕选择 | 正则 | - |
| `-dv`, `--drop-video` | 流选择器 > 排除视频 | 正则 | - |
| `-da`, `--drop-audio` | 流选择器 > 排除音频 | 正则 | - |
| `-ds`, `--drop-subtitle` | 流选择器 > 排除字幕 | 正则 | - |
| `--ad-keyword` | 设置 > 高级 > 广告关键字 | 正则 | - |

### 流选择参数格式

```
-sv id=REGEX:lang=REGEX:name=REGEX:codecs=REGEX:res=REGEX:frame=REGEX:for=best
```

| 参数 | 说明 |
| --- | --- |
| `id` | 流ID匹配 |
| `lang` | 语言匹配 |
| `name` | 名称匹配 |
| `codecs` | 编码匹配 |
| `res` | 分辨率匹配 |
| `frame` | 帧率匹配 |
| `ch` | 声道匹配 |
| `range` | 色域匹配 |
| `url` | URL匹配 |
| `segsMin/Max` | 分片数量范围 |
| `plistDurMin/Max` | 播放列表时长范围 |
| `for` | 选择方式 (best/worst/all/bestN) |

### 流选择示例

```
# 选择最佳视频
-sv best

# 选择4K+HEVC视频
-sv res="3840*":codecs=hvc1:for=best

# 选择所有音频
-sa all

# 选择最佳英语音轨
-sa lang=en:for=best

# 选择所有中文字幕
-ss name="中文":for=all

# 排除低画质
-dv res="480.*"|res="360.*"
```

---

## 代理设置

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--use-system-proxy` | 设置 > 网络 > 使用系统代理 | 布尔 | true |
| `--custom-proxy` | 设置 > 网络 > 自定义代理 | 文本 | - |

### 代理格式

```
http://127.0.0.1:7890
socks5://127.0.0.1:1080
```

---

## 范围与定时

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--custom-range` | 任务详情 > 下载范围 | 文本 | - |
| `--task-start-at` | 任务详情 > 定时开始 | 日期时间 | - |

### 范围格式

```
# 分片序号
0-10      # 下载前 11 个分片
10-       # 从第 11 个分片开始
-99       # 下载前 100 个分片

# 时间范围
05:00-20:00  # 下载 5 分钟到 20 分钟的内容
```

---

## 直播设置

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--live-perform-as-vod` | 设置 > 直播 > 点播模式 | 布尔 | false |
| `--live-real-time-merge` | 设置 > 直播 > 实时合并 | 布尔 | false |
| `--live-keep-segments` | 设置 > 直播 > 保留分片 | 布尔 | true |
| `--live-pipe-mux` | 设置 > 直播 > 管道混流 | 布尔 | false |
| `--live-fix-vtt-by-audio` | 设置 > 直播 > 字幕修正 | 布尔 | false |
| `--live-record-limit` | 设置 > 直播 > 录制时长 | 文本 | - |
| `--live-wait-time` | 设置 > 直播 > 刷新间隔 | 数字 | 0 (自动) |
| `--live-take-count` | 设置 > 直播 > 首次获取数 | 数字 | 16 |

### 录制时长格式

```
--live-record-limit 01:30:00  # 录制 1 小时 30 分钟
```

---

## 混流导入

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--mux-import` | 设置 > 混流 > 外部媒体导入 | 复合 | - |

### 导入格式

```
--mux-import path="zh-Hans.srt":lang=chi:name="中文 (简体)"
--mux-import path="D:\media\atmos.m4a":lang=eng:name="English Audio"
```

| 参数 | 说明 |
| --- | --- |
| `path` | 文件路径 |
| `lang` | 语言代码 |
| `name` | 描述信息 |

---

## 其他

| CLI 参数 | UI 位置 | 类型 | 默认值 |
| --- | --- | --- | --- |
| `--disable-update-check` | 设置 > 通用 > 禁用更新检查 | 布尔 | false |
| `--allow-hls-multi-ext-map` | 设置 > 高级 > 实验性功能 | 布尔 | false |
| `--morehelp` | 帮助 > 参数帮助 | 命令 | - |
| `-h`, `--help` | 帮助 > 命令帮助 | 命令 | - |
| `--version` | 关于 > 版本信息 | 命令 | - |

---

## 命名模板变量

| 变量 | 说明 | 示例 |
| --- | --- | --- |
| `<SaveName>` | 用户指定的保存名称 | "my_video" |
| `<Id>` | 流的任务ID | "video_1" |
| `<Codecs>` | 编解码器信息 | "avc1.64001f" |
| `<Language>` | 语言代码 | "zh-CN" |
| `<Resolution>` | 视频分辨率 | "1920x1080" |
| `<Bandwidth>` | 带宽/比特率 | "5000000" |
| `<MediaType>` | 媒体类型 | "VIDEO" |
| `<Channels>` | 音频声道 | "2" |
| `<FrameRate>` | 帧率 | "30" |
| `<VideoRange>` | 色域/HDR | "HDR10" |
| `<GroupId>` | 流组标识符 | "group_720p" |
| `<Ext>` | 文件扩展名 | "mp4" |

---

## 统计信息

- **总 CLI 参数**: 约 60+
- **P0 核心参数**: 约 15 个
- **P1 重要参数**: 约 20 个
- **P2/P3 高级参数**: 约 25 个
