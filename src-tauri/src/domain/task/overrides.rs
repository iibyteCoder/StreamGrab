//! 任务级配置覆盖
//!
//! 「默认值 + 覆盖」三层配置模型的第二层：
//! 添加任务对话框收集 [`TaskOverrides`]，随任务持久化（`tasks.overrides_json` 列），
//! 下载时由引擎与全局默认合并（非空覆盖优先）。

use crate::domain::config::{MuxFormat, SubtitleFormat};
use serde::{Deserialize, Serialize};

/// 任务级覆盖配置
///
/// 全部字段可选：`None` 表示沿用全局默认。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskOverrides {
    /// 保存目录覆盖
    pub save_dir: Option<String>,
    /// 文件名覆盖
    pub save_name: Option<String>,
    /// 混流容器格式覆盖
    pub mux_format: Option<MuxFormat>,
    /// 限速覆盖（如 "10M"）
    pub max_speed: Option<String>,
    /// 范围下载（如 "00:00:00-00:10:00"）
    pub custom_range: Option<String>,
    /// 字幕格式覆盖
    pub subtitle_format: Option<SubtitleFormat>,
    /// 仅下载字幕覆盖
    pub subtitles_only: Option<bool>,
    /// 定时开始时间（ISO 8601 本地时间字符串，前端调度器消费）
    pub scheduled_start_at: Option<String>,
    /// 流选择覆盖（对应手动选择的 -sv/-sa/-ss）
    pub selection: Option<StreamSelection>,
    /// 来源预设 ID（溯源用）
    pub preset_id: Option<String>,
    /// 任务级解密密钥（全局密钥库为空时生效）
    pub key: Option<String>,
}

impl TaskOverrides {
    /// 是否没有任何覆盖（等价于默认值）
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// 流选择（手动选择的具体流）
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StreamSelection {
    /// 视频流选择表达式
    pub video: Option<String>,
    /// 音频流选择表达式
    pub audio: Option<String>,
    /// 字幕流选择表达式
    pub subtitle: Option<String>,
}

/// 任务执行规格
///
/// 引擎构建命令行参数的统一入参：任务本身的信息 + 已解析的覆盖配置。
/// 全局默认（工具配置/应用配置）由引擎方法的另一组参数提供。
#[derive(Debug, Clone)]
pub struct TaskSpec {
    /// 任务 ID
    pub task_id: String,
    /// 下载 URL
    pub url: String,
    /// 文件名（已应用覆盖）
    pub file_name: String,
    /// 保存目录（已应用覆盖与全局默认解析）
    pub save_dir: String,
    /// 任务级覆盖
    pub overrides: TaskOverrides,
    /// URL 类型（决定引擎与参数分支）
    pub url_type: crate::domain::download::UrlType,
}

/// 任务预设
///
/// 命名的 [`TaskOverrides`] 组合，持久化于 `task_presets` 表。
/// 应用预设 = 将 overrides 批量填入添加任务对话框。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPreset {
    pub id: String,
    pub name: String,
    /// Lucide 图标名
    pub icon: Option<String>,
    pub description: Option<String>,
    pub overrides: TaskOverrides,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_overrides_detected() {
        assert!(TaskOverrides::default().is_empty());
        let o = TaskOverrides {
            max_speed: Some("5M".into()),
            ..Default::default()
        };
        assert!(!o.is_empty());
    }

    #[test]
    fn overrides_partial_json_ok() {
        let o: TaskOverrides =
            serde_json::from_str(r#"{"muxFormat":"mkv","subtitlesOnly":true}"#).unwrap();
        assert_eq!(o.mux_format, Some(MuxFormat::Mkv));
        assert_eq!(o.subtitles_only, Some(true));
        assert!(o.save_dir.is_none());
    }
}
