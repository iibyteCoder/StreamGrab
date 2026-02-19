//! 任务表操作
//!
//! 采用多表结构化设计，分离任务、进度、媒体信息、配置

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// ========================================
// 数据结构定义
// ========================================

/// 任务记录 - 核心信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub output_path: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub was_interrupted: bool,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// 任务进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TaskProgress {
    pub task_id: String,
    pub percent: i32,
    pub speed: i64,
    pub downloaded_size: i64,
    pub total_size: i64,
    pub downloaded_segments: i32,
    pub total_segments: i32,
    pub eta: i32,
    pub current_action: String,
    pub updated_at: String,
}

/// 媒体信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMediaInfo {
    pub task_id: String,
    pub resolution: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub frame_rate: Option<f64>,
    pub video_codec: Option<String>,
    pub video_range: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub audio_language: Option<String>,
    pub duration: Option<f64>,
    pub segment_count: Option<i32>,
    pub is_live: bool,
    pub is_encrypted: bool,
    pub file_format: Option<String>,
}

/// 任务配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct TaskConfig {
    pub task_id: String,
    pub thread_count: i32,
    pub retry_count: i32,
    pub timeout: i32,
    pub max_speed: String,
    pub auto_select: bool,
    pub select_video: Option<String>,
    pub select_audio: Option<String>,
    pub select_subtitle: Option<String>,
    pub drop_video: Option<String>,
    pub drop_audio: Option<String>,
    pub drop_subtitle: Option<String>,
    pub mux_format: String,
    pub mux_after_done: bool,
    pub skip_merge: bool,
    pub del_after_done: bool,
    pub check_segments_count: bool,
    pub custom_range: Option<String>,
    pub key: Option<String>,
    pub proxy: Option<String>,
}

/// 任务请求头
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TaskHeader {
    pub id: i64,
    pub task_id: String,
    pub name: String,
    pub value: String,
    pub enabled: bool,
}

/// 完整任务信息（JOIN 查询结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTaskRecord {
    // 任务基本信息
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub output_path: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub was_interrupted: bool,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    // 进度信息
    pub progress_percent: i32,
    pub progress_speed: i64,
    pub progress_downloaded_size: i64,
    pub progress_total_size: i64,
    pub progress_downloaded_segments: i32,
    pub progress_total_segments: i32,
    pub progress_eta: i32,
    pub progress_current_action: String,
    // 媒体信息（可选）
    pub media_resolution: Option<String>,
    pub media_width: Option<i32>,
    pub media_height: Option<i32>,
    pub media_frame_rate: Option<f64>,
    pub media_video_codec: Option<String>,
    pub media_video_range: Option<String>,
    pub media_audio_codec: Option<String>,
    pub media_audio_channels: Option<String>,
    pub media_audio_language: Option<String>,
    pub media_duration: Option<f64>,
    pub media_segment_count: Option<i32>,
    pub media_is_live: bool,
    pub media_is_encrypted: bool,
    pub media_file_format: Option<String>,
}

// ========================================
// 数据库管理器
// ========================================

/// 任务数据库管理器
pub struct TaskDb {
    conn: Mutex<Connection>,
}

impl TaskDb {
    /// 创建任务管理器
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ========================================
    // 任务基本操作
    // ========================================

