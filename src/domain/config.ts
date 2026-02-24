/**
 * 配置领域类型
 *
 * 与后端配置结构保持一致的类型定义
 */

// ========================================
// 值对象（枚举类型）
// ========================================

/** 下载器类型 */
export type DownloaderType = "m3u8dl" | "ffmpeg";

/** 解密引擎类型 */
export type DecryptionEngine = "FFMPEG" | "MP4DECRYPT" | "SHAKA_PACKAGER";

/** 混流格式 */
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
// 应用配置
// ========================================

/** 应用配置 */
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

/** 默认应用配置 */
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
// M3U8DL 配置
// ========================================

/** M3U8DL 配置 */
export interface M3U8DLSettings {
  n_m3u8dl_path: string;
  thread_count: number;
  retry_count: number;
  timeout: number;
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
  mux_format: MuxFormat;
  muxer: Muxer;
  mux_bin_path: string | null;
  mux_skip_subtitles: boolean;
  mux_keep_original: boolean;
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
}

/** 默认 M3U8DL 配置 */
export const DEFAULT_M3U8DL_SETTINGS: M3U8DLSettings = {
  n_m3u8dl_path: "",
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
  mux_format: "mp4",
  muxer: "ffmpeg",
  mux_bin_path: null,
  mux_skip_subtitles: false,
  mux_keep_original: false,
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
};

// ========================================
// FFmpeg 配置
// ========================================

