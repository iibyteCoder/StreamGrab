//! 数据库仓库模块
//!
//! 提供各领域实体的数据访问层

pub mod config_repo;
pub mod template_repo;

pub use config_repo::*;
pub use template_repo::*;
