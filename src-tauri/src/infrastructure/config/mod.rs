//! 环境配置模块
//!
//! 使用 TOML 配置文件管理不同环境的数据库路径和日志级别

mod loader;

pub use loader::*;
