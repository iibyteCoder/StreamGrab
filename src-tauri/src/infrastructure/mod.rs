//! 基础设施层
//!
//! 提供外部资源访问的实现（数据库、进程、工具、平台、引擎、文件系统、媒体）

pub mod db;
pub mod engines;
pub mod fs;
pub mod media;
pub mod platform;
pub mod process;
pub mod tools;

pub use db::{Database, DbProgressRepository};
pub use platform::Platform;
pub use tools::{SuiteInfo, ToolInfo, ToolPaths};