/** FFmpeg 配置 */
export interface FFmpegSettings {
  ffmpeg_path: string;
  ffprobe_path: string;
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

/** 默认 FFmpeg 配置 */
export const DEFAULT_FFMPEG_SETTINGS: FFmpegSettings = {
  ffmpeg_path: "",
  ffprobe_path: "",
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

// ========================================
// 网络配置
// ========================================

/** 网络配置 */
export interface NetworkSettings {
  use_system_proxy: boolean;
  custom_proxy: string | null;
  base_url: string | null;
  append_url_params: boolean;
}

/** 默认网络配置 */
export const DEFAULT_NETWORK_SETTINGS: NetworkSettings = {
  use_system_proxy: true,
  custom_proxy: null,
  base_url: null,
  append_url_params: false,
};

/** 网络请求头 */
export interface NetworkHeader {
  id: number;
  name: string;
  value: string;
  enabled: boolean;
  sort_order: number;
}

// ========================================
// 解密配置
// ========================================

/** 解密配置 */
export interface DecryptionSettings {
  key_text_file: string | null;
  decryption_engine: DecryptionEngine;
  decryption_bin_path: string | null;
  real_time_decryption: boolean;
  custom_hls_enabled: boolean;
  custom_hls_method: HlsEncryptionMethod;
  custom_hls_key_type: KeyValueType;
  custom_hls_key_value: string | null;
  custom_hls_iv_type: KeyValueType;
  custom_hls_iv_value: string | null;
}

/** 默认解密配置 */
export const DEFAULT_DECRYPTION_SETTINGS: DecryptionSettings = {
  key_text_file: null,
  decryption_engine: "MP4DECRYPT",
  decryption_bin_path: null,
  real_time_decryption: false,
  custom_hls_enabled: false,
  custom_hls_method: "UNKNOWN",
  custom_hls_key_type: "hex",
  custom_hls_key_value: null,
  custom_hls_iv_type: "hex",
  custom_hls_iv_value: null,
};

/** 解密密钥 */
export interface DecryptionKey {
  id: number;
  kid: string | null;
  key: string;
  sort_order: number;
}

// ========================================
// 配置模板
// ========================================

/** 配置模板 */
export interface ConfigTemplate {
  id: string;
  name: string;
  description: string | null;
  is_preset: boolean;
  downloader_type: DownloaderType;
  created_at: string;
  updated_at: string;
}

/** 模板覆盖配置 */
export interface TemplateOverrides {
  m3u8dl?: PartialM3U8DLSettings;
  ffmpeg?: PartialFFmpegSettings;
  network?: PartialNetworkSettings;
  decryption?: PartialDecryptionSettings;
  headers?: NetworkHeader[];
  keys?: DecryptionKey[];
  ad_filter_keywords?: string[];
  mux_imports?: MuxImport[];
}

/** 部分 M3U8DL 配置（用于模板覆盖） */
export interface PartialM3U8DLSettings {
  thread_count?: number;
  retry_count?: number;
  timeout?: number;
  max_speed?: string;
  auto_select?: boolean;
  select_video?: string;
  select_audio?: string;
  select_subtitle?: string;
  drop_video?: string;
  drop_audio?: string;
  drop_subtitle?: string;
  check_segments_count?: boolean;
  del_after_done?: boolean;
  skip_merge?: boolean;
  write_meta_json?: boolean;
  binary_merge?: boolean;
  concurrent_download?: boolean;
  mux_format?: MuxFormat;
  muxer?: Muxer;
  mux_bin_path?: string;
  mux_skip_subtitles?: boolean;
  mux_keep_original?: boolean;
  sub_only?: boolean;
  sub_format?: SubtitleFormat;
  auto_subtitle_fix?: boolean;
  live_perform_as_vod?: boolean;
  live_real_time_merge?: boolean;
  live_keep_segments?: boolean;
  live_pipe_mux?: boolean;
  live_fix_vtt_by_audio?: boolean;
  live_record_limit?: string;
  live_wait_time?: number;
  live_take_count?: number;
  allow_hls_multi_ext_map?: boolean;
  url_processor_args?: string;
  no_date_info?: boolean;
  use_ffmpeg_concat_demuxer?: boolean;
}

/** 部分 FFmpeg 配置（用于模板覆盖） */
export interface PartialFFmpegSettings {
  ffmpeg_path?: string;
  ffprobe_path?: string;
  retry_count?: number;
  timeout?: number;
  max_speed?: string;
  connection_timeout?: number;
  reconnect_attempts?: number;
  reconnect_delay?: number;
  overwrite_existing?: boolean;
  preserve_timestamps?: boolean;
  user_agent?: string;
  referer?: string;
}

/** 部分网络配置（用于模板覆盖） */
export interface PartialNetworkSettings {
  use_system_proxy?: boolean;
  custom_proxy?: string;
  base_url?: string;
  append_url_params?: boolean;
}

/** 部分解密配置（用于模板覆盖） */
export interface PartialDecryptionSettings {
  key_text_file?: string;
  decryption_engine?: DecryptionEngine;
  decryption_bin_path?: string;
  real_time_decryption?: boolean;
  custom_hls?: CustomHlsDecryption;
}

/** 自定义 HLS 解密配置 */
export interface CustomHlsDecryption {
  enabled: boolean;
  method: HlsEncryptionMethod;
  key: KeyValue;
  iv: KeyValue;
}

/** 密钥/IV 值 */
export interface KeyValue {
  value_type: KeyValueType;
  value: string;
}

/** 外部媒体导入 */
export interface MuxImport {
  path: string;
  lang?: string;
  name?: string;
}

// ========================================
// 已解析配置
// ========================================

/** 已解析的完整配置 */
export interface ResolvedConfig {
  downloader_type: DownloaderType;
  template_id: string | null;
  app: AppSettings;
  m3u8dl: M3U8DLSettings;
  ffmpeg: FFmpegSettings;
  network: NetworkSettings;
  decryption: DecryptionSettings;
  task: TaskSpecificConfig;
}

/** 任务特定配置 */
export interface TaskSpecificConfig {
  save_dir: string;
  save_name: string;
  save_pattern: string | null;
  custom_range: string | null;
  start_at: string | null;
}

// ========================================
// 完整配置（用于加载所有配置）
// ========================================

/** 完整配置 */
export interface AllConfig {
  app: AppSettings;
  m3u8dl: M3U8DLSettings;
  ffmpeg: FFmpegSettings;
  network: NetworkSettings;
  decryption: DecryptionSettings;
  headers: NetworkHeader[];
  keys: DecryptionKey[];
}
