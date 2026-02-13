/**
 * 默认值和常量定义
 */

import type { AppSettings, GeneralSettings, DownloadSettings, MuxSettings, NetworkSettings, LiveSettings, DecryptionSettings, AdvancedSettings, UISettings } from '@/types';

// 应用名称
export const APP_NAME = 'StreamGrab';

// 默认通用设置
export const DEFAULT_GENERAL_SETTINGS: GeneralSettings = {
  saveDir: 'C:\\Users\\ZYB33\\Downloads\\StreamGrab',
  tmpDir: 'C:\\Users\\ZYB33\\Downloads\\StreamGrab',
  language: 'zh-CN',
  autoStartDownload: true,
  minimizeToTray: false,
  checkUpdate: true,
};

// 默认下载设置
export const DEFAULT_DOWNLOAD_SETTINGS: DownloadSettings = {
  threadCount: 8,
  retryCount: 3,
  timeout: 100,
  maxSpeed: '0',
  autoSelect: true,
  selectVideo: '',
  selectAudio: '',
  selectSubtitle: '',
  dropVideo: '',
  dropAudio: '',
  dropSubtitle: '',
  savePattern: {
    enabled: false,
    template: '<SaveName>',
    presetId: 'basic',
  },
  adFilter: {
    enabled: false,
    keywords: [],
  },
  checkSegmentsCount: true,
  delAfterDone: true,
  skipMerge: false,
  writeMetaJson: false,
  binaryMerge: false,
  concurrentDownload: false,
  subOnly: false,
  subFormat: 'SRT',
  autoSubtitleFix: true,
};

// 默认混流设置
export const DEFAULT_MUX_SETTINGS: MuxSettings = {
  format: 'mp4',
  muxer: 'ffmpeg',
  binPath: '',
  keepOriginal: false,
  skipSubtitles: false,
  noDateInfo: false,
  useConcatDemuxer: false,
  muxImports: [],
};

// 默认网络设置
export const DEFAULT_NETWORK_SETTINGS: NetworkSettings = {
  useSystemProxy: true,
  customProxy: '',
  headers: [],
  baseUrl: '',
  appendUrlParams: false,
};

// 默认解密设置
export const DEFAULT_DECRYPTION_SETTINGS: DecryptionSettings = {
  keys: [],
  keyTextFile: '',
  engine: 'MP4DECRYPT',
  binPath: '',
  realTimeDecryption: false,
  customHls: {
    enabled: false,
    method: 'UNKNOWN',
    key: { type: 'hex', value: '' },
    iv: { type: 'hex', value: '' },
  },
};

// 默认直播设置
export const DEFAULT_LIVE_SETTINGS: LiveSettings = {
  performAsVod: false,
  realTimeMerge: false,
  keepSegments: true,
  pipeMux: false,
  fixVttByAudio: false,
  recordLimit: '',
  waitTime: 0,
  takeCount: 16,
};

// 默认高级设置
export const DEFAULT_ADVANCED_SETTINGS: AdvancedSettings = {
  ffmpegPath: '',
  n_m3u8dlPath: 'C:\\Users\\ZYB33\\Downloads\\my_projects\\StreamGrab\\tools\\N_m3u8DL-RE.exe',
  logLevel: 'INFO',
  logFilePath: '',
  noLog: false,
  allowHlsMultiExtMap: false,
  disableUpdateCheck: false,
  urlProcessorArgs: '',
};

// 默认 UI 设置
export const DEFAULT_UI_SETTINGS: UISettings = {
  theme: 'dark',
  showNotification: true,
  clipboardWatch: false,
};

// 完整默认设置
export const DEFAULT_SETTINGS: AppSettings = {
  general: DEFAULT_GENERAL_SETTINGS,
  download: DEFAULT_DOWNLOAD_SETTINGS,
  mux: DEFAULT_MUX_SETTINGS,
  network: DEFAULT_NETWORK_SETTINGS,
  live: DEFAULT_LIVE_SETTINGS,
  decryption: DEFAULT_DECRYPTION_SETTINGS,
  advanced: DEFAULT_ADVANCED_SETTINGS,
  ui: DEFAULT_UI_SETTINGS,
};

// 任务状态颜色映射
export const TASK_STATUS_COLORS: Record<string, string> = {
  pending: 'text-gray-400',
  analyzing: 'text-blue-400',
  downloading: 'text-blue-500',
  paused: 'text-yellow-500',
  merging: 'text-purple-400',
  muxing: 'text-purple-500',
  completed: 'text-green-500',
  failed: 'text-red-500',
  cancelled: 'text-gray-500',
};

// 任务状态背景色映射
export const TASK_STATUS_BG_COLORS: Record<string, string> = {
  pending: 'bg-gray-500/20',
  analyzing: 'bg-blue-500/20',
  downloading: 'bg-blue-500/20',
  paused: 'bg-yellow-500/20',
  merging: 'bg-purple-500/20',
  muxing: 'bg-purple-500/20',
  completed: 'bg-green-500/20',
  failed: 'bg-red-500/20',
  cancelled: 'bg-gray-500/20',
};

// 任务状态文本
export const TASK_STATUS_TEXT: Record<string, string> = {
  pending: '等待中',
  analyzing: '解析中',
  downloading: '下载中',
  paused: '已暂停',
  merging: '合并中',
  muxing: '混流中',
  completed: '已完成',
  failed: '失败',
  cancelled: '已取消',
};

// 任务状态配置 (组合文本和颜色)
export const TASK_STATUS_CONFIG: Record<string, { text: string; color: string }> = {
  pending: { text: '等待中', color: '#9ca3af' },
  analyzing: { text: '解析中', color: '#60a5fa' },
  downloading: { text: '下载中', color: '#3b82f6' },
  paused: { text: '已暂停', color: '#eab308' },
  merging: { text: '合并中', color: '#a855f7' },
  muxing: { text: '混流中', color: '#a855f7' },
  completed: { text: '已完成', color: '#22c55e' },
  failed: { text: '失败', color: '#ef4444' },
  cancelled: { text: '已取消', color: '#6b7280' },
};

// 支持的流格式
export const SUPPORTED_FORMATS = {
  m3u8: ['.m3u8', '.m3u'],
  mpd: ['.mpd'],
  mss: ['.ism/Manifest'],
};

// URL 正则表达式
export const URL_PATTERNS = {
  m3u8: /\.m3u8?($|\?)/i,
  mpd: /\.mpd($|\?)/i,
  mss: /\.ism\/Manifest($|\?)/i,
  http: /^https?:\/\//i,
};

// 最大并发任务数
export const MAX_CONCURRENT_TASKS = 5;

// 默认文件名长度限制
export const MAX_FILENAME_LENGTH = 200;

// 进度更新间隔（毫秒）
export const PROGRESS_UPDATE_INTERVAL = 500;

// Toast 默认持续时间（毫秒）
export const TOAST_DEFAULT_DURATION = 3000;

// 配置文件名
export const CONFIG_FILE_NAME = 'config.json';
export const HISTORY_FILE_NAME = 'history.json';
export const TEMPLATES_FILE_NAME = 'templates.json';
