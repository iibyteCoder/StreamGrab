//! 配置实体
//!
//! 定义配置相关的领域实体

use super::value_objects::*;
use serde::{Deserialize, Serialize};

// ========================================
// 应用配置
// ========================================

/// 应用配置实体
///
/// 软件本身的行为设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 语言
    pub language: Language,
    /// 是否自动开始下载
    pub auto_start_download: bool,
    /// 关闭窗口时最小化到托盘
    pub minimize_to_tray: bool,
    /// 是否检查更新
    pub check_update: bool,
    /// 默认保存目录
    pub default_save_dir: String,
    /// 默认临时目录
    pub default_tmp_dir: String,
    /// 主题
    pub theme: Theme,
    /// 是否显示通知
    pub show_notification: bool,
    /// 是否监视剪贴板
    pub clipboard_watch: bool,
    /// 日志级别
    pub log_level: LogLevel,
    /// 日志文件路径
    pub log_file_path: String,
    /// 禁用日志
    pub no_log: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            auto_start_download: true,
            minimize_to_tray: false,
            check_update: true,
            default_save_dir: String::new(),
            default_tmp_dir: String::new(),
            theme: Theme::Dark,
            show_notification: true,
            clipboard_watch: false,
            log_level: LogLevel::Info,
            log_file_path: String::new(),
            no_log: false,
        }
    }
}

// ========================================
// M3U8DL 配置
// ========================================

/// M3U8DL 配置实体
///
/// 流媒体下载专用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M3U8DLSettings {
    /// N_m3u8DL-RE 可执行文件路径
    pub n_m3u8dl_path: String,
    /// 下载线程数
    pub thread_count: i32,
    /// 重试次数
    pub retry_count: i32,
    /// HTTP 请求超时（秒）
    pub timeout: i32,
    /// 最大速度限制
    pub max_speed: String,
    /// 自动选择最佳流
    pub auto_select: bool,
    /// 视频流选择规则
    pub select_video: Option<String>,
    /// 音频流选择规则
    pub select_audio: Option<String>,
    /// 字幕流选择规则
    pub select_subtitle: Option<String>,
    /// 视频流排除规则
    pub drop_video: Option<String>,
    /// 音频流排除规则
    pub drop_audio: Option<String>,
    /// 字幕流排除规则
    pub drop_subtitle: Option<String>,
    /// 检查分片数量
    pub check_segments_count: bool,
    /// 完成后删除临时文件
    pub del_after_done: bool,
    /// 跳过合并
    pub skip_merge: bool,
    /// 写入元数据 JSON
    pub write_meta_json: bool,
    /// 二进制合并
    pub binary_merge: bool,
    /// 并发下载
    pub concurrent_download: bool,
    /// 混流格式
    pub mux_format: MuxFormat,
    /// 混流器
    pub muxer: Muxer,
    /// 混流器路径
    pub mux_bin_path: Option<String>,
    /// 混流时跳过字幕
    pub mux_skip_subtitles: bool,
    /// 混流后保留原文件
    mux_keep_original: bool,
    /// 仅下载字幕
    pub sub_only: bool,
    /// 字幕格式
    pub sub_format: SubtitleFormat,
    /// 自动修正字幕
    pub auto_subtitle_fix: bool,
    /// 直播：以点播方式下载
    pub live_perform_as_vod: bool,
    /// 直播：实时合并
    pub live_real_time_merge: bool,
    /// 直播：保留分片
    pub live_keep_segments: bool,
    /// 直播：管道混流
    pub live_pipe_mux: bool,
    /// 直播：通过音频修正 VTT
    pub live_fix_vtt_by_audio: bool,
    /// 直播：录制时长限制
    pub live_record_limit: Option<String>,
    /// 直播：等待时间
    pub live_wait_time: i32,
    /// 直播：首次获取分片数
    pub live_take_count: i32,
    /// 允许多个 EXT-X-MAP
    pub allow_hls_multi_ext_map: bool,
    /// URL 处理器参数
    pub url_processor_args: Option<String>,
    /// 不写入日期信息
    pub no_date_info: bool,
    /// 使用 ffmpeg concat demuxer
    pub use_ffmpeg_concat_demuxer: bool,
    /// 广告过滤关键字
    pub ad_filter_keywords: Vec<String>,
    /// 外部媒体导入
    pub mux_imports: Vec<MuxImport>,
}

