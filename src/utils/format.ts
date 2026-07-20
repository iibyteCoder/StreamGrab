/**
 * 格式化工具函数
 */

/**
 * 格式化文件大小
 * @param bytes 字节数
 * @param decimals 小数位数
 */
export function formatFileSize(bytes: number, decimals = 2): string {
  if (bytes === 0) return "0 B";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const index = Math.min(i, sizes.length - 1);

  return `${parseFloat((bytes / Math.pow(k, index)).toFixed(dm))} ${sizes[index]}`;
}

/**
 * 格式化速度
 * @param bytesPerSecond 每秒字节数
 */
export function formatSpeed(bytesPerSecond: number): string {
  if (bytesPerSecond === 0) return "0 B/s";
  return `${formatFileSize(bytesPerSecond)}/s`;
}

/**
 * 格式化时间（秒转为 HH:MM:SS 或 MM:SS）
 * @param seconds 秒数
 * @param showHours 是否总是显示小时
 */
export function formatTime(seconds: number, showHours = false): string {
  if (!isFinite(seconds) || seconds < 0)
    return showHours ? "00:00:00" : "00:00";

  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  const parts = [
    mins.toString().padStart(2, "0"),
    secs.toString().padStart(2, "0"),
  ];

  if (hrs > 0 || showHours) {
    parts.unshift(hrs.toString().padStart(2, "0"));
  }

  return parts.join(":");
}

/**
 * 格式化持续时间（智能格式）
 * @param seconds 秒数
 */
export function formatDuration(seconds: number): string {
  if (!isFinite(seconds) || seconds < 0) return "未知";

  if (seconds < 60) {
    return `${Math.round(seconds)} 秒`;
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  const parts: string[] = [];

  if (hours > 0) {
    parts.push(`${hours} 小时`);
  }
  if (minutes > 0) {
    parts.push(`${minutes} 分`);
  }
  if (secs > 0 && hours === 0) {
    parts.push(`${secs} 秒`);
  }

  return parts.join(" ") || "0 秒";
}

/**
 * 格式化进度百分比
 * @param value 进度值 (0-100)
 * @param decimals 小数位数
 */
export function formatPercent(value: number, decimals = 1): string {
  if (!isFinite(value)) return "0%";
  const clamped = Math.max(0, Math.min(100, value));
  return `${clamped.toFixed(decimals)}%`;
}

/**
 * 格式化比特率
 * @param bps 比特率 (bits per second)
 */
export function formatBitrate(bps: number): string {
  if (bps === 0) return "0 bps";

  const k = 1000;
  const sizes = ["bps", "Kbps", "Mbps", "Gbps"];

  const i = Math.floor(Math.log(bps) / Math.log(k));
  const index = Math.min(i, sizes.length - 1);

  return `${parseFloat((bps / Math.pow(k, index)).toFixed(2))} ${sizes[index]}`;
}

/**
 * 格式化日期时间
 * @param date 日期对象或时间戳
 * @param format 格式类型
 */
export function formatDateTime(
  date: Date | string | number,
  format: "full" | "date" | "time" | "relative" = "full",
): string {
  const d = new Date(date);

  if (isNaN(d.getTime())) return "无效日期";

  if (format === "relative") {
    return formatRelativeTime(d);
  }

  const year = d.getFullYear();
  const month = (d.getMonth() + 1).toString().padStart(2, "0");
  const day = d.getDate().toString().padStart(2, "0");
  const hours = d.getHours().toString().padStart(2, "0");
  const minutes = d.getMinutes().toString().padStart(2, "0");
  const seconds = d.getSeconds().toString().padStart(2, "0");

  if (format === "date") {
    return `${year}-${month}-${day}`;
  }

  if (format === "time") {
    return `${hours}:${minutes}:${seconds}`;
  }

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

/**
 * 格式化相对时间
 * @param date 日期对象
 */
export function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) {
    return "刚刚";
  } else if (diffMin < 60) {
    return `${diffMin} 分钟前`;
  } else if (diffHour < 24) {
    return `${diffHour} 小时前`;
  } else if (diffDay < 7) {
    return `${diffDay} 天前`;
  } else {
    return formatDateTime(date, "date");
  }
}

/**
 * 格式化分辨率
 * @param width 宽度
 * @param height 高度
 */
export function formatResolution(width: number, height: number): string {
  if (height >= 2160) return "4K";
  if (height >= 1440) return "2K";
  if (height >= 1080) return "1080p";
  if (height >= 720) return "720p";
  if (height >= 480) return "480p";
  if (height >= 360) return "360p";
  return `${width}x${height}`;
}

/**
 * 格式化帧率
 * @param fps 帧率
 */
export function formatFrameRate(fps: number): string {
  if (fps >= 60) return "60fps";
  if (fps >= 30) return "30fps";
  if (fps >= 24) return "24fps";
  return `${fps}fps`;
}

/**
 * 格式化声道数
 * @param channels 声道数
 */
export function formatChannels(channels: number | string): string {
  const ch = typeof channels === "string" ? parseInt(channels, 10) : channels;

  switch (ch) {
    case 1:
      return "单声道";
    case 2:
      return "立体声";
    case 6:
      return "5.1";
    case 8:
      return "7.1";
    default:
      return `${ch} 声道`;
  }
}

/**
 * 截断字符串
 * @param str 原字符串
 * @param maxLength 最大长度
 * @param suffix 后缀
 */
export function truncate(
  str: string,
  maxLength: number,
  suffix = "...",
): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - suffix.length) + suffix;
}

/**
 * 从 URL 提取文件名
 * @param url URL
 */
export function extractFileName(url: string): string {
  try {
    const urlObj = new URL(url);
    const pathname = urlObj.pathname;
    const filename = pathname.split("/").pop() || "video";

    // 移除扩展名
    const nameWithoutExt = filename.replace(/\.[^.]+$/, "");

    return nameWithoutExt || "video";
  } catch {
    return "video";
  }
}

/**
 * 清理文件名（移除非法字符）
 * @param filename 文件名
 */
export function sanitizeFileName(filename: string): string {
  // 移除 Windows 不允许的字符
  return filename
    .replace(/[<>:"/\\|?*]/g, "_")
    .replace(/\s+/g, "_")
    .slice(0, 200);
}

/**
 * 分割文件名为名称和扩展名
 * @param filename 文件名
 */
export function splitFilename(filename: string): { stem: string; ext: string } {
  const lastDot = filename.lastIndexOf(".");
  if (lastDot <= 0) {
    return { stem: filename, ext: "" };
  }
  return {
    stem: filename.slice(0, lastDot),
    ext: filename.slice(lastDot + 1),
  };
}

/**
 * 生成带时间戳的唯一文件名
 * @param baseName 基础文件名（可含扩展名）
 * @returns 唯一文件名，格式：basename_YYYYMMDD_HHMMSS.ext
 */
export function generateTimestampedFilename(baseName: string): string {
  const { stem, ext } = splitFilename(baseName);

  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  const hours = String(now.getHours()).padStart(2, "0");
  const minutes = String(now.getMinutes()).padStart(2, "0");
  const seconds = String(now.getSeconds()).padStart(2, "0");
  const timestamp = `${year}${month}${day}_${hours}${minutes}${seconds}`;

  if (ext) {
    return `${stem}_${timestamp}.${ext}`;
  }
  return `${stem}_${timestamp}`;
}

// 别名导出，保持 API 一致性
export const formatBytes = formatFileSize;
export const formatDate = (date: Date | string | number) =>
  formatDateTime(date, "relative");
