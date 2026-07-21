//! 下载引擎实现（策略实现层）
//!
//! 每个外部工具一个子模块，实现 `domain::download::DownloadEngine`。
//!
//! ## 新增工具五步清单
//!
//! 1. `infrastructure/tools/` 注册 `ToolDefinition`
//! 2. `domain/config.rs` 加工具配置类型 + `ToolConfigs` 字段
//! 3. 本目录新增 `<tool>/`（`args.rs` + `parser.rs` + `mod.rs` 实现 trait）
//! 4. `domain/download/url_type.rs` 加分派规则
//! 5. 前端加设置标签页

pub mod ffmpeg;
pub mod nm3u8dl;

use crate::domain::download::EngineRegistry;

/// 构建默认引擎注册表
///
/// 注册顺序决定 `Unknown` URL 的兜底引擎：N_m3u8DL-RE 先注册（格式覆盖最广）
pub fn default_registry() -> EngineRegistry {
    let mut registry = EngineRegistry::new();
    registry.register(Box::new(nm3u8dl::Nm3u8dlEngine::new()));
    registry.register(Box::new(ffmpeg::FfmpegEngine::new()));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::download::{ToolId, UrlType};

    #[test]
    fn default_registry_dispatches_all_types() {
        let registry = default_registry();
        assert_eq!(
            registry.for_url(UrlType::Hls).unwrap().id(),
            ToolId::Nm3u8dl
        );
        assert_eq!(
            registry.for_url(UrlType::Dash).unwrap().id(),
            ToolId::Nm3u8dl
        );
        assert_eq!(
            registry.for_url(UrlType::Mss).unwrap().id(),
            ToolId::Nm3u8dl
        );
        assert_eq!(
            registry.for_url(UrlType::HttpVideo).unwrap().id(),
            ToolId::Ffmpeg
        );
        // Unknown 兜底到 N_m3u8DL-RE
        assert_eq!(
            registry.for_url(UrlType::Unknown).unwrap().id(),
            ToolId::Nm3u8dl
        );

        assert_eq!(registry.get(ToolId::Ffmpeg).unwrap().id(), ToolId::Ffmpeg);
    }
}
