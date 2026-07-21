//! 错误定义
//!
//! 统一的类型化错误。基础设施层与领域层全部使用 [`AppResult`]，
//! 仅在 Tauri 命令层边界转换为 `String`（前端 invoke 契约要求）。

/// 应用程序错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(String),
    /// 进程错误
    #[error("进程错误: {0}")]
    Process(String),
    /// 工具未找到
    #[error("工具未找到: {0}")]
    ToolNotFound(String),
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    /// 解析错误
    #[error("解析错误: {0}")]
    Parse(String),
    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    /// 网络请求错误
    #[error("网络错误: {0}")]
    Http(#[from] reqwest::Error),
    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl AppError {
    /// 数据库错误
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    /// 进程错误
    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }

    /// 工具未找到
    pub fn tool_not_found(msg: impl Into<String>) -> Self {
        Self::ToolNotFound(msg.into())
    }

    /// 配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// 解析错误
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// 其他错误
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// 便捷的 Result 类型别名
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_category_prefix() {
        assert_eq!(
            AppError::database("table missing").to_string(),
            "数据库错误: table missing"
        );
        assert_eq!(AppError::other("boom").to_string(), "boom");
    }

    #[test]
    fn from_rusqlite_maps_to_database() {
        let err = rusqlite::Error::InvalidQuery;
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::Database(_)));
    }

    #[test]
    fn from_serde_json_maps_to_serialization() {
        let result: Result<serde_json::Value, _> = serde_json::from_str("{invalid");
        let err = result.unwrap_err();
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::Serialization(_)));
    }
}
