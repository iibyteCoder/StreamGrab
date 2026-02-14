/**
 * CLI 命令参数构建器
 * 将应用配置转换为 N_m3u8DL-RE 命令行参数
 */

import type { AppSettings, TaskConfig } from '@/types';

/**
 * 构建命令行参数
 * @param url 下载 URL
 * @param config 任务配置
 * @param settings 应用设置
 */
export function buildCommandArgs(
  url: string,
  config: TaskConfig,
  settings: AppSettings
): string[] {
  const args: string[] = [url];

  // 基础参数
  if (config.saveDir) {
    args.push('--save-dir', config.saveDir);
  }
  if (config.saveName) {
    args.push('--save-name', config.saveName);
  }

  // 临时目录
  if (settings.general.tmpDir) {
    args.push('--tmp-dir', settings.general.tmpDir);
  }

  // 下载参数
  if (config.threadCount) {
    args.push('--thread-count', String(config.threadCount));
  }
  if (config.retryCount) {
    args.push('--download-retry-count', String(config.retryCount));
  }
  if (config.timeout) {
    args.push('--http-request-timeout', String(config.timeout));
  }
  if (config.maxSpeed && config.maxSpeed !== '0') {
    args.push('-R', config.maxSpeed);
  }

  // 流选择
  if (config.autoSelect) {
    args.push('--auto-select');
  }
  if (config.selectVideo) {
    args.push('-sv', config.selectVideo);
  }
  if (config.selectAudio) {
    args.push('-sa', config.selectAudio);
  }
  if (config.selectSubtitle) {
    args.push('-ss', config.selectSubtitle);
  }

  // 流排除
  if (config.dropVideo) {
    args.push('-dv', config.dropVideo);
  }
  if (config.dropAudio) {
    args.push('-da', config.dropAudio);
  }
  if (config.dropSubtitle) {
    args.push('-ds', config.dropSubtitle);
  }

  // 命名模板
  if (config.savePattern?.enabled && config.savePattern.template) {
    args.push('--save-pattern', config.savePattern.template);
  }

  // 广告过滤
  if (settings.download.adFilter.enabled && settings.download.adFilter.keywords.length > 0) {
    args.push('--ad-keyword', settings.download.adFilter.keywords.join('|'));
  }

  // 混流设置
  if (config.muxFormat && config.muxAfterDone) {
    const muxOptions = buildMuxOptions(config.muxFormat, settings.mux);
    args.push('-M', muxOptions);
  }

  // 混流高级选项
  if (settings.mux.noDateInfo) {
    args.push('--no-date-info');
  }
  if (settings.mux.useConcatDemuxer) {
    args.push('--use-ffmpeg-concat-demuxer');
  }

  // 外部媒体导入
  if (settings.mux.muxImports.length > 0) {
    args.push(...buildMuxImportArgs(settings.mux.muxImports));
  }

  // 网络设置
  if (settings.network.useSystemProxy) {
    args.push('--use-system-proxy');
  } else if (settings.network.customProxy) {
    args.push('--custom-proxy', settings.network.customProxy);
  }

  // 请求头
  for (const header of settings.network.headers.filter((h) => h.enabled)) {
    args.push('-H', `${header.key}: ${header.value}`);
  }

  // BaseURL
  if (settings.network.baseUrl) {
    args.push('--base-url', settings.network.baseUrl);
  }
  if (settings.network.appendUrlParams) {
    args.push('--append-url-params');
  }

  // 其他选项
  if (config.skipMerge) {
    args.push('--skip-merge');
  }
  if (!config.delAfterDone) {
    args.push('--no-delete-temp');
  }
  if (!config.checkSegmentsCount) {
    args.push('--check-segments-count', 'false');
  }
  if (settings.download.binaryMerge) {
    args.push('--binary-merge');
  }
  if (settings.download.writeMetaJson) {
    args.push('--write-meta-json');
  }
  if (settings.download.concurrentDownload) {
    args.push('-mt');
  }
  if (settings.advanced.ffmpegPath) {
    args.push('--ffmpeg-binary-path', settings.advanced.ffmpegPath);
  }

  // 字幕设置
  if (settings.download.subOnly) {
    args.push('--sub-only');
  }
  if (settings.download.subFormat) {
    args.push('--sub-format', settings.download.subFormat);
  }
  if (settings.download.autoSubtitleFix) {
    args.push('--auto-subtitle-fix');
  }

  // 解密设置
  // 密钥数组优先，其次使用单个密钥
  if (settings.decryption.keys.length > 0) {
    args.push(...buildKeyArgs(settings.decryption.keys));
  } else if (config.key) {
    args.push('--key', config.key);
  }
  if (settings.decryption.keyTextFile) {
    args.push('--key-text-file', settings.decryption.keyTextFile);
  }
  if (settings.decryption.engine) {
    args.push('--decryption-engine', settings.decryption.engine);
  }
  if (settings.decryption.binPath) {
    args.push('--decryption-binary-path', settings.decryption.binPath);
  }
  if (settings.decryption.realTimeDecryption) {
    args.push('--mp4-real-time-decryption');
  }

  // 高级 HLS 解密
  if (settings.decryption.customHls.enabled) {
    if (settings.decryption.customHls.method !== 'UNKNOWN') {
      args.push('--custom-hls-method', settings.decryption.customHls.method);
    }
    if (settings.decryption.customHls.key.value) {
      const keyArg = buildKeyValue(settings.decryption.customHls.key);
      args.push('--custom-hls-key', keyArg);
    }
    if (settings.decryption.customHls.iv.value) {
      const ivArg = buildKeyValue(settings.decryption.customHls.iv);
      args.push('--custom-hls-iv', ivArg);
    }
  }

  // 直播设置
  if (settings.live.performAsVod) {
    args.push('--live-perform-as-vod');
  }
  if (settings.live.realTimeMerge) {
    args.push('--live-real-time-merge');
  }
  if (!settings.live.keepSegments) {
    args.push('--live-keep-segments', 'false');
  }
  if (settings.live.pipeMux) {
    args.push('--live-pipe-mux');
  }
  if (settings.live.fixVttByAudio) {
    args.push('--live-fix-vtt-by-audio');
  }
  if (settings.live.recordLimit) {
    args.push('--live-record-limit', settings.live.recordLimit);
  }
  if (settings.live.waitTime > 0) {
    args.push('--live-wait-time', String(settings.live.waitTime));
  }
  if (settings.live.takeCount !== 16) {
    args.push('--live-take-count', String(settings.live.takeCount));
  }

  // 范围下载
  if (config.customRange) {
    args.push('--custom-range', config.customRange);
  }

  // 定时开始
  if (config.startAt) {
    const dateStr = formatDateForCli(config.startAt);
    args.push('--task-start-at', dateStr);
  }

  // 高级设置
  if (settings.advanced.logLevel && settings.advanced.logLevel !== 'INFO') {
    args.push('--log-level', settings.advanced.logLevel);
  }
  if (settings.advanced.logFilePath) {
    args.push('--log-file-path', settings.advanced.logFilePath);
  }
  if (settings.advanced.noLog) {
    args.push('--no-log');
  }
  if (settings.advanced.allowHlsMultiExtMap) {
    args.push('--allow-hls-multi-ext-map');
  }
  if (settings.advanced.disableUpdateCheck) {
    args.push('--disable-update-check');
  }
  if (settings.advanced.urlProcessorArgs) {
    args.push('--urlprocessor-args', settings.advanced.urlProcessorArgs);
  }

  return args;
}

