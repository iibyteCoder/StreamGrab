//! 配置领域类型
//!
//! 后端配置形状的唯一权威来源，与前端 `src/domain/config.ts` 一一对应。
//!
//! ## 存储模型
//!
//! - [`AppSettings`] → `app_settings` 表（单行 JSON）
//! - [`Nm3u8dlConfig`] → `tool_settings` 表 `tool_id = "nm3u8dl"` 行
//! - [`FfmpegConfig`] → `tool_settings` 表 `tool_id = "ffmpeg"` 行
//!
//! 所有类型均派生 `serde(default)`：JSON 中缺失的字段自动回落到默认值，
//! 这是「默认值 + 覆盖」三层配置模型的基础。

use serde::{Deserialize, Serialize};

// ========================================
// 值对象（枚举）
// ========================================

/// 主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    Light,
    #[default]
    Dark,
    System,
}

/// 界面语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
    #[serde(rename = "en-US")]
    EnUs,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Off,
}

/// 混流容器格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MuxFormat {
    #[default]
    Mp4,
    Mkv,
}

/// 混流器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Muxer {
    #[default]
    Ffmpeg,
    Mkvmerge,
}

/// 解密引擎
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DecryptionEngine {
    #[serde(rename = "FFMPEG")]
    Ffmpeg,
    #[default]
    #[serde(rename = "MP4DECRYPT")]
    Mp4Decrypt,
    #[serde(rename = "SHAKA_PACKAGER")]
    ShakaPackager,
}

/// HLS 加密方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HlsEncryptionMethod {
    #[serde(rename = "AES_128")]
    Aes128,
    #[serde(rename = "AES_128_ECB")]
    Aes128Ecb,
    #[serde(rename = "CENC")]
    Cenc,
    #[serde(rename = "CHACHA20")]
    Chacha20,
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "SAMPLE_AES")]
    SampleAes,
    #[serde(rename = "SAMPLE_AES_CTR")]
    SampleAesCtr,
    #[default]
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// 密钥/IV 值的表示类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyValueType {
    File,
    #[default]
    Hex,
    Base64,
}

/// 字幕格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubtitleFormat {
    #[default]
    Srt,
    Vtt,
}

// ========================================
// 应用配置（app_settings 表）
// ========================================

/// 应用级配置（通用·界面）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// 界面语言
    pub language: Language,
    /// 添加任务后自动开始下载
    pub auto_start_download: bool,
    /// 关闭窗口时最小化到托盘
    pub minimize_to_tray: bool,
    /// 启动时检查更新
    pub check_update: bool,
    /// 默认保存目录（空 = 系统下载目录）
    pub default_save_dir: String,
    /// 默认临时目录（空 = 跟随保存目录）
    pub default_tmp_dir: String,
    /// 主题
    pub theme: Theme,
    /// 下载完成时发送系统通知
    pub show_notification: bool,
    /// 监控剪贴板自动识别链接
    pub clipboard_watch: bool,
    /// 日志级别
    pub log_level: LogLevel,
    /// 日志文件路径（空 = 默认位置）
    pub log_file_path: String,
    /// 禁用日志
    pub no_log: bool,
    /// 最大并发下载任务数
    pub max_concurrent_tasks: u32,
}

/// 关闭窗口时的行为
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseBehavior {
    /// 隐藏到系统托盘（应用继续运行）
    Minimize,
    /// 正常退出
    Exit,
}

/// 从应用设置解析关闭窗口行为（纯函数，可单测）
pub fn resolve_close_behavior(settings: &AppSettings) -> CloseBehavior {
    if settings.minimize_to_tray {
        CloseBehavior::Minimize
    } else {
        CloseBehavior::Exit
    }
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
            theme: Theme::default(),
            show_notification: true,
            clipboard_watch: false,
            log_level: LogLevel::default(),
            log_file_path: String::new(),
            no_log: false,
            max_concurrent_tasks: 5,
        }
    }
}

// ========================================
// 网络配置（N_m3u8DL-RE 子配置）
// ========================================

/// 网络配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// 使用系统代理
    pub use_system_proxy: bool,
    /// 自定义代理地址
    pub custom_proxy: Option<String>,
    /// BaseURL 替换
    pub base_url: Option<String>,
    /// 追加 URL 参数
    pub append_url_params: bool,
    /// 自定义请求头库
    pub headers: Vec<NetworkHeader>,
}

