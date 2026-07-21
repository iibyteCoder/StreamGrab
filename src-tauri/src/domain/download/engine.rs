//! 下载引擎抽象（策略模式）
//!
//! 每个外部下载工具实现一个 [`DownloadEngine`]：自包含命令行参数构建、
//! 输出解析与进度模型。命令层按 URL 类型从 [`EngineRegistry`] 取得引擎，
//! 不感知具体工具的存在与 CLI 细节。
//!
//! ## 新增工具五步清单（扩展契约）
//!
//! 1. `infrastructure/tools/` 注册 `ToolDefinition`（二进制检测/版本获取/下载更新）
//! 2. `domain/config.rs` 新增工具配置类型并加入 [`ToolConfigs`]，仓储侧 `tool_settings` 自动支持
//! 3. `infrastructure/engines/<tool>/` 实现 [`DownloadEngine`]（`args.rs` 参数构建 + `parser.rs` 输出解析）
//! 4. [`UrlType::engine`] 增加一条分派规则
//! 5. 前端增加一个设置标签页组件
//!
//! 刻意不引入插件框架：静态注册 + trait 对象对这个规模的工具集已足够。

use crate::domain::config::{AppSettings, ToolConfigs};
use crate::domain::download::{StreamInfo, UrlType};
use crate::domain::task::{ProgressData, TaskSpec};
use crate::shared::AppError;
use serde::{Deserialize, Serialize};

/// 工具标识
///
/// 与 `tool_settings` 表的行键、`ToolRegistry` 的注册键一一对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    /// N_m3u8DL-RE：流媒体下载（HLS/DASH/MSS）
    Nm3u8dl,
    /// FFmpeg：直链视频下载 / 混流 / 探测
    Ffmpeg,
}

impl ToolId {
    /// 持久化键
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nm3u8dl => "nm3u8dl",
            Self::Ffmpeg => "ffmpeg",
        }
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ToolId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nm3u8dl" => Ok(Self::Nm3u8dl),
            "ffmpeg" => Ok(Self::Ffmpeg),
            other => Err(AppError::config(format!("未知工具: {other}"))),
        }
    }
}

/// 引擎从进程输出中解析出的事件
///
/// 由进程管理器包装为 Tauri 事件推送给前端（观察者模式的领域侧）
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineEvent {
    /// 日志行
    Log { level: String, message: String },
    /// 进度更新
    Progress { data: ProgressData },
    /// 阶段变化（如 merging / muxing）
    Status { action: String },
}

/// 下载引擎策略
///
/// 引擎实例全局共享、无状态；逐任务的输出解析状态由 [`EngineSession`] 持有。
pub trait DownloadEngine: Send + Sync {
    /// 工具标识
    fn id(&self) -> ToolId;

    /// 是否处理该 URL 类型
    fn handles(&self, url_type: UrlType) -> bool;

    /// 构建下载命令行参数
    ///
    /// 合并规则：任务级覆盖（非空）> 全局默认。
    /// 参数不含程序路径本身（由进程管理层拼接）。
    fn build_download_args(
        &self,
        spec: &TaskSpec,
        tools: &ToolConfigs,
        app: &AppSettings,
    ) -> Vec<String>;

    /// 构建解析模式命令行参数（仅解析流信息，不下载）
    fn build_parse_args(&self, url: &str, tools: &ToolConfigs, app: &AppSettings) -> Vec<String>;

    /// 解析「解析模式」的完整输出为流信息（`parse_url` 使用）
    ///
    /// N_m3u8DL-RE 引擎接收自身 stdout；FFmpeg 引擎接收 ffprobe 的 JSON 输出。
    fn parse_streams(&self, stdout: &str) -> StreamInfo;

    /// 创建逐任务的输出解析会话
    ///
    /// 会话持有跨行解析状态（如 FFmpeg 的进度缓冲、N_m3u8DL-RE 的
    /// 视频/音频双流分片进度聚合），生命周期与单次下载相同。
    fn new_session(&self) -> Box<dyn EngineSession>;
}

/// 逐任务的引擎输出解析会话
pub trait EngineSession: Send {
    /// 解析单行进程输出为事件（逐行调用，需保持低开销）
    fn parse_line(&mut self, line: &str) -> Option<EngineEvent>;
}

