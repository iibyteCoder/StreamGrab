//! 下载领域模块
//!
//! URL 检测与引擎分派、下载引擎策略（Strategy）、流信息、进度跟踪

mod engine;
mod progress;
mod stream_info;
mod url_type;

pub use engine::*;
pub use progress::*;
pub use stream_info::*;
pub use url_type::*;
