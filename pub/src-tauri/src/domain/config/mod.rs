//! 配置领域模块
//!
//! 定义配置相关的领域实体、值对象和业务规则

pub mod entity;
pub mod resolver;
pub mod value_objects;

pub use entity::*;
pub use resolver::*;
pub use value_objects::*;