/**
 * 构建混流选项字符串
 */
function buildMuxOptions(
  format: string,
  muxSettings: AppSettings['mux']
): string {
  const parts: string[] = [`format=${format}`];

  parts.push(`muxer=${muxSettings.muxer}`);

  if (muxSettings.binPath) {
    parts.push(`bin_path="${muxSettings.binPath}"`);
  }

  if (muxSettings.skipSubtitles) {
    parts.push('skip_sub=true');
  }

  if (muxSettings.keepOriginal) {
    parts.push('keep=true');
  }

  return parts.join(':');
}

/**
 * 构建密钥/IV 值参数
 */
function buildKeyValue(kv: { type: string; value: string }): string {
  switch (kv.type) {
    case 'file':
      return kv.value;
    case 'hex':
      return kv.value;
    case 'base64':
      return kv.value;
    default:
      return kv.value;
  }
}

/**
 * 格式化日期为 CLI 格式 (yyyyMMddHHmmss)
 */
function formatDateForCli(date: Date | string): string {
  const d = new Date(date);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  const hours = String(d.getHours()).padStart(2, '0');
  const minutes = String(d.getMinutes()).padStart(2, '0');
  const seconds = String(d.getSeconds()).padStart(2, '0');
  return `${year}${month}${day}${hours}${minutes}${seconds}`;
}

/**
 * 构建外部媒体导入参数
 */
export function buildMuxImportArgs(
  imports: Array<{ path: string; lang?: string; name?: string }>
): string[] {
  const args: string[] = [];

  for (const imp of imports) {
    const parts: string[] = [`path="${imp.path}"`];

    if (imp.lang) {
      parts.push(`lang=${imp.lang}`);
    }
    if (imp.name) {
      parts.push(`name="${imp.name}"`);
    }

    args.push('--mux-import', parts.join(':'));
  }

  return args;
}

/**
 * 构建密钥参数
 */
export function buildKeyArgs(
  keys: Array<{ kid?: string; key: string }>
): string[] {
  const args: string[] = [];

  for (const k of keys) {
    if (k.kid) {
      args.push('--key', `${k.kid}:${k.key}`);
    } else {
      args.push('--key', k.key);
    }
  }

  return args;
}

/**
 * 构建完整的命令行字符串（用于日志/调试）
 */
export function buildCommandString(
  url: string,
  config: TaskConfig,
  settings: AppSettings
): string {
  const args = buildCommandArgs(url, config, settings);
  return ['N_m3u8DL-RE', ...args.map(escapeArg)].join(' ');
}

/**
 * 转义命令行参数
 */
function escapeArg(arg: string): string {
  if (arg.includes(' ') || arg.includes('"') || arg.includes("'")) {
    return `"${arg.replace(/"/g, '\\"')}"`;
  }
  return arg;
}
