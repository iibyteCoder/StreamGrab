// ============================================
// 任务相关类型
// ============================================

// 任务状态
export type TaskStatus =
  | "pending" // 等待中
  | "analyzing" // 解析中
  | "downloading" // 下载中
  | "paused" // 已暂停
  | "merging" // 合并中
  | "muxing" // 混流中
  | "completed" // 已完成
  | "failed" // 失败
  | "cancelled"; // 已取消

// 下载任务
export interface DownloadTask {
  id: string;
  url: string;
  fileName: string;
  saveDir: string;
  status: TaskStatus;
  progress: TaskProgressData;
  error?: string;
  config?: Partial<TaskConfig>;
  outputPath?: string;
  createdAt: Date;
  updatedAt: Date;
  startedAt?: Date;
  completedAt?: Date;
}

// 任务进度数据
export interface TaskProgressData {
  percent: number;
  speed: number; // bytes/s
  downloadedSize: number;
  totalSize: number;
  downloadedSegments: number;
  totalSegments: number;
  eta: number; // seconds
  currentAction: string;
}

// 任务日志条目
export interface TaskLogEntry {
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

// 任务配置
export interface TaskConfig {
  saveDir: string;
  saveName: string;
  threadCount: number;
  retryCount: number;
  timeout: number;
  maxSpeed: string;

  // 流选择
  autoSelect: boolean;
  selectVideo?: string;
  selectAudio?: string;
  selectSubtitle?: string;

  // 流排除 (新增)
  dropVideo?: string;
  dropAudio?: string;
  dropSubtitle?: string;

  // 命名模板 (新增)
  savePattern?: SavePatternSettings;

  // 混流
  muxFormat?: "mp4" | "mkv";
  muxAfterDone: boolean;

  // 其他选项
  skipMerge: boolean;
  delAfterDone: boolean;
  checkSegmentsCount: boolean;

  // 范围下载 (新增)
  customRange?: string;

  // 定时开始 (新增)
  startAt?: Date | null;

  // 高级
  headers?: HeaderConfig[];
  proxy?: string;
  key?: string;
}

// ============================================
// 设置相关类型
// ============================================

// 应用设置
export interface AppSettings {
  general: GeneralSettings;
  download: DownloadSettings;
  mux: MuxSettings;
  network: NetworkSettings;
  live: LiveSettings;
  decryption: DecryptionSettings;
  advanced: AdvancedSettings;
  ui: UISettings;
}

// 通用设置
export interface GeneralSettings {
  saveDir: string;
  tmpDir: string;
  language: "zh-CN" | "en-US" | "zh-TW";
  autoStartDownload: boolean;
  minimizeToTray: boolean;
  checkUpdate: boolean;
}

// 下载设置
export interface DownloadSettings {
  threadCount: number;
  retryCount: number;
  timeout: number;
  maxSpeed: string;

  // 流选择
  autoSelect: boolean;
  selectVideo: string;
  selectAudio: string;
  selectSubtitle: string;

  // 流排除 (新增)
  dropVideo: string;
  dropAudio: string;
  dropSubtitle: string;

  // 命名模板 (新增)
  savePattern: SavePatternSettings;

  // 广告过滤 (新增)
  adFilter: AdFilterSettings;

  // 选项
  checkSegmentsCount: boolean;
  delAfterDone: boolean;
  skipMerge: boolean;
  writeMetaJson: boolean;
  binaryMerge: boolean;
  concurrentDownload: boolean;
  subOnly: boolean;
  subFormat: "SRT" | "VTT";
  autoSubtitleFix: boolean;
}

// 命名模板设置 (新增)
export interface SavePatternSettings {
  enabled: boolean;
  template: string;
  presetId: string;
}

// 广告过滤设置 (新增)
export interface AdFilterSettings {
  enabled: boolean;
  keywords: string[];
}

// 混流设置
export interface MuxSettings {
  format: "mp4" | "mkv";
  muxer: "ffmpeg" | "mkvmerge";
  binPath: string;
  keepOriginal: boolean;
  skipSubtitles: boolean;
  noDateInfo: boolean; // 新增
  useConcatDemuxer: boolean; // 新增
  muxImports: MuxImport[]; // 新增
}

// 外部媒体导入 (新增)
export interface MuxImport {
  path: string;
  lang?: string;
  name?: string;
}

// 网络设置
export interface NetworkSettings {
  useSystemProxy: boolean;
  customProxy: string;
  headers: HeaderConfig[];
  baseUrl: string; // 新增
  appendUrlParams: boolean; // 新增
}

// 解密设置
export interface DecryptionSettings {
  keys: KeyConfig[];
  keyTextFile: string;
  engine: "FFMPEG" | "MP4DECRYPT" | "SHAKA_PACKAGER";
  binPath: string;
  realTimeDecryption: boolean;

