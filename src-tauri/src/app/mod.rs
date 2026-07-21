//! 应用层
//!
//! Tauri 命令处理（薄层委托）与系统托盘

pub mod commands;
pub mod tray;

pub use tray::create_tray;
