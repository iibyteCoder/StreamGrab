//! 数据仓储
//!
//! 每个仓储持有共享连接（`Arc<Mutex<Connection>>`），
//! 锁获取统一转为 `AppError::Database`（避免 Mutex 中毒级联 panic）

mod history_repo;
mod preset_repo;
mod progress_repo;
mod settings_repo;
mod task_repo;

pub use history_repo::*;
pub use preset_repo::*;
pub use progress_repo::*;
pub use settings_repo::*;
pub use task_repo::*;
