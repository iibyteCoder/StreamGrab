//! 领域层
//!
//! 核心业务逻辑，不依赖外部框架和基础设施

pub mod download;
pub mod media;
pub mod task;

pub use download::{ProgressTracker, StreamInfo, UrlType};
pub use media::MediaAnalyzer;
pub use task::TaskEntity;