impl Default for M3U8DLSettings {
    fn default() -> Self {
        Self {
            n_m3u8dl_path: String::new(),
            thread_count: 8,
            retry_count: 3,
            timeout: 100,
            max_speed: String::new(),
            auto_select: true,
            select_video: None,
            select_audio: None,
            select_subtitle: None,
            drop_video: None,
            drop_audio: None,
            drop_subtitle: None,
            check_segments_count: true,
            del_after_done: true,
            skip_merge: false,
            write_meta_json: false,
            binary_merge: false,
            concurrent_download: false,
            mux_format: MuxFormat::MP4,
            muxer: Muxer::FFmpeg,
            mux_bin_path: None,
            mux_skip_subtitles: false,
            mux_keep_original: false,
            sub_only: false,
            sub_format: SubtitleFormat::SRT,
            auto_subtitle_fix: true,
            live_perform_as_vod: false,
            live_real_time_merge: false,
            live_keep_segments: true,
            live_pipe_mux: false,
            live_fix_vtt_by_audio: false,
            live_record_limit: None,
            live_wait_time: 0,
            live_take_count: 16,
            allow_hls_multi_ext_map: false,
            url_processor_args: None,
            no_date_info: false,
            use_ffmpeg_concat_demuxer: false,
            ad_filter_keywords: Vec::new(),
            mux_imports: Vec::new(),
        }
    }
}

/// 外部媒体导入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuxImport {
    /// 文件路径
    pub path: String,
    /// 语言代码
    pub lang: Option<String>,
    /// 描述名称
    pub name: Option<String>,
}

// ========================================
// FFmpeg 配置
// ========================================

/// FFmpeg 配置实体
///
/// 直链下载专用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFmpegSettings {
    /// FFmpeg 可执行文件路径
    pub ffmpeg_path: String,
    /// FFprobe 可执行文件路径
    pub ffprobe_path: String,
    /// 重试次数
    pub retry_count: i32,
    /// 超时时间（秒）
    pub timeout: i32,
    /// 最大速度限制
    pub max_speed: String,
    /// 连接超时（秒）
    pub connection_timeout: i32,
    /// 重连尝试次数
    pub reconnect_attempts: i32,
    /// 重连延迟（秒）
    pub reconnect_delay: i32,
    /// 覆盖已存在文件
    pub overwrite_existing: bool,
    /// 保留时间戳
    pub preserve_timestamps: bool,
    /// User-Agent
    pub user_agent: Option<String>,
    /// Referer
    pub referer: Option<String>,
}

impl Default for FFmpegSettings {
    fn default() -> Self {
        Self {
            ffmpeg_path: String::new(),
            ffprobe_path: String::new(),
            retry_count: 3,
            timeout: 60,
            max_speed: String::new(),
            connection_timeout: 30,
            reconnect_attempts: 3,
            reconnect_delay: 5,
            overwrite_existing: false,
            preserve_timestamps: true,
            user_agent: None,
            referer: None,
        }
    }
}

// ========================================
// 网络配置
// ========================================

/// 网络配置实体
///
/// 共用网络设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// 使用系统代理
    pub use_system_proxy: bool,
    /// 自定义代理地址
    pub custom_proxy: Option<String>,
    /// Base URL
    pub base_url: Option<String>,
    /// 将 URL 参数添加到分片
    pub append_url_params: bool,
    /// 自定义请求头
    pub headers: Vec<HeaderConfig>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            use_system_proxy: true,
            custom_proxy: None,
            base_url: None,
            append_url_params: false,
            headers: Vec::new(),
        }
    }
}

/// 请求头配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    /// 请求头名称
    pub name: String,
    /// 请求头值
    pub value: String,
    /// 是否启用
    pub enabled: bool,
    /// 排序顺序
    pub sort_order: i32,
}

