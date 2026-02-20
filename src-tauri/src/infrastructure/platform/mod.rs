//! 平台抽象层
//!
//! 提供跨平台的配置和工具名称解析

#[path = "platform.rs"]
mod platform_impl;

pub use platform_impl::*;
