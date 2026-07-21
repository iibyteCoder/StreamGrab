//! 进度跟踪模块
//!
//! 负责收集、缓冲和持久化下载进度历史
//!
//! ## 架构设计
//!
//! 采用生产者-消费者模式：
//! 1. 下载进程产生进度数据 -> 调用 `record_progress()`
//! 2. 进度数据被缓冲在内存中
//! 3. 定时或完成时批量写入数据库
//!
//! ## 采样策略
//!
//! 采用双维度采样：
//! - 进度维度：每隔 N% 记录一次
//! - 速度维度：速度变化超过 M% 时额外记录

use crate::shared::AppResult;
use dashmap::DashMap;
use std::sync::Arc;

/// 进度数据点
#[derive(Debug, Clone)]
pub struct ProgressPoint {
    /// 进度百分比 (0-100)
    pub percent: i32,
    /// 下载速度 (bytes/s)
    pub speed: i64,
    /// 已下载大小 (bytes)
    pub downloaded_size: i64,
    /// 时间戳 (毫秒)
    pub timestamp: i64,
}

/// 任务进度缓冲区
#[derive(Debug, Clone)]
struct TaskProgressBuffer {
    /// 缓冲的数据点
    points: Vec<ProgressPoint>,
    /// 上次保存时间（毫秒）
    last_save_time: i64,
    /// 上次记录的进度百分比
    last_recorded_percent: i32,
}

impl Default for TaskProgressBuffer {
    fn default() -> Self {
        Self {
            points: Vec::with_capacity(64),
            last_save_time: 0,
            last_recorded_percent: -100,
        }
    }
}

/// 进度历史持久化接口
///
/// 由基础设施层实现，解耦领域层与数据库
pub trait ProgressRepository: Send + Sync {
    /// 保存进度历史记录
    fn save(&self, task_id: &str, points: &[ProgressPoint]) -> AppResult<()>;
}

/// 进度跟踪器
///
/// 负责收集和缓冲进度数据
pub struct ProgressTracker {
    /// 任务进度缓冲区 (task_id -> buffer)
    buffers: DashMap<String, TaskProgressBuffer>,
    /// 持久化仓库
    repository: Arc<dyn ProgressRepository>,
    /// 保存间隔（毫秒）
    save_interval_ms: i64,
    /// 进度间隔（百分比）
    progress_interval: i32,
    /// 速度变化阈值（百分比）
    speed_change_threshold: f64,
}

impl ProgressTracker {
    /// 创建新的进度跟踪器
    pub fn new(repository: Arc<dyn ProgressRepository>) -> Self {
        Self {
            buffers: DashMap::new(),
            repository,
            save_interval_ms: 2000,      // 2 秒保存一次
            progress_interval: 2,        // 每 2% 进度记录一次
            speed_change_threshold: 0.2, // 速度变化 20% 时记录
        }
    }

    /// 记录进度数据点
    ///
    /// 采用双维度采样策略
    pub fn record_progress(&self, task_id: &str, percent: i32, speed: i64, downloaded_size: i64) {
        let now = chrono::Utc::now().timestamp_millis();

        let mut buffer = self.buffers.entry(task_id.to_string()).or_default();

        // 进度维度采样
        let should_record = if percent - buffer.last_recorded_percent >= self.progress_interval {
            true
        } else if !buffer.points.is_empty() {
            // 速度维度采样
            let last_speed = buffer.points.last().map(|p| p.speed).unwrap_or(0);
            if last_speed > 0 && speed > 0 {
                let change_ratio = ((speed - last_speed) as f64 / last_speed as f64).abs();
                change_ratio > self.speed_change_threshold
            } else {
                false
            }
        } else {
            percent > 0 // 第一个数据点
        };

        if !should_record {
            return;
        }

        // 添加数据点
        buffer.points.push(ProgressPoint {
            percent,
            speed,
            downloaded_size,
            timestamp: now,
        });
        buffer.last_recorded_percent = percent;

        // 检查是否需要保存
        if now - buffer.last_save_time >= self.save_interval_ms {
            let points_to_save: Vec<_> = buffer.points.drain(..).collect();
            buffer.last_save_time = now;
            drop(buffer);

            // 保存到仓库
            if let Err(e) = self.repository.save(task_id, &points_to_save) {
                log::error!("Failed to save progress history: {}", e);
            }
        }
    }

    /// 强制刷新任务的所有缓冲数据
    ///
    /// 在下载完成或任务结束时调用
    pub fn flush(&self, task_id: &str) {
        if let Some((_, buffer)) = self.buffers.remove(task_id) {
            if !buffer.points.is_empty() {
                if let Err(e) = self.repository.save(task_id, &buffer.points) {
                    log::error!("Failed to flush progress history: {}", e);
                }
            }
        }
    }

    /// 清除任务的缓冲区（不保存）
    pub fn clear(&self, task_id: &str) {
        self.buffers.remove(task_id);
    }
}

// ========================================
// 全局实例（用于简单场景）
// ========================================

use once_cell::sync::OnceCell;
use std::sync::Mutex;

static PROGRESS_TRACKER: OnceCell<Mutex<Option<Arc<ProgressTracker>>>> = OnceCell::new();

/// 初始化全局进度跟踪器
pub fn init_progress_tracker(repository: Arc<dyn ProgressRepository>) {
    let tracker = ProgressTracker::new(repository);
    let _ = PROGRESS_TRACKER.set(Mutex::new(Some(Arc::new(tracker))));
}

/// 获取全局进度跟踪器
pub fn get_progress_tracker() -> Option<Arc<ProgressTracker>> {
    PROGRESS_TRACKER
        .get()
        .and_then(|m| m.lock().ok())
        .and_then(|m| m.clone())
}

/// 便捷函数：记录进度
pub fn record_progress(task_id: &str, percent: i32, speed: i64, downloaded_size: i64) {
    if let Some(tracker) = get_progress_tracker() {
        tracker.record_progress(task_id, percent, speed, downloaded_size);
    }
}

/// 便捷函数：刷新进度
pub fn flush_progress(task_id: &str) {
    if let Some(tracker) = get_progress_tracker() {
        tracker.flush(task_id);
    }
}

/// 便捷函数：清除进度缓冲
pub fn clear_progress_buffer(task_id: &str) {
    if let Some(tracker) = get_progress_tracker() {
        tracker.clear(task_id);
    }
}
