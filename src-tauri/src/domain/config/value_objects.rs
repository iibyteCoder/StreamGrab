//! 配置值对象
//!
//! 不可变的值对象，用于表示配置中的特定概念

use serde::{Deserialize, Serialize;

/// 下载器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloaderType {
    /// N_m3u8DL-RE - 流媒体下载
    M3U8DL,
    /// FFmpeg - 直链下载
    FFmpeg,
}

impl Default for DownloaderType {
    fn default() -> Self {
        Self::M3U8DL
    }
}

impl std::fmt::Display for DownloaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::M3U8DL => write!(f, "m3u8dl"),
            Self::FFmpeg => write!(f, "ffmpeg"),
        }
    }
}

impl std::str::FromStr for DownloaderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "m3u8dl" | "m3u8" => Ok(Self::M3U8DL),
            "ffmpeg" => Ok(Self::FFmpeg),
            _ => Err(format!("Unknown downloader type: {}", s)),
        }
    }
}

/// 解密引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DecryptionEngine {
    FFmpeg,
    MP4Decrypt,
    ShakaPackager,
}

impl Default for DecryptionEngine {
    fn default() -> Self {
        Self::MP4Decrypt
    }
}

impl std::fmt::Display for DecryptionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FFmpeg => write!(f, "FFMPEG"),
            Self::MP4Decrypt => write!(f, "MP4DECRYPT"),
            Self::ShakaPackager => write!(f, "SHAKA_PACKAGER"),
        }
    }
}

impl std::str::FromStr for DecryptionEngine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "FFMPEG" => Ok(Self::FFmpeg),
            "MP4DECRYPT" => Ok(Self::MP4Decrypt),
            "SHAKA_PACKAGER" | "SHAKA" => Ok(Self::ShakaPackager),
            _ => Err(format!("Unknown decryption engine: {}", s)),
        }
    }
}

/// 混流格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MuxFormat {
    MP4,
    MKV,
}

impl Default for MuxFormat {
    fn default() -> Self {
        Self::MP4
    }
}

impl std::fmt::Display for MuxFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MP4 => write!(f, "mp4"),
            Self::MKV => write!(f, "mkv"),
        }
    }
}

impl std::str::FromStr for MuxFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp4" => Ok(Self::MP4),
            "mkv" => Ok(Self::MKV),
            _ => Err(format!("Unknown mux format: {}", s)),
        }
    }
}

/// 混流器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Muxer {
    FFmpeg,
    MKVMerge,
}

impl Default for Muxer {
    fn default() -> Self {
        Self::FFmpeg
    }
}

impl std::fmt::Display for Muxer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FFmpeg => write!(f, "ffmpeg"),
            Self::MKVMerge => write!(f, "mkvmerge"),
        }
    }
}

impl std::str::FromStr for Muxer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ffmpeg" => Ok(Self::FFmpeg),
            "mkvmerge" => Ok(Self::MKVMerge),
            _ => Err(format!("Unknown muxer: {}", s)),
        }
    }
}

/// HLS 加密方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HlsEncryptionMethod {
    AES_128,
    AES_128_ECB,
    CENC,
    CHACHA20,
    NONE,
    SAMPLE_AES,
    SAMPLE_AES_CTR,
    UNKNOWN,
}

impl Default for HlsEncryptionMethod {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

impl std::fmt::Display for HlsEncryptionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AES_128 => write!(f, "AES_128"),
            Self::AES_128_ECB => write!(f, "AES_128_ECB"),
            Self::CENC => write!(f, "CENC"),
            Self::CHACHA20 => write!(f, "CHACHA20"),
            Self::NONE => write!(f, "NONE"),
            Self::SAMPLE_AES => write!(f, "SAMPLE_AES"),
            Self::SAMPLE_AES_CTR => write!(f, "SAMPLE_AES_CTR"),
            Self::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}

impl std::str::FromStr for HlsEncryptionMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "AES_128" => Ok(Self::AES_128),
            "AES_128_ECB" => Ok(Self::AES_128_ECB),
            "CENC" => Ok(Self::CENC),
            "CHACHA20" => Ok(Self::CHACHA20),
            "NONE" => Ok(Self::NONE),
            "SAMPLE_AES" => Ok(Self::SAMPLE_AES),
            "SAMPLE_AES_CTR" => Ok(Self::SAMPLE_AES_CTR),
            "UNKNOWN" => Ok(Self::UNKNOWN),
            _ => Err(format!("Unknown HLS encryption method: {}", s)),
        }
    }
}

/// 密钥/IV 值类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyValueType {
    File,
    Hex,
    Base64,
}

impl Default for KeyValueType {
    fn default() -> Self {
        Self::Hex
    }
}

impl std::fmt::Display for KeyValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Hex => write!(f, "hex"),
            Self::Base64 => write!(f, "base64"),
        }
    }
}

impl std::str::FromStr for KeyValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            "hex" => Ok(Self::Hex),
            "base64" => Ok(Self::Base64),
            _ => Err(format!("Unknown key value type: {}", s)),
        }
    }
}

/// 字幕格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubtitleFormat {
    SRT,
    VTT,
}

impl Default for SubtitleFormat {
    fn default() -> Self {
        Self::SRT
    }
}

impl std::fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SRT => write!(f, "SRT"),
            Self::VTT => write!(f, "VTT"),
        }
    }
}

impl std::str::FromStr for SubtitleFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SRT" => Ok(Self::SRT),
            "VTT" | "WEBVTT" => Ok(Self::VTT),
            _ => Err(format!("Unknown subtitle format: {}", s)),
        }
    }
}

/// 主题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
            Self::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Err(format!("Unknown theme: {}", s)),
        }
    }
}

/// 语言代码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "zh-CN")]
    ZhCN,
    #[serde(rename = "zh-TW")]
    ZhTW,
    #[serde(rename = "en-US")]
    EnUS,
}

impl Default for Language {
    fn default() -> Self {
        Self::ZhCN
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZhCN => write!(f, "zh-CN"),
            Self::ZhTW => write!(f, "zh-TW"),
            Self::EnUS => write!(f, "en-US"),
        }
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zh-CN" => Ok(Self::ZhCN),
            "zh-TW" => Ok(Self::ZhTW),
            "en-US" => Ok(Self::EnUS),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Off => write!(f, "OFF"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(Self::Debug),
            "INFO" => Ok(Self::Info),
            "WARN" => Ok(Self::Warn),
            "ERROR" => Ok(Self::Error),
            "OFF" => Ok(Self::Off),
            _ => Err(format!("Unknown log level: {}", s)),
        }
    }
}
