/**
 * 通用类型和常量
 */

// ============================================
// 通用配置类型
// ============================================

// 请求头配置
export interface HeaderConfig {
  key: string;
  value: string;
  enabled: boolean;
}

// 密钥配置
export interface KeyConfig {
  kid?: string;
  key: string;
}

// 命名模板设置
export interface SavePatternSettings {
  enabled: boolean;
  template: string;
  presetId: string;
}

// 广告过滤设置
export interface AdFilterSettings {
  enabled: boolean;
  keywords: string[];
}

// 外部媒体导入
export interface MuxImport {
  path: string;
  lang?: string;
  name?: string;
}

// 高级 HLS 解密
export interface CustomHlsDecryption {
  enabled: boolean;
  method:
    | "AES_128"
    | "AES_128_ECB"
    | "CENC"
    | "CHACHA20"
    | "NONE"
    | "SAMPLE_AES"
    | "SAMPLE_AES_CTR"
    | "UNKNOWN";
  key: {
    type: "file" | "hex" | "base64";
    value: string;
  };
  iv: {
    type: "file" | "hex" | "base64";
    value: string;
  };
}

// ============================================
// 历史记录类型
// ============================================

export interface HistoryRecord {
  id: string;
  url: string;
  file_name: string;
  save_path: string;
  file_size: number;
  duration: number;
  completed_at: string;
  task_id?: string;
}

// ============================================
// 日志类型
// ============================================

export interface LogSettings {
  level: "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF";
  enableFileOutput: boolean;
  logFilePath: string;
  maxFileSize: number; // MB
  maxFileCount: number;
}

export interface LogEntry {
  timestamp: Date;
  level: "DEBUG" | "INFO" | "WARN" | "ERROR";
  message: string;
  source?: string;
}

// ============================================
// 默认值常量
// ============================================

export const DEFAULT_SAVE_PATTERN_PRESETS = [
  { id: "basic", name: "基础", template: "<SaveName>" },
  { id: "resolution", name: "包含分辨率", template: "<SaveName>_<Resolution>" },
  {
    id: "bandwidth",
    name: "包含带宽",
    template: "<SaveName>_<Resolution>_<Bandwidth>kbps",
  },
  {
    id: "multi-audio",
    name: "多音轨",
    template: "<SaveName>_<Language>_<Channels>ch",
  },
  {
    id: "full",
    name: "完整信息",
    template: "<MediaType>_<Resolution>_<Codecs>_<Language>",
  },
];

export const DEFAULT_AD_FILTER_PRESETS = [
  "ad\\.domain\\.com",
  "\\/ad\\/|\\/ads\\/|\\/advert",
  "doubleclick\\.net",
];

export const HLS_ENCRYPTION_METHODS = [
  { value: "AES_128", label: "AES-128 CBC" },
  { value: "AES_128_ECB", label: "AES-128 ECB" },
  { value: "CENC", label: "通用加密 (CENC)" },
  { value: "CHACHA20", label: "ChaCha20" },
  { value: "SAMPLE_AES", label: "采样 AES" },
  { value: "SAMPLE_AES_CTR", label: "采样 AES CTR" },
  { value: "NONE", label: "无加密" },
  { value: "UNKNOWN", label: "未知" },
] as const;
