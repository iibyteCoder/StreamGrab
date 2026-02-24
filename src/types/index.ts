/**
 * 类型统一导出
 */

// 任务相关
export type {
  TaskStatus,
  DownloadTask,
  TaskProgressData,
  TaskLogEntry,
  TaskConfig,
  MediaInfo,
} from "./task";

// 设置相关（旧版，保持兼容）
export type {
  AppSettings as LegacyAppSettings,
  GeneralSettings,
  DownloadSettings,
  MuxSettings,
  NetworkSettings as LegacyNetworkSettings,
  DecryptionSettings as LegacyDecryptionSettings,
  LiveSettings,
  AdvancedSettings,
  UISettings,
  ConfigTemplate as LegacyConfigTemplate,
  ScheduledTask,
} from "./settings";

// 流相关
export type {
  StreamInfo,
  BaseStream,
  VideoStream,
  AudioStream,
  SubtitleStream,
  StreamSelection,
  UrlType,
} from "./stream";

// 流相关函数
export { detectUrlType, needsFfmpeg, isStreamingType } from "./stream";

// 通用类型
export type {
  HeaderConfig,
  KeyConfig,
  SavePatternSettings,
  AdFilterSettings,
  MuxImport,
  CustomHlsDecryption,
  HistoryRecord,
  LogSettings,
  LogEntry,
} from "./common";

// 常量
export {
  DEFAULT_SAVE_PATTERN_PRESETS,
  DEFAULT_AD_FILTER_PRESETS,
  HLS_ENCRYPTION_METHODS,
} from "./common";

// 新版领域类型
export * from "@/domain";