impl Default for NetworkConfig {
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

/// 自定义 HTTP 请求头
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkHeader {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub sort_order: i32,
}

/// 广告关键词过滤（`--ad-keyword`，正则）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdKeyword {
    pub id: i64,
    /// 分片 URL 匹配正则
    pub keyword: String,
    pub enabled: bool,
    pub sort_order: i32,
}

/// 混流导入的外部媒体文件（`--mux-import`）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MuxImport {
    pub id: i64,
    /// 文件路径
    pub path: String,
    /// 语言代码（可选，如 `chi`/`eng`）
    pub lang: Option<String>,
    /// 描述（可选，如「中文 (简体)」）
    pub name: Option<String>,
    pub enabled: bool,
    pub sort_order: i32,
}

// ========================================
// 解密配置（N_m3u8DL-RE 子配置）
// ========================================

/// 解密配置
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DecryptionConfig {
    /// 密钥文本文件路径
    pub key_text_file: Option<String>,
    /// 解密引擎
    pub engine: DecryptionEngine,
    /// 解密器二进制路径
    pub bin_path: Option<String>,
    /// 下载时实时解密
    pub real_time_decryption: bool,
    /// 自定义 HLS 解密
    pub custom_hls: CustomHlsConfig,
    /// KID:KEY 密钥库
    pub keys: Vec<DecryptionKey>,
}

/// 自定义 HLS 解密配置
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomHlsConfig {
    pub enabled: bool,
    pub method: HlsEncryptionMethod,
    pub key_type: KeyValueType,
    pub key_value: Option<String>,
    pub iv_type: KeyValueType,
    pub iv_value: Option<String>,
}

/// 解密密钥
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecryptionKey {
    pub id: i64,
    /// KID（可空，空则为纯 KEY）
    pub kid: Option<String>,
    pub key: String,
    pub sort_order: i32,
}

// ========================================
// N_m3u8DL-RE 工具配置（tool_settings["nm3u8dl"]）
// ========================================

/// N_m3u8DL-RE 工具配置
///
/// 流媒体下载引擎（HLS/DASH/MSS）的全部默认行为，
/// 含网络与解密两个子配置。任务级覆盖见 `TaskOverrides`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Nm3u8dlConfig {
    /// 工具二进制路径（空 = 自动检测）
    pub path: String,
    /// 下载线程数
    pub thread_count: u32,
    /// 下载重试次数
    pub retry_count: u32,
    /// HTTP 请求超时（秒）
    pub timeout: u32,
    /// 限速（如 "10M"，空 = 不限速）
    pub max_speed: String,
    /// 自动选择最佳流
    pub auto_select: bool,
    /// 视频流选择表达式
    pub select_video: Option<String>,
    /// 音频流选择表达式
    pub select_audio: Option<String>,
    /// 字幕流选择表达式
    pub select_subtitle: Option<String>,
    /// 视频流排除表达式
    pub drop_video: Option<String>,
    /// 音频流排除表达式
    pub drop_audio: Option<String>,
    /// 字幕流排除表达式
    pub drop_subtitle: Option<String>,
    /// 校验分片数量
    pub check_segments_count: bool,
    /// 完成后删除临时文件
    pub del_after_done: bool,
    /// 跳过合并
    pub skip_merge: bool,
    /// 写入元数据 JSON
    pub write_meta_json: bool,
    /// 二进制合并
    pub binary_merge: bool,
    /// 并发下载多个流
    pub concurrent_download: bool,
    /// 仅下载字幕
    pub sub_only: bool,
    /// 字幕格式
    pub sub_format: SubtitleFormat,
    /// 自动修正字幕时间轴
    pub auto_subtitle_fix: bool,
    /// 直播以点播方式处理
    pub live_perform_as_vod: bool,
    /// 直播实时合并
    pub live_real_time_merge: bool,
    /// 直播保留分片
    pub live_keep_segments: bool,
    /// 直播管道混流
    pub live_pipe_mux: bool,
    /// 直播按音频修正 VTT 时间轴
    pub live_fix_vtt_by_audio: bool,
    /// 直播录制时长限制（如 "01:00:00"）
    pub live_record_limit: Option<String>,
    /// 直播刷新等待时间（秒）
    pub live_wait_time: u32,
    /// 直播一次拉取的分片数
    pub live_take_count: u32,
    /// 允许 HLS 多 EXT-X-MAP
    pub allow_hls_multi_ext_map: bool,
    /// URL 处理器参数
    pub url_processor_args: Option<String>,
    /// 文件名不带日期信息
    pub no_date_info: bool,
    /// 使用 FFmpeg concat 解复用器
    pub use_ffmpeg_concat_demuxer: bool,
    /// 保存文件名模板（--save-pattern，如 `<SaveName>_<Resolution>_<Bandwidth>`）
    pub save_pattern: Option<String>,
    /// 广告关键词过滤列表（--ad-keyword）
    pub ad_keywords: Vec<AdKeyword>,
    /// 混流导入的外部媒体文件（--mux-import）
    pub mux_imports: Vec<MuxImport>,
    /// 网络子配置
    pub network: NetworkConfig,
    /// 解密子配置
    pub decryption: DecryptionConfig,
}

