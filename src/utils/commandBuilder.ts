/**
 * CLI 命令参数构建器
 * 将应用配置转换为 N_m3u8DL-RE 命令行参数
 */

import type {
  AllConfig,
  M3U8DLSettings,
  NetworkSettings,
  NetworkHeader,
  DecryptionSettings,
  DecryptionKey,
} from "@/domain/config";
import type { TaskConfig } from "@/types";

/**
 * 构建下载命令参数
 */
export function buildCommandArgs(
  url: string,
  config: TaskConfig,
  allConfig: AllConfig,
): string[] {
  const args: string[] = [url];
  const { app, m3u8dl, network, decryption, headers, keys } = allConfig;

  // === 基础参数 ===
  if (config.saveDir) {
    args.push("--save-dir", config.saveDir);
  }
  if (config.saveName) {
    args.push("--save-name", config.saveName);
  }
  const tmpDir = app.default_tmp_dir || config.saveDir;
  if (tmpDir) {
    args.push("--tmp-dir", tmpDir);
  }

  // === 下载参数 ===
  if (config.threadCount ?? m3u8dl.thread_count !== 8) {
    args.push(
      "--thread-count",
      String(config.threadCount ?? m3u8dl.thread_count),
    );
  }
  if (config.retryCount ?? m3u8dl.retry_count !== 3) {
    args.push(
      "--download-retry-count",
      String(config.retryCount ?? m3u8dl.retry_count),
    );
  }
  if (config.timeout ?? m3u8dl.timeout !== 100) {
    args.push(
      "--http-request-timeout",
      String(config.timeout ?? m3u8dl.timeout),
    );
  }
  const maxSpeed = config.maxSpeed ?? m3u8dl.max_speed;
  if (maxSpeed && maxSpeed !== "0") {
    args.push("-R", maxSpeed);
  }

  // === 流选择 ===
  if (config.autoSelect ?? m3u8dl.auto_select) {
    args.push("--auto-select");
  }
  const selectVideo = config.selectVideo ?? m3u8dl.select_video;
  if (selectVideo) {
    args.push("-sv", selectVideo);
  }
  const selectAudio = config.selectAudio ?? m3u8dl.select_audio;
  if (selectAudio) {
    args.push("-sa", selectAudio);
  }
  const selectSubtitle = config.selectSubtitle ?? m3u8dl.select_subtitle;
  if (selectSubtitle) {
    args.push("-ss", selectSubtitle);
  }

  // === 流排除 ===
  const dropVideo = config.dropVideo ?? m3u8dl.drop_video;
  if (dropVideo) {
    args.push("-dv", dropVideo);
  }
  const dropAudio = config.dropAudio ?? m3u8dl.drop_audio;
  if (dropAudio) {
    args.push("-da", dropAudio);
  }
  const dropSubtitle = config.dropSubtitle ?? m3u8dl.drop_subtitle;
  if (dropSubtitle) {
    args.push("-ds", dropSubtitle);
  }

  // === 混流设置 ===
  const muxFormat = config.muxFormat ?? m3u8dl.mux_format;
  if (muxFormat && config.muxAfterDone !== false) {
    args.push("-M", buildMuxOptions(muxFormat, m3u8dl));
  }
  if (m3u8dl.no_date_info) {
    args.push("--no-date-info");
  }
  if (m3u8dl.use_ffmpeg_concat_demuxer) {
    args.push("--use-ffmpeg-concat-demuxer");
  }

  // === 网络设置 ===
  addNetworkArgs(args, network, headers);

  // === 其他下载选项 ===
  if (config.skipMerge ?? m3u8dl.skip_merge) {
    args.push("--skip-merge");
  }
  if (config.delAfterDone ?? m3u8dl.del_after_done) {
    // 默认删除临时文件
  } else {
    args.push("--no-delete-temp");
  }
  if (config.checkSegmentsCount ?? m3u8dl.check_segments_count) {
    // 默认检查分片数量
  } else {
    args.push("--check-segments-count", "false");
  }
  if (m3u8dl.binary_merge) {
    args.push("--binary-merge");
  }
  if (m3u8dl.write_meta_json) {
    args.push("--write-meta-json");
  }
  if (m3u8dl.concurrent_download) {
    args.push("-mt");
  }

  // === 字幕设置 ===
  if (m3u8dl.sub_only) {
    args.push("--sub-only");
  }
  if (m3u8dl.sub_format) {
    args.push("--sub-format", m3u8dl.sub_format);
  }
  if (m3u8dl.auto_subtitle_fix) {
    args.push("--auto-subtitle-fix");
  }

  // === 解密设置 ===
  addDecryptionArgs(args, decryption, keys);

  // 任务级密钥
  if (keys.length === 0 && config.key) {
    args.push("--key", config.key);
  }

  // === 直播设置 ===
  if (m3u8dl.live_perform_as_vod) {
    args.push("--live-perform-as-vod");
  }
  if (m3u8dl.live_real_time_merge) {
    args.push("--live-real-time-merge");
  }
  if (!m3u8dl.live_keep_segments) {
    args.push("--live-keep-segments", "false");
  }
  if (m3u8dl.live_pipe_mux) {
    args.push("--live-pipe-mux");
  }
  if (m3u8dl.live_fix_vtt_by_audio) {
    args.push("--live-fix-vtt-by-audio");
  }
  if (m3u8dl.live_record_limit) {
    args.push("--live-record-limit", m3u8dl.live_record_limit);
  }
  if (m3u8dl.live_wait_time > 0) {
    args.push("--live-wait-time", String(m3u8dl.live_wait_time));
  }
  if (m3u8dl.live_take_count !== 16) {
    args.push("--live-take-count", String(m3u8dl.live_take_count));
  }

  // === 范围下载 ===
  if (config.customRange) {
    args.push("--custom-range", config.customRange);
  }

  // === 定时开始 ===
  if (config.startAt) {
    args.push("--task-start-at", formatDateForCli(config.startAt));
  }

  // === 日志设置 ===
  if (app.no_log) {
    args.push("--no-log");
  } else if (app.log_level && app.log_level !== "INFO") {
    args.push("--log-level", app.log_level);
  }

  // === 高级设置 ===
  if (app.log_file_path) {
    args.push("--log-file-path", app.log_file_path);
  }
  if (m3u8dl.allow_hls_multi_ext_map) {
    args.push("--allow-hls-multi-ext-map");
  }
  if (m3u8dl.url_processor_args) {
    args.push("--urlprocessor-args", m3u8dl.url_processor_args);
  }

  return args;
}

