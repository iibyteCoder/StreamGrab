//! 应用层
//!
//! Tauri 命令处理，薄层委托给领域层

pub mod commands;
pub mod tray;

pub use commands::*;
pub use tray::create_tray;