impl Default for Nm3u8dlConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
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
            sub_only: false,
            sub_format: SubtitleFormat::default(),
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
            save_pattern: None,
            ad_keywords: Vec::new(),
            mux_imports: Vec::new(),
            network: NetworkConfig::default(),
            decryption: DecryptionConfig::default(),
        }
    }
}

// ========================================
// FFmpeg 工具配置（tool_settings["ffmpeg"]）
// ========================================

/// HTTP basic 认证
///
/// ffmpeg 以 `-auth_type basic` + `Authorization: Basic <base64(user:pass)>` 头实现。
/// username 为空表示不启用认证。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
}

/// FFmpeg 工具配置
///
/// 覆盖三个职责：混流默认值（被 N_m3u8DL-RE 的 `-M` 参数消费）、
/// 直链视频下载默认值、ffprobe 媒体分析的二进制管理。
///
/// 直链下载字段与真实 ffmpeg 参数一一对应（均有实测验证）：
/// retry_count→`-reconnect_max_retries`、timeout→`-rw_timeout`（µs）、
/// connection_timeout→`-timeout`（µs）、preserve_timestamps→`-copyts`、
/// reconnect_attempts→`-reconnect 1 -reconnect_streamed 1`、
/// 以及 http_proxy/max_redirects/cookies/auth/reconnect_on_http_error 等。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FfmpegConfig {
    /// ffmpeg 二进制路径（空 = 自动检测）
    pub ffmpeg_path: String,
    /// ffprobe 二进制路径（空 = 自动检测）
    pub ffprobe_path: String,
    // —— 混流默认值 ——
    /// 混流容器格式
    pub mux_format: MuxFormat,
    /// 混流器
    pub muxer: Muxer,
    /// 自定义混流器路径
    pub mux_bin_path: Option<String>,
    /// 混流时跳过字幕
    pub mux_skip_subtitles: bool,
    /// 混流后保留原始文件
    pub mux_keep_original: bool,
    // —— 直链下载默认值 ——
    /// 断线重连开关（0 = 关闭；>0 输出 -reconnect 系列）
    pub reconnect_attempts: u32,
    /// 重连延迟上限（秒，→ -reconnect_delay_max）
    pub reconnect_delay: u32,
    /// 重试次数（→ -reconnect_max_retries）
    pub retry_count: u32,
    /// 网络 IO 超时（秒 → -rw_timeout 微秒）
    pub timeout: u32,
    /// 连接超时（秒 → -timeout 微秒）
    pub connection_timeout: u32,
    /// 覆盖已存在文件（-y / -n）
    pub overwrite_existing: bool,
    /// 保留输入时间戳（-copyts）
    pub preserve_timestamps: bool,
    /// 自定义 User-Agent（-user_agent）
    pub user_agent: Option<String>,
    /// 自定义 Referer（-headers Referer）
    pub referer: Option<String>,
    /// 直链代理（-http_proxy）
    pub http_proxy: Option<String>,
    /// Cookie（-cookies，换行分隔 Set-Cookie 语法）
    pub cookies: Option<String>,
    /// HTTP basic 认证（-auth_type basic + Authorization 头）
    pub auth: AuthConfig,
    /// 最大重定向次数（-max_redirects，ffmpeg 默认 8）
    pub max_redirects: u32,
    /// 对指定 HTTP 状态码重连（-reconnect_on_http_error，如 "404,429"）
    pub reconnect_on_http_error: Option<String>,
    /// 重连总时长上限（-reconnect_delay_total_max，ffmpeg 默认 256）
    pub reconnect_delay_total_max: u32,
    /// 尊重 Retry-After 头（-respect_retry_after，ffmpeg 默认 true）
    pub respect_retry_after: bool,
}