    /// 加载所有任务（完整信息，包含进度和媒体信息）
    pub fn load_all(&self) -> Result<Vec<FullTaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT
                    t.id, t.url, t.file_name, t.save_dir, t.output_path, t.status, t.error,
                    t.was_interrupted, t.created_at, t.updated_at, t.started_at, t.completed_at,
                    COALESCE(p.percent, 0) as progress_percent,
                    COALESCE(p.speed, 0) as progress_speed,
                    COALESCE(p.downloaded_size, 0) as progress_downloaded_size,
                    COALESCE(p.total_size, 0) as progress_total_size,
                    COALESCE(p.downloaded_segments, 0) as progress_downloaded_segments,
                    COALESCE(p.total_segments, 0) as progress_total_segments,
                    COALESCE(p.eta, 0) as progress_eta,
                    COALESCE(p.current_action, '') as progress_current_action,
                    m.resolution as media_resolution,
                    m.width as media_width,
                    m.height as media_height,
                    m.frame_rate as media_frame_rate,
                    m.video_codec as media_video_codec,
                    m.video_range as media_video_range,
                    m.audio_codec as media_audio_codec,
                    m.audio_channels as media_audio_channels,
                    m.audio_language as media_audio_language,
                    m.duration as media_duration,
                    m.segment_count as media_segment_count,
                    COALESCE(m.is_live, 0) as media_is_live,
                    COALESCE(m.is_encrypted, 0) as media_is_encrypted,
                    m.file_format as media_file_format
                 FROM tasks t
                 LEFT JOIN task_progress p ON t.id = p.task_id
                 LEFT JOIN task_media_info m ON t.id = m.task_id
                 ORDER BY t.created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(FullTaskRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_dir: row.get(3)?,
                    output_path: row.get(4)?,
                    status: row.get(5)?,
                    error: row.get(6)?,
                    was_interrupted: row.get::<_, i64>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    started_at: row.get(10)?,
                    completed_at: row.get(11)?,
                    progress_percent: row.get(12)?,
                    progress_speed: row.get(13)?,
                    progress_downloaded_size: row.get(14)?,
                    progress_total_size: row.get(15)?,
                    progress_downloaded_segments: row.get(16)?,
                    progress_total_segments: row.get(17)?,
                    progress_eta: row.get(18)?,
                    progress_current_action: row.get(19)?,
                    media_resolution: row.get(20)?,
                    media_width: row.get(21)?,
                    media_height: row.get(22)?,
                    media_frame_rate: row.get(23)?,
                    media_video_codec: row.get(24)?,
                    media_video_range: row.get(25)?,
                    media_audio_codec: row.get(26)?,
                    media_audio_channels: row.get(27)?,
                    media_audio_language: row.get(28)?,
                    media_duration: row.get(29)?,
                    media_segment_count: row.get(30)?,
                    media_is_live: row.get::<_, i64>(31)? != 0,
                    media_is_encrypted: row.get::<_, i64>(32)? != 0,
                    media_file_format: row.get(33)?,
                })
            })
            .map_err(|e| format!("Failed to query tasks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tasks: {}", e))?;

        Ok(tasks)
    }

    /// 根据 ID 获取任务
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Result<Option<FullTaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT
                    t.id, t.url, t.file_name, t.save_dir, t.output_path, t.status, t.error,
                    t.was_interrupted, t.created_at, t.updated_at, t.started_at, t.completed_at,
                    COALESCE(p.percent, 0) as progress_percent,
                    COALESCE(p.speed, 0) as progress_speed,
                    COALESCE(p.downloaded_size, 0) as progress_downloaded_size,
                    COALESCE(p.total_size, 0) as progress_total_size,
                    COALESCE(p.downloaded_segments, 0) as progress_downloaded_segments,
                    COALESCE(p.total_segments, 0) as progress_total_segments,
                    COALESCE(p.eta, 0) as progress_eta,
                    COALESCE(p.current_action, '') as progress_current_action,
                    m.resolution as media_resolution,
                    m.width as media_width,
                    m.height as media_height,
                    m.frame_rate as media_frame_rate,
                    m.video_codec as media_video_codec,
                    m.video_range as media_video_range,
                    m.audio_codec as media_audio_codec,
                    m.audio_channels as media_audio_channels,
                    m.audio_language as media_audio_language,
                    m.duration as media_duration,
                    m.segment_count as media_segment_count,
                    COALESCE(m.is_live, 0) as media_is_live,
                    COALESCE(m.is_encrypted, 0) as media_is_encrypted,
                    m.file_format as media_file_format
                 FROM tasks t
                 LEFT JOIN task_progress p ON t.id = p.task_id
                 LEFT JOIN task_media_info m ON t.id = m.task_id
                 WHERE t.id = ?1",
                params![id],
                |row| {
                    Ok(FullTaskRecord {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        file_name: row.get(2)?,
                        save_dir: row.get(3)?,
                        output_path: row.get(4)?,
                        status: row.get(5)?,
                        error: row.get(6)?,
                        was_interrupted: row.get::<_, i64>(7)? != 0,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                        started_at: row.get(10)?,
                        completed_at: row.get(11)?,
                        progress_percent: row.get(12)?,
                        progress_speed: row.get(13)?,
                        progress_downloaded_size: row.get(14)?,
                        progress_total_size: row.get(15)?,
                        progress_downloaded_segments: row.get(16)?,
                        progress_total_segments: row.get(17)?,
                        progress_eta: row.get(18)?,
                        progress_current_action: row.get(19)?,
                        media_resolution: row.get(20)?,
                        media_width: row.get(21)?,
                        media_height: row.get(22)?,
                        media_frame_rate: row.get(23)?,
                        media_video_codec: row.get(24)?,
                        media_video_range: row.get(25)?,
                        media_audio_codec: row.get(26)?,
                        media_audio_channels: row.get(27)?,
                        media_audio_language: row.get(28)?,
                        media_duration: row.get(29)?,
                        media_segment_count: row.get(30)?,
                        media_is_live: row.get::<_, i64>(31)? != 0,
                        media_is_encrypted: row.get::<_, i64>(32)? != 0,
                        media_file_format: row.get(33)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query task: {}", e))?;

        Ok(result)
    }

    /// 创建任务（同时创建关联的进度记录）
    pub fn create(&self, task: &TaskRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 插入任务
        conn.execute(
            "INSERT INTO tasks (id, url, file_name, save_dir, output_path, status, error,
                              was_interrupted, created_at, updated_at, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                task.id,
                task.url,
                task.file_name,
                task.save_dir,
                task.output_path,
                task.status,
                task.error,
                task.was_interrupted as i64,
                task.created_at,
                task.updated_at,
                task.started_at,
                task.completed_at,
            ],
        )
        .map_err(|e| format!("Failed to create task: {}", e))?;

        // 创建初始进度记录
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO task_progress (task_id, updated_at)
             VALUES (?1, ?2)",
            params![task.id, now],
        )
        .map_err(|e| format!("Failed to create task progress: {}", e))?;

        Ok(())
    }

    /// 更新任务状态
    pub fn update_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE tasks SET status = ?1, error = ?2, updated_at = ?3 WHERE id = ?4",
            params![status, error, now, id],
        )
        .map_err(|e| format!("Failed to update task status: {}", e))?;

        Ok(())
    }

    /// 更新任务输出路径
    pub fn update_output_path(&self, id: &str, output_path: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE tasks SET output_path = ?1, updated_at = ?2 WHERE id = ?3",
            params![output_path, now, id],
        )
        .map_err(|e| format!("Failed to update task output path: {}", e))?;

        Ok(())
    }

    /// 删除任务（级联删除关联数据）
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete task: {}", e))?;

        Ok(())
    }

    // ========================================
    // 进度操作
    // ========================================

    /// 更新任务进度
    #[allow(clippy::too_many_arguments)]
    pub fn update_progress(
        &self,
        task_id: &str,
        percent: i32,
        speed: i64,
        downloaded_size: i64,
        total_size: i64,
        downloaded_segments: i32,
        total_segments: i32,
        eta: i32,
        current_action: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO task_progress (task_id, percent, speed, downloaded_size, total_size,
                                       downloaded_segments, total_segments, eta, current_action, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(task_id) DO UPDATE SET
                 percent = excluded.percent,
                 speed = excluded.speed,
                 downloaded_size = excluded.downloaded_size,
                 total_size = excluded.total_size,
                 downloaded_segments = excluded.downloaded_segments,
                 total_segments = excluded.total_segments,
                 eta = excluded.eta,
                 current_action = excluded.current_action,
                 updated_at = excluded.updated_at",
            params![
                task_id,
                percent,
                speed,
                downloaded_size,
                total_size,
                downloaded_segments,
                total_segments,
                eta,
                current_action,
                now
            ],
        )
        .map_err(|e| format!("Failed to update task progress: {}", e))?;

        Ok(())
    }

    // ========================================
    // 媒体信息操作
    // ========================================

    /// 更新任务媒体信息
    pub fn update_media_info(&self, task_id: &str, info: &TaskMediaInfo) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO task_media_info (task_id, resolution, width, height, frame_rate,
                                         video_codec, video_range, audio_codec, audio_channels,
                                         audio_language, duration, segment_count, is_live, is_encrypted, file_format)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(task_id) DO UPDATE SET
                 resolution = excluded.resolution,
                 width = excluded.width,
                 height = excluded.height,
                 frame_rate = excluded.frame_rate,
                 video_codec = excluded.video_codec,
                 video_range = excluded.video_range,
                 audio_codec = excluded.audio_codec,
                 audio_channels = excluded.audio_channels,
                 audio_language = excluded.audio_language,
                 duration = excluded.duration,
                 segment_count = excluded.segment_count,
                 is_live = excluded.is_live,
                 is_encrypted = excluded.is_encrypted,
                 file_format = excluded.file_format",
            params![
                task_id,
                info.resolution,
                info.width,
                info.height,
                info.frame_rate,
                info.video_codec,
                info.video_range,
                info.audio_codec,
                info.audio_channels,
                info.audio_language,
                info.duration,
                info.segment_count,
                info.is_live as i64,
                info.is_encrypted as i64,
                info.file_format,
            ],
        )
        .map_err(|e| format!("Failed to update task media info: {}", e))?;

        Ok(())
    }

    // ========================================
    // 配置操作
    // ========================================

    /// 获取任务配置
    #[allow(dead_code)]
    pub fn get_config(&self, task_id: &str) -> Result<Option<TaskConfig>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT task_id, thread_count, retry_count, timeout, max_speed, auto_select,
                        select_video, select_audio, select_subtitle, drop_video, drop_audio, drop_subtitle,
                        mux_format, mux_after_done, skip_merge, del_after_done, check_segments_count,
                        custom_range, key, proxy
                 FROM task_config WHERE task_id = ?1",
                params![task_id],
                |row| {
                    Ok(TaskConfig {
                        task_id: row.get(0)?,
                        thread_count: row.get(1)?,
                        retry_count: row.get(2)?,
                        timeout: row.get(3)?,
                        max_speed: row.get(4)?,
                        auto_select: row.get::<_, i64>(5)? != 0,
                        select_video: row.get(6)?,
                        select_audio: row.get(7)?,
                        select_subtitle: row.get(8)?,
                        drop_video: row.get(9)?,
                        drop_audio: row.get(10)?,
                        drop_subtitle: row.get(11)?,
                        mux_format: row.get(12)?,
                        mux_after_done: row.get::<_, i64>(13)? != 0,
                        skip_merge: row.get::<_, i64>(14)? != 0,
                        del_after_done: row.get::<_, i64>(15)? != 0,
                        check_segments_count: row.get::<_, i64>(16)? != 0,
                        custom_range: row.get(17)?,
                        key: row.get(18)?,
                        proxy: row.get(19)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to get task config: {}", e))?;

        Ok(result)
    }

    /// 保存任务配置
    #[allow(dead_code)]
    pub fn save_config(&self, config: &TaskConfig) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO task_config (task_id, thread_count, retry_count, timeout, max_speed, auto_select,
                                     select_video, select_audio, select_subtitle, drop_video, drop_audio, drop_subtitle,
                                     mux_format, mux_after_done, skip_merge, del_after_done, check_segments_count,
                                     custom_range, key, proxy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
             ON CONFLICT(task_id) DO UPDATE SET
                 thread_count = excluded.thread_count,
                 retry_count = excluded.retry_count,
                 timeout = excluded.timeout,
                 max_speed = excluded.max_speed,
                 auto_select = excluded.auto_select,
                 select_video = excluded.select_video,
                 select_audio = excluded.select_audio,
                 select_subtitle = excluded.select_subtitle,
                 drop_video = excluded.drop_video,
                 drop_audio = excluded.drop_audio,
                 drop_subtitle = excluded.drop_subtitle,
                 mux_format = excluded.mux_format,
                 mux_after_done = excluded.mux_after_done,
                 skip_merge = excluded.skip_merge,
                 del_after_done = excluded.del_after_done,
                 check_segments_count = excluded.check_segments_count,
                 custom_range = excluded.custom_range,
                 key = excluded.key,
                 proxy = excluded.proxy",
            params![
                config.task_id,
                config.thread_count,
                config.retry_count,
                config.timeout,
                config.max_speed,
                config.auto_select as i64,
                config.select_video,
                config.select_audio,
                config.select_subtitle,
                config.drop_video,
                config.drop_audio,
                config.drop_subtitle,
                config.mux_format,
                config.mux_after_done as i64,
                config.skip_merge as i64,
                config.del_after_done as i64,
                config.check_segments_count as i64,
                config.custom_range,
                config.key,
                config.proxy,
            ],
        )
        .map_err(|e| format!("Failed to save task config: {}", e))?;

        Ok(())
    }

    // ========================================
    // 批量操作
    // ========================================

    /// 删除指定状态的任务
    pub fn delete_by_status(&self, statuses: &[&str]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let placeholders: Vec<String> = statuses.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM tasks WHERE status IN ({})",
            placeholders.join(", ")
        );
        let params: Vec<&dyn rusqlite::ToSql> =
            statuses.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let rows_affected = conn
            .execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to delete tasks by status: {}", e))?;

        Ok(rows_affected)
    }

    /// 清除已完成的任务
    pub fn clear_finished(&self) -> Result<usize, String> {
        self.delete_by_status(&["completed", "failed", "cancelled"])
    }

    /// 清除所有任务
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM tasks", [])
            .map_err(|e| format!("Failed to clear tasks: {}", e))?;

        Ok(())
    }

    /// 标记活跃任务为已中断
    pub fn mark_active_interrupted(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let rows_affected = conn
            .execute(
                "UPDATE tasks SET was_interrupted = 1, status = 'paused'
                 WHERE status IN ('downloading', 'analyzing', 'merging', 'muxing')",
                [],
            )
            .map_err(|e| format!("Failed to mark tasks as interrupted: {}", e))?;

        Ok(rows_affected)
    }

    /// 加载可恢复的任务
    pub fn load_recoverable(&self) -> Result<Vec<FullTaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT
                    t.id, t.url, t.file_name, t.save_dir, t.output_path, t.status, t.error,
                    t.was_interrupted, t.created_at, t.updated_at, t.started_at, t.completed_at,
                    COALESCE(p.percent, 0) as progress_percent,
                    COALESCE(p.speed, 0) as progress_speed,
                    COALESCE(p.downloaded_size, 0) as progress_downloaded_size,
                    COALESCE(p.total_size, 0) as progress_total_size,
                    COALESCE(p.downloaded_segments, 0) as progress_downloaded_segments,
                    COALESCE(p.total_segments, 0) as progress_total_segments,
                    COALESCE(p.eta, 0) as progress_eta,
                    COALESCE(p.current_action, '') as progress_current_action,
                    m.resolution as media_resolution,
                    m.width as media_width,
                    m.height as media_height,
                    m.frame_rate as media_frame_rate,
                    m.video_codec as media_video_codec,
                    m.video_range as media_video_range,
                    m.audio_codec as media_audio_codec,
                    m.audio_channels as media_audio_channels,
                    m.audio_language as media_audio_language,
                    m.duration as media_duration,
                    m.segment_count as media_segment_count,
                    COALESCE(m.is_live, 0) as media_is_live,
                    COALESCE(m.is_encrypted, 0) as media_is_encrypted,
                    m.file_format as media_file_format
                 FROM tasks t
                 LEFT JOIN task_progress p ON t.id = p.task_id
                 LEFT JOIN task_media_info m ON t.id = m.task_id
                 WHERE t.was_interrupted = 1 OR t.status IN ('downloading', 'paused', 'analyzing')
                 ORDER BY t.created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(FullTaskRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_dir: row.get(3)?,
                    output_path: row.get(4)?,
                    status: row.get(5)?,
                    error: row.get(6)?,
                    was_interrupted: row.get::<_, i64>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    started_at: row.get(10)?,
                    completed_at: row.get(11)?,
                    progress_percent: row.get(12)?,
                    progress_speed: row.get(13)?,
                    progress_downloaded_size: row.get(14)?,
                    progress_total_size: row.get(15)?,
                    progress_downloaded_segments: row.get(16)?,
                    progress_total_segments: row.get(17)?,
                    progress_eta: row.get(18)?,
                    progress_current_action: row.get(19)?,
                    media_resolution: row.get(20)?,
                    media_width: row.get(21)?,
                    media_height: row.get(22)?,
                    media_frame_rate: row.get(23)?,
                    media_video_codec: row.get(24)?,
                    media_video_range: row.get(25)?,
                    media_audio_codec: row.get(26)?,
                    media_audio_channels: row.get(27)?,
                    media_audio_language: row.get(28)?,
                    media_duration: row.get(29)?,
                    media_segment_count: row.get(30)?,
                    media_is_live: row.get::<_, i64>(31)? != 0,
                    media_is_encrypted: row.get::<_, i64>(32)? != 0,
                    media_file_format: row.get(33)?,
                })
            })
            .map_err(|e| format!("Failed to query recoverable tasks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tasks: {}", e))?;

        Ok(tasks)
    }

    // ========================================
    // 进度历史操作
    // ========================================

    /// 添加进度历史记录
    pub fn add_progress_history(
        &self,
        task_id: &str,
        percent: i32,
        speed: i64,
        downloaded_size: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let timestamp = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO progress_history (task_id, timestamp, percent, speed, downloaded_size)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![task_id, timestamp, percent, speed, downloaded_size],
        )
        .map_err(|e| format!("Failed to add progress history: {}", e))?;

        Ok(())
    }

    /// 获取任务的进度历史
    pub fn get_progress_history(
        &self,
        task_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ProgressHistoryRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let mut stmt = conn
            .prepare(&format!(
                "SELECT id, task_id, timestamp, percent, speed, downloaded_size
                 FROM progress_history
                 WHERE task_id = ?
                 ORDER BY timestamp ASC
                 {}",
                limit_clause
            ))
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let records = stmt
            .query_map(params![task_id], |row| {
                Ok(ProgressHistoryRecord {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    percent: row.get(3)?,
                    speed: row.get(4)?,
                    downloaded_size: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query progress history: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect records: {}", e))?;

        Ok(records)
    }

    /// 清除任务的进度历史
    pub fn clear_progress_history(&self, task_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM progress_history WHERE task_id = ?",
            params![task_id],
        )
        .map_err(|e| format!("Failed to clear progress history: {}", e))?;

        Ok(())
    }
}

/// 进度历史记录
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressHistoryRecord {
    pub id: i64,
    pub task_id: String,
    pub timestamp: String,
    pub percent: i32,
    pub speed: i64,
    pub downloaded_size: i64,
}
