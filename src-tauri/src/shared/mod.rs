//! 共享模块
//!
//! 提供跨层的公共错误定义与路径类型。
//! 注意：原 `types.rs`（StreamInfo/UrlType 副本）已删除，
//! 唯一来源为 `domain/download/`。

mod error;
mod path;

pub use error::*;
pub use path::ResolvedPath;
