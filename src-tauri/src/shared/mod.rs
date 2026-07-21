//! 共享模块
//!
//! 提供跨层的公共错误定义。
//! 注意：原 `types.rs`（StreamInfo/UrlType 副本）已删除，
//! 唯一来源为 `domain/download/`。

mod error;

pub use error::*;
