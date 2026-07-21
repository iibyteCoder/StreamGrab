/**
 * 配置领域类型（前端唯一权威来源）
 *
 * 与后端 `src-tauri/src/domain/config.rs` 一一对应：
 * - 配置组（AppSettings / Nm3u8dlConfig / FfmpegConfig）为 snake_case JSON
 * - 任务相关类型（TaskOverrides / TaskPreset）为 camelCase JSON
 *
 * 三层配置模型：全局默认（本文件的 DEFAULT_*）→ 任务级覆盖（TaskOverrides）→ 引擎合并构建命令行
 */

// ========================================
// 值对象（枚举类型）
// ========================================

/** 工具标识（对应后端 tool_settings 行键） */
export type ToolId = "nm3u8dl" | "ffmpeg";

/** 解密引擎类型 */
export type DecryptionEngine = "FFMPEG" | "MP4DECRYPT" | "SHAKA_PACKAGER";

/** 混流容器格式 */
export type MuxFormat = "mp4" | "mkv";

/** 混流器类型 */
export type Muxer = "ffmpeg" | "mkvmerge";

/** HLS 加密方法 */
export type HlsEncryptionMethod =
  | "AES_128"
  | "AES_128_ECB"
  | "CENC"
  | "CHACHA20"
  | "NONE"
  | "SAMPLE_AES"
  | "SAMPLE_AES_CTR"
  | "UNKNOWN";

/** 密钥/IV 值类型 */
export type KeyValueType = "file" | "hex" | "base64";

/** 字幕格式 */
export type SubtitleFormat = "SRT" | "VTT";

/** 主题类型 */
export type Theme = "light" | "dark" | "system";

/** 语言代码 */
export type Language = "zh-CN" | "zh-TW" | "en-US";

/** 日志级别 */
export type LogLevel = "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF";

// ========================================
// 应用配置（app_settings 表）
// ========================================

/** 应用级配置（通用·界面） */
export interface AppSettings {
  language: Language;
  auto_start_download: boolean;
  minimize_to_tray: boolean;
  check_update: boolean;
  default_save_dir: string;
  default_tmp_dir: string;
  theme: Theme;
  show_notification: boolean;
  clipboard_watch: boolean;
  log_level: LogLevel;
  log_file_path: string;
  no_log: boolean;
}

export const DEFAULT_APP_SETTINGS: AppSettings = {
  language: "zh-CN",
  auto_start_download: true,
  minimize_to_tray: false,
  check_update: true,
  default_save_dir: "",
  default_tmp_dir: "",
  theme: "dark",
  show_notification: true,
  clipboard_watch: false,
  log_level: "INFO",
  log_file_path: "",
  no_log: false,
};

// ========================================
// N_m3u8DL-RE 工具配置（tool_settings["nm3u8dl"]）
// ========================================

/** 自定义 HTTP 请求头 */
export interface NetworkHeader {
  id: number;
  name: string;
  value: string;
  enabled: boolean;
  sort_order: number;
}

/** 网络配置（N_m3u8DL-RE 子配置） */
export interface NetworkConfig {
  use_system_proxy: boolean;
  custom_proxy: string | null;
  base_url: string | null;
  append_url_params: boolean;
  headers: NetworkHeader[];
}

export const DEFAULT_NETWORK_CONFIG: NetworkConfig = {
  use_system_proxy: true,
  custom_proxy: null,
  base_url: null,
  append_url_params: false,
  headers: [],
};

/** 解密密钥 */
export interface DecryptionKey {
  id: number;
  /** KID（可空，空则为纯 KEY） */
  kid: string | null;
  key: string;
  sort_order: number;
}

/** 自定义 HLS 解密配置 */
export interface CustomHlsConfig {
  enabled: boolean;
  method: HlsEncryptionMethod;
  key_type: KeyValueType;
  key_value: string | null;
  iv_type: KeyValueType;
  iv_value: string | null;
}

/** 解密配置（N_m3u8DL-RE 子配置） */
export interface DecryptionConfig {
  key_text_file: string | null;
  engine: DecryptionEngine;
  bin_path: string | null;
  real_time_decryption: boolean;
  custom_hls: CustomHlsConfig;
  keys: DecryptionKey[];
}

export const DEFAULT_DECRYPTION_CONFIG: DecryptionConfig = {
  key_text_file: null,
  engine: "MP4DECRYPT",
  bin_path: null,
  real_time_decryption: false,
  custom_hls: {
    enabled: false,
    method: "UNKNOWN",
    key_type: "hex",
    key_value: null,
    iv_type: "hex",
    iv_value: null,
  },
  keys: [],
};

/**
 * N_m3u8DL-RE 工具配置
 *
 * 流媒体下载引擎（HLS/DASH/MSS）的全部默认行为，含网络与解密子配置
 */