/// 引擎注册表：按 URL 类型分派策略
///
/// 注册顺序决定 `Unknown` 类型的回退引擎（应首先注册 N_m3u8DL-RE，
/// 其格式覆盖最广，作为兜底）。
#[derive(Default)]
pub struct EngineRegistry {
    engines: Vec<Box<dyn DownloadEngine>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册引擎
    pub fn register(&mut self, engine: Box<dyn DownloadEngine>) {
        self.engines.push(engine);
    }

    /// 按 URL 类型分派；`Unknown` 回退到首个注册的引擎
    pub fn for_url(&self, url_type: UrlType) -> Option<&dyn DownloadEngine> {
        self.engines
            .iter()
            .find(|e| e.handles(url_type))
            .or_else(|| self.engines.first())
            .map(|e| e.as_ref())
    }

    /// 按工具标识获取
    pub fn get(&self, id: ToolId) -> Option<&dyn DownloadEngine> {
        self.engines
            .iter()
            .find(|e| e.id() == id)
            .map(|e| e.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_id_str_round_trip() {
        for (id, s) in [(ToolId::Nm3u8dl, "nm3u8dl"), (ToolId::Ffmpeg, "ffmpeg")] {
            assert_eq!(id.as_str(), s);
            assert_eq!(id.to_string(), s);
            assert_eq!(s.parse::<ToolId>().unwrap(), id);
            assert_eq!(serde_json::to_value(id).unwrap(), s);
        }
        assert!("yt-dlp".parse::<ToolId>().is_err());
    }

    /// 测试用假引擎
    struct StubEngine {
        id: ToolId,
    }

    impl DownloadEngine for StubEngine {
        fn id(&self) -> ToolId {
            self.id
        }
        fn handles(&self, url_type: UrlType) -> bool {
            match self.id {
                ToolId::Nm3u8dl => url_type.is_streaming(),
                ToolId::Ffmpeg => url_type.needs_ffmpeg(),
            }
        }
        fn build_download_args(
            &self,
            _spec: &TaskSpec,
            _tools: &ToolConfigs,
            _app: &AppSettings,
        ) -> Vec<String> {
            vec![]
        }
        fn build_parse_args(
            &self,
            _url: &str,
            _tools: &ToolConfigs,
            _app: &AppSettings,
        ) -> Vec<String> {
            vec![]
        }
        fn parse_streams(&self, _stdout: &str) -> StreamInfo {
            StreamInfo::default()
        }
        fn new_session(&self) -> Box<dyn EngineSession> {
            Box::new(StubSession)
        }
    }

    struct StubSession;

    impl EngineSession for StubSession {
        fn parse_line(&mut self, _line: &str) -> Option<EngineEvent> {
            None
        }
    }

    fn test_registry() -> EngineRegistry {
        let mut r = EngineRegistry::new();
        r.register(Box::new(StubEngine {
            id: ToolId::Nm3u8dl,
        }));
        r.register(Box::new(StubEngine { id: ToolId::Ffmpeg }));
        r
    }

    #[test]
    fn dispatch_by_url_type() {
        let r = test_registry();
        assert_eq!(r.for_url(UrlType::Hls).unwrap().id(), ToolId::Nm3u8dl);
        assert_eq!(r.for_url(UrlType::Dash).unwrap().id(), ToolId::Nm3u8dl);
        assert_eq!(r.for_url(UrlType::HttpVideo).unwrap().id(), ToolId::Ffmpeg);
        // Unknown 回退到首个注册引擎（Nm3u8dl，格式覆盖最广）
        assert_eq!(r.for_url(UrlType::Unknown).unwrap().id(), ToolId::Nm3u8dl);
    }

    #[test]
    fn get_by_tool_id() {
        let r = test_registry();
        assert_eq!(r.get(ToolId::Ffmpeg).unwrap().id(), ToolId::Ffmpeg);
        let empty = EngineRegistry::new();
        assert!(empty.get(ToolId::Ffmpeg).is_none());
        assert!(empty.for_url(UrlType::Hls).is_none());
    }

    #[test]
    fn engine_event_serializes_tagged() {
        let e = EngineEvent::Progress {
            data: ProgressData {
                percent: 55,
                ..Default::default()
            },
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["type"], "progress");
        assert_eq!(v["data"]["percent"], 55);

        let l = EngineEvent::Log {
            level: "INFO".into(),
            message: "hello".into(),
        };
        let v = serde_json::to_value(&l).unwrap();
        assert_eq!(v["type"], "log");
        assert_eq!(v["message"], "hello");
    }
}
