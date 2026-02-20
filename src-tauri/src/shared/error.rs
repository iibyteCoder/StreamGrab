//! 错误定义
//!
//! 统一的错误类型定义

use std::fmt;

/// 应用程序错误类型
#[derive(Debug)]
pub enum AppError {
    /// 数据库错误
    Database(String),
    /// 进程错误
    Process(String),
    /// 工具未找到
    ToolNotFound(String),
    /// 配置错误
    Config(String),
    /// IO 错误
    Io(String),
    /// 解析错误
    Parse(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "数据库错误: {}", msg),
            AppError::Process(msg) => write!(f, "进程错误: {}", msg),
            AppError::ToolNotFound(msg) => write!(f, "工具未找到: {}", msg),
            AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            AppError::Io(msg) => write!(f, "IO错误: {}", msg),
            AppError::Parse(msg) => write!(f, "解析错误: {}", msg),
            AppError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

/// 便捷的 Result 类型别名
pub type AppResult<T> = Result<T, AppError>;