export interface Nm3u8dlConfig {
  /** 工具二进制路径（空 = 自动检测） */
  path: string;
  thread_count: number;
  retry_count: number;
  /** HTTP 请求超时（秒） */
  timeout: number;
  /** 限速（如 "10M"，空 = 不限速） */
  max_speed: string;
  auto_select: boolean;
  select_video: string | null;
  select_audio: string | null;
  select_subtitle: string | null;
  drop_video: string | null;
  drop_audio: string | null;
  drop_subtitle: string | null;
  check_segments_count: boolean;
  del_after_done: boolean;
  skip_merge: boolean;
  write_meta_json: boolean;
  binary_merge: boolean;
  concurrent_download: boolean;
  sub_only: boolean;
  sub_format: SubtitleFormat;
  auto_subtitle_fix: boolean;
  live_perform_as_vod: boolean;
  live_real_time_merge: boolean;
  live_keep_segments: boolean;
  live_pipe_mux: boolean;
  live_fix_vtt_by_audio: boolean;
  live_record_limit: string | null;
  live_wait_time: number;
  live_take_count: number;
  allow_hls_multi_ext_map: boolean;
  url_processor_args: string | null;
  no_date_info: boolean;
  use_ffmpeg_concat_demuxer: boolean;
  network: NetworkConfig;
  decryption: DecryptionConfig;
}

export const DEFAULT_NM3U8DL_CONFIG: Nm3u8dlConfig = {
  path: "",
  thread_count: 8,
  retry_count: 3,
  timeout: 100,
  max_speed: "",
  auto_select: true,
  select_video: null,
  select_audio: null,
  select_subtitle: null,
  drop_video: null,
  drop_audio: null,
  drop_subtitle: null,
  check_segments_count: true,
  del_after_done: true,
  skip_merge: false,
  write_meta_json: false,
  binary_merge: false,
  concurrent_download: false,
  sub_only: false,
  sub_format: "SRT",
  auto_subtitle_fix: true,
  live_perform_as_vod: false,
  live_real_time_merge: false,
  live_keep_segments: true,
  live_pipe_mux: false,
  live_fix_vtt_by_audio: false,
  live_record_limit: null,
  live_wait_time: 0,
  live_take_count: 16,
  allow_hls_multi_ext_map: false,
  url_processor_args: null,
  no_date_info: false,
  use_ffmpeg_concat_demuxer: false,
  network: DEFAULT_NETWORK_CONFIG,
  decryption: DEFAULT_DECRYPTION_CONFIG,
};

// ========================================
// FFmpeg 工具配置（tool_settings["ffmpeg"]）
// ========================================

/**
 * FFmpeg 工具配置
 *
 * 覆盖三个职责：混流默认值（被 N_m3u8DL-RE 的 -M 参数消费）、
 * 直链视频下载默认值、ffprobe 媒体分析的二进制管理
 */
export interface FfmpegConfig {
  ffmpeg_path: string;
  ffprobe_path: string;
  // —— 混流默认值 ——
  mux_format: MuxFormat;
  muxer: Muxer;
  mux_bin_path: string | null;
  mux_skip_subtitles: boolean;
  mux_keep_original: boolean;
  // —— 直链下载默认值 ——
  retry_count: number;
  timeout: number;
  max_speed: string;
  connection_timeout: number;
  reconnect_attempts: number;
  reconnect_delay: number;
  overwrite_existing: boolean;
  preserve_timestamps: boolean;
  user_agent: string | null;
  referer: string | null;
}

export const DEFAULT_FFMPEG_CONFIG: FfmpegConfig = {
  ffmpeg_path: "",
  ffprobe_path: "",
  mux_format: "mp4",
  muxer: "ffmpeg",
  mux_bin_path: null,
  mux_skip_subtitles: false,
  mux_keep_original: false,
  retry_count: 3,
  timeout: 60,
  max_speed: "",
  connection_timeout: 30,
  reconnect_attempts: 3,
  reconnect_delay: 5,
  overwrite_existing: false,
  preserve_timestamps: true,
  user_agent: null,
  referer: null,
};

/** 全部工具配置 */
export interface ToolConfigs {
  nm3u8dl: Nm3u8dlConfig;
  ffmpeg: FfmpegConfig;
}

// ========================================
// 任务级覆盖（「默认值 + 覆盖」模型的第二层）
// ========================================

/** 流选择（手动选择的具体流，覆盖全局 select_* 默认） */
export interface StreamSelection {
  video?: string | null;
  audio?: string | null;
  subtitle?: string | null;
}

/**
 * 任务级覆盖配置
 *
 * 全部字段可选：undefined/null = 沿用全局默认。
 * 添加任务对话框收集 → 随任务持久化 → 下载时后端引擎合并。
 */
export interface TaskOverrides {
  saveDir?: string | null;
  saveName?: string | null;
  muxFormat?: MuxFormat | null;
  maxSpeed?: string | null;
  customRange?: string | null;
  subtitleFormat?: SubtitleFormat | null;
  subtitlesOnly?: boolean | null;
  /** 定时开始（ISO 8601 本地时间字符串，前端调度器消费） */
  scheduledStartAt?: string | null;
  selection?: StreamSelection | null;
  /** 来源预设 ID（溯源用） */
  presetId?: string | null;
  /** 任务级解密密钥（全局密钥库为空时生效） */
  key?: string | null;
}

/** 任务预设：命名的 TaskOverrides 组合 */
export interface TaskPreset {
  id: string;
  name: string;
  /** Lucide 图标名 */
  icon?: string | null;
  description?: string | null;
  overrides: TaskOverrides;
  createdAt: string;
  updatedAt: string;
}