impl Default for FfmpegConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: String::new(),
            ffprobe_path: String::new(),
            mux_format: MuxFormat::default(),
            muxer: Muxer::default(),
            mux_bin_path: None,
            mux_skip_subtitles: false,
            mux_keep_original: false,
            reconnect_attempts: 3,
            reconnect_delay: 5,
            retry_count: 3,
            timeout: 60,
            connection_timeout: 30,
            overwrite_existing: false,
            preserve_timestamps: true,
            user_agent: None,
            referer: None,
            http_proxy: None,
            cookies: None,
            auth: AuthConfig::default(),
            max_redirects: 8,
            reconnect_on_http_error: None,
            reconnect_delay_total_max: 256,
            respect_retry_after: true,
        }
    }
}

// ========================================
// 工具配置集合
// ========================================

/// 全部工具配置
///
/// 引擎构建参数时的统一入参：每个引擎取自己需要的部分。
/// 新增工具时在此添加字段并在仓储加载即可。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolConfigs {
    pub nm3u8dl: Nm3u8dlConfig,
    pub ffmpeg: FfmpegConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_settings_defaults_match_frontend() {
        let s = AppSettings::default();
        assert!(s.auto_start_download);
        assert!(!s.minimize_to_tray);
        assert!(s.check_update);
        assert!(s.show_notification);
        assert!(!s.clipboard_watch);
        assert_eq!(s.log_level, LogLevel::Info);
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(serde_json::to_value(s.language).unwrap(), "zh-CN");
        assert_eq!(s.max_concurrent_tasks, 5);
    }

    #[test]
    fn nm3u8dl_defaults_match_frontend() {
        let c = Nm3u8dlConfig::default();
        assert_eq!(c.thread_count, 8);
        assert_eq!(c.retry_count, 3);
        assert_eq!(c.timeout, 100);
        assert!(c.auto_select);
        assert!(c.check_segments_count);
        assert!(c.del_after_done);
        assert!(c.auto_subtitle_fix);
        assert!(c.live_keep_segments);
        assert_eq!(c.live_take_count, 16);
        assert!(c.network.use_system_proxy);
        assert_eq!(c.decryption.engine, DecryptionEngine::Mp4Decrypt);
    }

    #[test]
    fn ffmpeg_defaults_match_frontend() {
        let c = FfmpegConfig::default();
        assert_eq!(c.retry_count, 3);
        assert_eq!(c.timeout, 60);
        assert_eq!(c.connection_timeout, 30);
        assert_eq!(c.reconnect_attempts, 3);
        assert_eq!(c.reconnect_delay, 5);
        assert!(!c.overwrite_existing);
        assert!(c.preserve_timestamps);
        assert_eq!(c.max_redirects, 8);
        assert_eq!(c.reconnect_delay_total_max, 256);
        assert!(c.respect_retry_after);
        assert!(c.auth.username.is_empty());
        assert_eq!(c.mux_format, MuxFormat::Mp4);
        assert_eq!(c.muxer, Muxer::Ffmpeg);
    }

    #[test]
    fn partial_json_falls_back_to_defaults() {
        // tool_settings 存储允许部分 JSON，缺失字段回落默认值
        let c: Nm3u8dlConfig = serde_json::from_str(r#"{"thread_count": 16}"#).unwrap();
        assert_eq!(c.thread_count, 16);
        assert_eq!(c.retry_count, 3);
        assert!(c.auto_select);
    }

    #[test]
    fn resolve_close_behavior_maps_minimize_to_tray() {
        let mut s = AppSettings::default();
        s.minimize_to_tray = true;
        assert_eq!(resolve_close_behavior(&s), CloseBehavior::Minimize);

        s.minimize_to_tray = false;
        assert_eq!(resolve_close_behavior(&s), CloseBehavior::Exit);
    }

    #[test]
    fn enum_serde_matches_frontend_strings() {
        assert_eq!(serde_json::to_string(&MuxFormat::Mkv).unwrap(), r#""mkv""#);
        assert_eq!(
            serde_json::to_string(&DecryptionEngine::Mp4Decrypt).unwrap(),
            r#""MP4DECRYPT""#
        );
        assert_eq!(
            serde_json::to_string(&HlsEncryptionMethod::Aes128).unwrap(),
            r#""AES_128""#
        );
        assert_eq!(
            serde_json::to_string(&SubtitleFormat::Vtt).unwrap(),
            r#""VTT""#
        );
        assert_eq!(
            serde_json::to_string(&LogLevel::Debug).unwrap(),
            r#""DEBUG""#
        );
    }
}