/**
 * 构建解析 URL 的命令参数
 */
export function buildParseArgs(
  url: string,
  allConfig: AllConfig,
  _parseId: string,
  _tempDir: string,
): string[] {
  const args: string[] = [url];

  args.push("--skip-download");
  args.push("--auto-select");

  addNetworkArgs(args, allConfig.network, allConfig.headers);
  addDecryptionArgs(args, allConfig.decryption, allConfig.keys, false);

  if (allConfig.app.no_log) {
    args.push("--no-log");
  } else if (allConfig.app.log_level && allConfig.app.log_level !== "INFO") {
    args.push("--log-level", allConfig.app.log_level);
  }

  return args;
}

// ============================================
// 辅助函数
// ============================================

/**
 * 添加网络相关参数
 */
function addNetworkArgs(
  args: string[],
  network: NetworkSettings,
  headers: NetworkHeader[],
): void {
  if (network.use_system_proxy) {
    args.push("--use-system-proxy");
  } else if (network.custom_proxy) {
    args.push("--custom-proxy", network.custom_proxy);
  }

  for (const header of headers.filter((h) => h.enabled)) {
    args.push("-H", `${header.name}: ${header.value}`);
  }

  if (network.base_url) {
    args.push("--base-url", network.base_url);
  }
  if (network.append_url_params) {
    args.push("--append-url-params");
  }
}

/**
 * 添加解密相关参数
 */
function addDecryptionArgs(
  args: string[],
  decryption: DecryptionSettings,
  keys: DecryptionKey[],
  includeRealTime: boolean = true,
): void {
  if (keys.length > 0) {
    args.push(...buildKeyArgs(keys));
  }
  if (decryption.key_text_file) {
    args.push("--key-text-file", decryption.key_text_file);
  }
  if (decryption.decryption_engine) {
    args.push("--decryption-engine", decryption.decryption_engine);
  }
  if (decryption.decryption_bin_path) {
    args.push("--decryption-binary-path", decryption.decryption_bin_path);
  }
  if (includeRealTime && decryption.real_time_decryption) {
    args.push("--mp4-real-time-decryption");
  }

  if (decryption.custom_hls_enabled) {
    if (decryption.custom_hls_method !== "UNKNOWN") {
      args.push("--custom-hls-method", decryption.custom_hls_method);
    }
    if (decryption.custom_hls_key_value) {
      args.push("--custom-hls-key", decryption.custom_hls_key_value);
    }
    if (decryption.custom_hls_iv_value) {
      args.push("--custom-hls-iv", decryption.custom_hls_iv_value);
    }
  }
}

/**
 * 构建混流选项字符串
 */
function buildMuxOptions(format: string, muxSettings: M3U8DLSettings): string {
  const parts: string[] = [`format=${format}`, `muxer=${muxSettings.muxer}`];

  if (muxSettings.mux_bin_path) {
    parts.push(`bin_path="${muxSettings.mux_bin_path}"`);
  }
  if (muxSettings.mux_skip_subtitles) {
    parts.push("skip_sub=true");
  }
  if (muxSettings.mux_keep_original) {
    parts.push("keep=true");
  }

  return parts.join(":");
}

/**
 * 构建密钥参数
 */
export function buildKeyArgs(keys: DecryptionKey[]): string[] {
  const args: string[] = [];

  for (const k of keys) {
    if (k.kid) {
      args.push("--key", `${k.kid}:${k.key}`);
    } else {
      args.push("--key", k.key);
    }
  }

  return args;
}

/**
 * 格式化日期为 CLI 格式
 */
function formatDateForCli(date: Date | string): string {
  const d = new Date(date);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hours = String(d.getHours()).padStart(2, "0");
  const minutes = String(d.getMinutes()).padStart(2, "0");
  const seconds = String(d.getSeconds()).padStart(2, "0");
  return `${year}${month}${day}${hours}${minutes}${seconds}`;
}

/**
 * 构建完整的命令行字符串（用于日志/调试）
 */
export function buildCommandString(
  url: string,
  config: TaskConfig,
  allConfig: AllConfig,
): string {
  const args = buildCommandArgs(url, config, allConfig);
  return ["N_m3u8DL-RE", ...args.map(escapeArg)].join(" ");
}

/**
 * 转义命令行参数
 */
function escapeArg(arg: string): string {
  if (arg.includes(" ") || arg.includes('"') || arg.includes("'")) {
    return `"${arg.replace(/"/g, '\\"')}"`;
  }
  return arg;
}
