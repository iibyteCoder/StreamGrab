/**
 * 默认值和常量定义
 */

// 应用名称
export const APP_NAME = "StreamGrab";

// 任务状态颜色映射
export const TASK_STATUS_COLORS: Record<string, string> = {
  pending: "text-gray-400",
  analyzing: "text-blue-400",
  downloading: "text-blue-500",
  paused: "text-yellow-500",
  merging: "text-purple-400",
  muxing: "text-purple-500",
  completed: "text-green-500",
  failed: "text-red-500",
  cancelled: "text-gray-500",
};

// 任务状态背景色映射
export const TASK_STATUS_BG_COLORS: Record<string, string> = {
  pending: "bg-gray-500/20",
  analyzing: "bg-blue-500/20",
  downloading: "bg-blue-500/20",
  paused: "bg-yellow-500/20",
  merging: "bg-purple-500/20",
  muxing: "bg-purple-500/20",
  completed: "bg-green-500/20",
  failed: "bg-red-500/20",
  cancelled: "bg-gray-500/20",
};

// 任务状态文本
export const TASK_STATUS_TEXT: Record<string, string> = {
  pending: "等待中",
  analyzing: "解析中",
  downloading: "下载中",
  paused: "已暂停",
  merging: "合并中",
  muxing: "混流中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

// 任务状态配置 (组合文本和颜色)
export const TASK_STATUS_CONFIG: Record<
  string,
  { text: string; color: string }
> = {
  pending: { text: "等待中", color: "#9ca3af" },
  analyzing: { text: "解析中", color: "#60a5fa" },
  downloading: { text: "下载中", color: "#3b82f6" },
  paused: { text: "已暂停", color: "#eab308" },
  merging: { text: "合并中", color: "#a855f7" },
  muxing: { text: "混流中", color: "#a855f7" },
  completed: { text: "已完成", color: "#22c55e" },
  failed: { text: "失败", color: "#ef4444" },
  cancelled: { text: "已取消", color: "#6b7280" },
};

// 支持的流格式
export const SUPPORTED_FORMATS = {
  m3u8: [".m3u8", ".m3u"],
  mpd: [".mpd"],
  mss: [".ism/Manifest"],
};

// URL 正则表达式
export const URL_PATTERNS = {
  m3u8: /\.m3u8?($|\?)/i,
  mpd: /\.mpd($|\?)/i,
  mss: /\.ism\/Manifest($|\?)/i,
  http: /^https?:\/\//i,
};

// 默认文件名长度限制
export const MAX_FILENAME_LENGTH = 200;

// Toast 默认持续时间（毫秒）
export const TOAST_DEFAULT_DURATION = 3000;

// 配置文件名
export const CONFIG_FILE_NAME = "config.json";
export const HISTORY_FILE_NAME = "history.json";
export const TEMPLATES_FILE_NAME = "templates.json";
