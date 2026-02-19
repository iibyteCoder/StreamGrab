/**
 * 设置相关类型
 */

import type {
  HeaderConfig,
  KeyConfig,
  SavePatternSettings,
  AdFilterSettings,
  MuxImport,
  CustomHlsDecryption,
} from "./common";

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

  // 流排除
  dropVideo: string;
  dropAudio: string;
  dropSubtitle: string;

  // 命名模板
  savePattern: SavePatternSettings;

  // 广告过滤
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

// 混流设置
export interface MuxSettings {
  format: "mp4" | "mkv";
  muxer: "ffmpeg" | "mkvmerge";
  binPath: string;
  keepOriginal: boolean;
  skipSubtitles: boolean;
  noDateInfo: boolean;
  useConcatDemuxer: boolean;
  muxImports: MuxImport[];
}

// 网络设置
export interface NetworkSettings {
  useSystemProxy: boolean;
  customProxy: string;
  headers: HeaderConfig[];
  baseUrl: string;
  appendUrlParams: boolean;
}

// 解密设置
export interface DecryptionSettings {
  keys: KeyConfig[];
  keyTextFile: string;
  engine: "FFMPEG" | "MP4DECRYPT" | "SHAKA_PACKAGER";
  binPath: string;
  realTimeDecryption: boolean;

  // 高级 HLS 解密
  customHls: CustomHlsDecryption;
}

// 直播设置
export interface LiveSettings {
  performAsVod: boolean;
  realTimeMerge: boolean;
  keepSegments: boolean;
  pipeMux: boolean;
  fixVttByAudio: boolean;
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
  allowHlsMultiExtMap: boolean;
  disableUpdateCheck: boolean;
  urlProcessorArgs: string;
}

// UI 设置
export interface UISettings {
  theme: "light" | "dark" | "system";
  showNotification: boolean;
  clipboardWatch: boolean;
}

// 配置模板
export interface ConfigTemplate {
  id: string;
  name: string;
  description: string;
  settings: Partial<AppSettings>;
  createdAt: Date;
  updatedAt: Date;
}

// 定时任务
export interface ScheduledTask {
  id: string;
  taskId: string;
  scheduledTime: Date;
  repeat: "none" | "daily" | "weekly";
  enabled: boolean;
}
