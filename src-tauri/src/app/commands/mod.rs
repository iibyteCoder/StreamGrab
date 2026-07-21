//! Tauri 命令模块（薄命令层，按域分组）
//!
//! 命令层职责：参数转换 + 委托领域/基础设施层 + 边界处 `AppResult → Result<T, String>`
//!（Tauri 前端契约要求字符串错误，类型化错误只在内部使用）

pub mod download;
pub mod history;
pub mod presets;
pub mod settings;
pub mod system;
pub mod tasks;
pub mod tools;

/// 命令边界：`AppResult` → Tauri 契约的 `Result<T, String>`
pub(crate) fn api<T>(result: crate::shared::AppResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}