// ========================================
// 解密配置
// ========================================

/// 解密配置实体
///
/// 共用解密设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionSettings {
    /// 密钥文本文件路径
    pub key_text_file: Option<String>,
    /// 解密引擎
    pub decryption_engine: DecryptionEngine,
    /// 解密工具路径
    pub decryption_bin_path: Option<String>,
    /// 实时解密
    pub real_time_decryption: bool,
    /// 自定义 HLS 解密
    pub custom_hls: CustomHlsDecryption,
    /// 解密密钥列表
    pub keys: Vec<DecryptionKey>,
}

impl Default for DecryptionSettings {
    fn default() -> Self {
        Self {
            key_text_file: None,
            decryption_engine: DecryptionEngine::MP4Decrypt,
            decryption_bin_path: None,
            real_time_decryption: false,
            custom_hls: CustomHlsDecryption::default(),
            keys: Vec::new(),
        }
    }
}

/// 自定义 HLS 解密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHlsDecryption {
    /// 是否启用
    pub enabled: bool,
    /// 加密方法
    pub method: HlsEncryptionMethod,
    /// 密钥
    pub key: KeyValue,
    /// IV
    pub iv: KeyValue,
}

impl Default for CustomHlsDecryption {
    fn default() -> Self {
        Self {
            enabled: false,
            method: HlsEncryptionMethod::UNKNOWN,
            key: KeyValue::default(),
            iv: KeyValue::default(),
        }
    }
}

/// 密钥/IV 值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    /// 值类型
    pub value_type: KeyValueType,
    /// 值内容
    pub value: String,
}

impl Default for KeyValue {
    fn default() -> Self {
        Self {
            value_type: KeyValueType::Hex,
            value: String::new(),
        }
    }
}

/// 解密密钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionKey {
    /// KID
    pub kid: Option<String>,
    /// 密钥
    pub key: String,
    /// 排序顺序
    pub sort_order: i32,
}

// ========================================
// 配置模板
// ========================================

/// 配置模板实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    /// 模板 ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: Option<String>,
    /// 是否为预设模板
    pub is_preset: bool,
    /// 下载器类型
    pub downloader_type: DownloaderType,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 模板配置覆盖
    pub overrides: TemplateOverrides,
}

/// 模板配置覆盖
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateOverrides {
    /// M3U8DL 覆盖配置
    pub m3u8dl: Option<PartialM3U8DLSettings>,
    /// FFmpeg 覆盖配置
    pub ffmpeg: Option<PartialFFmpegSettings>,
    /// 网络覆盖配置
    pub network: Option<PartialNetworkSettings>,
    /// 解密覆盖配置
    pub decryption: Option<PartialDecryptionSettings>,
    /// 网络请求头（替换全局）
    pub headers: Option<Vec<HeaderConfig>>,
    /// 解密密钥（替换全局）
    pub keys: Option<Vec<DecryptionKey>>,
    /// 广告过滤关键字（替换全局）
    pub ad_filter_keywords: Option<Vec<String>>,
    /// 外部媒体导入（替换全局）
    pub mux_imports: Option<Vec<MuxImport>>,
}

