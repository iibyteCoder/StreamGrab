//! 媒体分析器
//!
//! 领域层只定义契约，ffprobe 实现位于 `infrastructure/media/`。

use super::info::MediaInfo;
use crate::shared::AppResult;

/// 媒体分析器 Trait
///
/// 由基础设施层实现（`FfprobeAnalyzer`），封装 ffprobe 调用。
pub trait MediaAnalyzer: Send + Sync {
    /// 分析本地媒体文件
    fn analyze(&self, file_path: &str) -> AppResult<MediaInfo>;
}
