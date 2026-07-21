//! 领域层
//!
//! 核心业务逻辑，不依赖外部框架和基础设施。
//! 消费方请使用完整路径（如 `crate::domain::config::AppSettings`），
//! 不再经顶层 re-export。

pub mod config;
pub mod download;
pub mod media;
pub mod task;