/// 部分 M3U8DL 配置（用于模板覆盖）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialM3U8DLSettings {
    pub thread_count: Option<i32>,
    pub retry_count: Option<i32>,
    pub timeout: Option<i32>,
    pub max_speed: Option<String>,
    pub auto_select: Option<bool>,
    pub select_video: Option<String>,
    pub select_audio: Option<String>,
    pub select_subtitle: Option<String>,
    pub drop_video: Option<String>,
    pub drop_audio: Option<String>,
    pub drop_subtitle: Option<String>,
    pub check_segments_count: Option<bool>,
    pub del_after_done: Option<bool>,
    pub skip_merge: Option<bool>,
    pub write_meta_json: Option<bool>,
    pub binary_merge: Option<bool>,
    pub concurrent_download: Option<bool>,
    pub mux_format: Option<MuxFormat>,
    pub muxer: Option<Muxer>,
    pub mux_bin_path: Option<String>,
    pub mux_skip_subtitles: Option<bool>,
    pub mux_keep_original: Option<bool>,
    pub sub_only: Option<bool>,
    pub sub_format: Option<SubtitleFormat>,
    pub auto_subtitle_fix: Option<bool>,
    pub live_perform_as_vod: Option<bool>,
    pub live_real_time_merge: Option<bool>,
    pub live_keep_segments: Option<bool>,
    pub live_pipe_mux: Option<bool>,
    pub live_fix_vtt_by_audio: Option<bool>,
    pub live_record_limit: Option<String>,
    pub live_wait_time: Option<i32>,
    pub live_take_count: Option<i32>,
    pub allow_hls_multi_ext_map: Option<bool>,
    pub url_processor_args: Option<String>,
    pub no_date_info: Option<bool>,
    pub use_ffmpeg_concat_demuxer: Option<bool>,
}

/// 部分 FFmpeg 配置（用于模板覆盖）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialFFmpegSettings {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub retry_count: Option<i32>,
    pub timeout: Option<i32>,
    pub max_speed: Option<String>,
    pub connection_timeout: Option<i32>,
    pub reconnect_attempts: Option<i32>,
    pub reconnect_delay: Option<i32>,
    pub overwrite_existing: Option<bool>,
    pub preserve_timestamps: Option<bool>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

/// 部分网络配置（用于模板覆盖）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialNetworkSettings {
    pub use_system_proxy: Option<bool>,
    pub custom_proxy: Option<String>,
    pub base_url: Option<String>,
    pub append_url_params: Option<bool>,
}

/// 部分解密配置（用于模板覆盖）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialDecryptionSettings {
    pub key_text_file: Option<String>,
    pub decryption_engine: Option<DecryptionEngine>,
    pub decryption_bin_path: Option<String>,
    pub real_time_decryption: Option<bool>,
    pub custom_hls: Option<CustomHlsDecryption>,
}

// ========================================
// 任务配置
// ========================================

/// 任务配置实体
///
/// 每个任务的独立配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// 任务 ID
    pub task_id: String,
    /// 使用的模板 ID
    pub template_id: Option<String>,
    /// 下载器类型
    pub downloader_type: DownloaderType,
    /// 任务级配置覆盖
    pub overrides: TaskConfigOverrides,
}

/// 任务配置覆盖
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskConfigOverrides {
    /// 保存目录
    pub save_dir: Option<String>,
    /// 保存文件名
    pub save_name: Option<String>,
    /// 保存命名模板
    pub save_pattern: Option<String>,
    /// 网络请求头（追加到全局/模板）
    pub headers: Vec<HeaderConfig>,
    /// M3U8DL 特定覆盖
    pub m3u8dl: Option<PartialM3U8DLSettings>,
    /// FFmpeg 特定覆盖
    pub ffmpeg: Option<PartialFFmpegSettings>,
}

// ========================================
// 已解析配置
// ========================================

/// 已解析的完整配置
///
/// 合并后的配置，用于命令行构建
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedConfig {
    /// 下载器类型
    pub downloader_type: DownloaderType,
    /// 使用的模板 ID
    pub template_id: Option<String>,
    /// 应用配置
    pub app: AppSettings,
    /// M3U8DL 配置
    pub m3u8dl: M3U8DLSettings,
    /// FFmpeg 配置
    pub ffmpeg: FFmpegSettings,
    /// 网络配置
    pub network: NetworkSettings,
    /// 解密配置
    pub decryption: DecryptionSettings,
    /// 任务特定值
    pub task: TaskSpecificConfig,
}

/// 任务特定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpecificConfig {
    /// 保存目录
    pub save_dir: String,
    /// 保存文件名
    pub save_name: String,
    /// 保存命名模板
    pub save_pattern: Option<String>,
    /// 自定义范围
    pub custom_range: Option<String>,
    /// 定时开始
    pub start_at: Option<String>,
}
