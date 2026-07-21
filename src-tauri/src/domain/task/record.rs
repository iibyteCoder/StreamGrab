//! 任务聚合记录
//!
//! 单表聚合模型：基础列 + 三个 JSON 列（progress / media_info / overrides），
//! 取代旧版 4 表 JOIN。仓储层负责与 SQLite 行的相互映射。

use super::entity::{ProgressData, TaskStatus};
use super::overrides::{TaskOverrides, TaskSpec};
use crate::domain::download::UrlType;
use crate::domain::media::MediaInfo;
use serde::{Deserialize, Serialize};

/// 任务聚合记录（`tasks` 表一行）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub output_path: Option<String>,
    pub status: TaskStatus,
    pub error: Option<String>,
    pub was_interrupted: bool,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    /// 实时进度（progress_json 列）
    #[serde(default)]
    pub progress: ProgressData,
    /// 媒体元数据（media_info_json 列）
    pub media_info: Option<MediaInfo>,
    /// 任务级配置覆盖（overrides_json 列）
    pub overrides: Option<TaskOverrides>,
}

impl TaskRecord {
    /// 当前本地时间（ISO 8601，无时区后缀，JS 可直接解析）
    pub fn now() -> String {
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    /// 构建引擎输入规格
    ///
    /// 覆盖已在创建时解析进 `save_dir`/`file_name`（命令层负责），
    /// 此处原样透传给引擎做参数合并。
    pub fn spec(&self) -> TaskSpec {
        TaskSpec {
            task_id: self.id.clone(),
            url: self.url.clone(),
            file_name: self.file_name.clone(),
            save_dir: self.save_dir.clone(),
            overrides: self.overrides.clone().unwrap_or_default(),
            url_type: UrlType::detect(&self.url),
        }
    }

    /// 是否已到终态
    pub fn is_finished(&self) -> bool {
        self.status.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_carries_overrides_and_detects_type() {
        let record = TaskRecord {
            id: "t1".into(),
            url: "https://example.com/index.m3u8".into(),
            file_name: "video".into(),
            save_dir: "D:/Videos".into(),
            output_path: None,
            status: TaskStatus::Pending,
            error: None,
            was_interrupted: false,
            created_at: TaskRecord::now(),
            updated_at: TaskRecord::now(),
            started_at: None,
            completed_at: None,
            progress: ProgressData::default(),
            media_info: None,
            overrides: Some(TaskOverrides {
                max_speed: Some("5M".into()),
                ..Default::default()
            }),
        };

        let spec = record.spec();
        assert_eq!(spec.url_type, UrlType::Hls);
        assert_eq!(spec.overrides.max_speed.as_deref(), Some("5M"));
        assert_eq!(spec.task_id, "t1");
    }

    #[test]
    fn serde_round_trip_is_camel_case() {
        let record = TaskRecord {
            id: "t1".into(),
            url: "u".into(),
            file_name: "f".into(),
            save_dir: "d".into(),
            output_path: None,
            status: TaskStatus::Completed,
            error: None,
            was_interrupted: false,
            created_at: TaskRecord::now(),
            updated_at: TaskRecord::now(),
            started_at: None,
            completed_at: None,
            progress: ProgressData::default(),
            media_info: None,
            overrides: None,
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["fileName"], "f");
        assert_eq!(json["saveDir"], "d");
        assert_eq!(json["status"], "completed");
        let back: TaskRecord = serde_json::from_value(json).unwrap();
        assert_eq!(back, record);
    }
}