  // 高级 HLS 解密 (新增)
  customHls: CustomHlsDecryption;
}

// 高级 HLS 解密 (新增)
export interface CustomHlsDecryption {
  enabled: boolean;
  method:
    | "AES_128"
    | "AES_128_ECB"
    | "CENC"
    | "CHACHA20"
    | "NONE"
    | "SAMPLE_AES"
    | "SAMPLE_AES_CTR"
    | "UNKNOWN";
  key: {
    type: "file" | "hex" | "base64";
    value: string;
  };
  iv: {
    type: "file" | "hex" | "base64";
    value: string;
  };
}

// 密钥配置
export interface KeyConfig {
  kid?: string;
  key: string;
}

// 直播设置
export interface LiveSettings {
  performAsVod: boolean;
  realTimeMerge: boolean;
  keepSegments: boolean;
  pipeMux: boolean;
  fixVttByAudio: boolean; // 新增
  recordLimit: string;
  waitTime: number;
  takeCount: number;
}

// 高级设置
export interface AdvancedSettings {
  ffmpegPath: string;
  n_m3u8dlPath: string;
  logLevel: "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF";
  logFilePath: string;
  noLog: boolean;
  allowHlsMultiExtMap: boolean; // 新增 - 实验性功能
  disableUpdateCheck: boolean; // 新增
  urlProcessorArgs: string; // 新增
}

// UI 设置
export interface UISettings {
  theme: "light" | "dark" | "system";
  showNotification: boolean;
  clipboardWatch: boolean;
}

// 请求头配置
export interface HeaderConfig {
  key: string;
  value: string;
  enabled: boolean;
}

// ============================================
// 流信息类型
// ============================================

// 流信息
export interface StreamInfo {
  videos: VideoStream[];
  audios: AudioStream[];
  subtitles: SubtitleStream[];
  duration: number;
  segmentCount: number;
  isLive: boolean;
  isEncrypted: boolean;
}

// 基础流
export interface BaseStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string;
  selected?: boolean;
}

// 视频流
export interface VideoStream extends BaseStream {
  resolution: string;
  width: number;
  height: number;
  frameRate: number;
  videoRange: "SDR" | "HDR10" | "HDR10+" | "DV" | "HLG";
}

// 音频流
export interface AudioStream extends BaseStream {
  channels: string;
  sampleRate: number;
  isDefault: boolean;
}

// 字幕流
export interface SubtitleStream extends BaseStream {
  format: "srt" | "vtt" | "ttml";
  isDefault: boolean;
  isForced: boolean;
}

// ============================================
// 历史记录类型
// ============================================

export interface HistoryRecord {
  id: string;
  url: string;
  file_name: string;
  save_path: string;
  file_size: number;
  duration: number;
  completed_at: string;
}

// ============================================
// 配置模板类型
// ============================================

export interface ConfigTemplate {
  id: string;
  name: string;
  description: string;
  settings: Partial<AppSettings>;
  createdAt: Date;
  updatedAt: Date;
}

// ============================================
// 定时任务类型
// ============================================

export interface ScheduledTask {
  id: string;
  taskId: string;
  scheduledTime: Date;
  repeat: "none" | "daily" | "weekly";
  enabled: boolean;
}

// ============================================
// 流选择类型
// ============================================

export interface StreamSelection {
  videoIds: string[];
  audioIds: string[];
  subtitleIds: string[];
}

// ============================================
// 日志类型 (新增)
// ============================================

export interface LogSettings {
  level: "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF";
  enableFileOutput: boolean;
  logFilePath: string;
  maxFileSize: number; // MB
  maxFileCount: number;
}

export interface LogEntry {
  timestamp: Date;
  level: "DEBUG" | "INFO" | "WARN" | "ERROR";
  message: string;
  source?: string;
}

// ============================================
// 默认值常量
// ============================================

export const DEFAULT_SAVE_PATTERN_PRESETS = [
  { id: "basic", name: "基础", template: "<SaveName>" },
  { id: "resolution", name: "包含分辨率", template: "<SaveName>_<Resolution>" },
  { id: "bandwidth", name: "包含带宽", template: "<SaveName>_<Resolution>_<Bandwidth>kbps" },
  { id: "multi-audio", name: "多音轨", template: "<SaveName>_<Language>_<Channels>ch" },
  { id: "full", name: "完整信息", template: "<MediaType>_<Resolution>_<Codecs>_<Language>" },
];

export const DEFAULT_AD_FILTER_PRESETS = [
  "ad\\.domain\\.com",
  "\\/ad\\/|\\/ads\\/|\\/advert",
  "doubleclick\\.net",
];

export const HLS_ENCRYPTION_METHODS = [
  { value: "AES_128", label: "AES-128 CBC" },
  { value: "AES_128_ECB", label: "AES-128 ECB" },
  { value: "CENC", label: "通用加密 (CENC)" },
  { value: "CHACHA20", label: "ChaCha20" },
  { value: "SAMPLE_AES", label: "采样 AES" },
  { value: "SAMPLE_AES_CTR", label: "采样 AES CTR" },
  { value: "NONE", label: "无加密" },
  { value: "UNKNOWN", label: "未知" },
] as const;
