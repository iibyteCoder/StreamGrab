/**
 * 验证工具函数
 */

import { URL_PATTERNS } from './constants';

/**
 * URL 验证结果
 */
export interface UrlValidationResult {
  valid: boolean;
  type: 'm3u8' | 'mpd' | 'mss' | 'unknown' | null;
  error?: string;
}

/**
 * 验证 URL 是否有效
 * @param url URL 字符串
 */
export function validateUrl(url: string): UrlValidationResult {
  if (!url || typeof url !== 'string') {
    return { valid: false, type: null, error: 'URL 不能为空' };
  }

  const trimmedUrl = url.trim();

  if (!trimmedUrl) {
    return { valid: false, type: null, error: 'URL 不能为空' };
  }

  // 检查是否是 HTTP/HTTPS 协议
  if (!URL_PATTERNS.http.test(trimmedUrl)) {
    return { valid: false, type: null, error: 'URL 必须以 http:// 或 https:// 开头' };
  }

  // 尝试解析 URL
  try {
    new URL(trimmedUrl);
  } catch {
    return { valid: false, type: null, error: 'URL 格式无效' };
  }

  // 检测流类型
  if (URL_PATTERNS.m3u8.test(trimmedUrl)) {
    return { valid: true, type: 'm3u8' };
  }

  if (URL_PATTERNS.mpd.test(trimmedUrl)) {
    return { valid: true, type: 'mpd' };
  }

  if (URL_PATTERNS.mss.test(trimmedUrl)) {
    return { valid: true, type: 'mss' };
  }

  // URL 有效但类型未知
  return { valid: true, type: 'unknown', error: '无法识别的流类型，可能不是 M3U8/MPD/MSS 链接' };
}

/**
 * 检查 URL 是否是 M3U8 链接
 * @param url URL 字符串
 */
export function isM3u8Url(url: string): boolean {
  return URL_PATTERNS.m3u8.test(url);
}

/**
 * 检查 URL 是否是 MPD 链接
 * @param url URL 字符串
 */
export function isMpdUrl(url: string): boolean {
  return URL_PATTERNS.mpd.test(url);
}

/**
 * 检查 URL 是否是 MSS 链接
 * @param url URL 字符串
 */
export function isMssUrl(url: string): boolean {
  return URL_PATTERNS.mss.test(url);
}

/**
 * 验证文件路径
 * @param path 文件路径
 */
export function validateFilePath(path: string): { valid: boolean; error?: string } {
  if (!path || typeof path !== 'string') {
    return { valid: false, error: '路径不能为空' };
  }

  const trimmedPath = path.trim();

  if (!trimmedPath) {
    return { valid: false, error: '路径不能为空' };
  }

  // Windows 路径检查
  if (/^[a-zA-Z]:/.test(trimmedPath)) {
    // 检查驱动器字母
    if (!/^[a-zA-Z]:[\\/]/.test(trimmedPath)) {
      return { valid: false, error: '无效的 Windows 路径格式' };
    }
    // 检查非法字符
    if (/[<>:"|?*]/.test(trimmedPath)) {
      return { valid: false, error: '路径包含非法字符' };
    }
  }

  // Unix 路径检查
  if (trimmedPath.startsWith('/')) {
    // 检查空路径段
    if (trimmedPath.includes('//')) {
      return { valid: false, error: '路径格式无效' };
    }
  }

  return { valid: true };
}

/**
 * 验证文件名
 * @param filename 文件名
 */
export function validateFileName(filename: string): { valid: boolean; error?: string } {
  if (!filename || typeof filename !== 'string') {
    return { valid: false, error: '文件名不能为空' };
  }

  const trimmedName = filename.trim();

  if (!trimmedName) {
    return { valid: false, error: '文件名不能为空' };
  }

  // 检查非法字符
  if (/[<>:"/\\|?*]/.test(trimmedName)) {
    return { valid: false, error: '文件名包含非法字符' };
  }

  // 检查保留名称 (Windows)
  const reservedNames = [
    'CON', 'PRN', 'AUX', 'NUL',
    'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
    'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
  ];

  const nameWithoutExt = trimmedName.split('.')[0].toUpperCase();
  if (reservedNames.includes(nameWithoutExt)) {
    return { valid: false, error: '文件名不能使用系统保留名称' };
  }

  // 检查长度
  if (trimmedName.length > 200) {
    return { valid: false, error: '文件名过长' };
  }

  return { valid: true };
}

/**
 * 验证端口号
 * @param port 端口号
 */
export function validatePort(port: number | string): { valid: boolean; error?: string } {
  const portNum = typeof port === 'string' ? parseInt(port, 10) : port;

  if (isNaN(portNum)) {
    return { valid: false, error: '端口号必须是数字' };
  }

  if (portNum < 1 || portNum > 65535) {
    return { valid: false, error: '端口号必须在 1-65535 之间' };
  }

  return { valid: true };
}

/**
 * 验证代理 URL
 * @param proxyUrl 代理 URL
 */
export function validateProxyUrl(proxyUrl: string): { valid: boolean; error?: string } {
  if (!proxyUrl || typeof proxyUrl !== 'string') {
    return { valid: false, error: '代理地址不能为空' };
  }

  const trimmed = proxyUrl.trim();

  if (!trimmed) {
    return { valid: false, error: '代理地址不能为空' };
  }

  // 支持的协议
  const supportedProtocols = ['http://', 'https://', 'socks4://', 'socks5://', 'socks://'];

  const hasProtocol = supportedProtocols.some((p) => trimmed.toLowerCase().startsWith(p));

  if (!hasProtocol) {
    return { valid: false, error: '代理地址必须以 http://, https://, socks4://, socks5:// 开头' };
  }

  try {
    const url = new URL(trimmed);

    if (!url.hostname) {
      return { valid: false, error: '代理地址缺少主机名' };
    }

    if (url.port) {
      const portResult = validatePort(url.port);
      if (!portResult.valid) {
        return portResult;
      }
    }

    return { valid: true };
  } catch {
    return { valid: false, error: '代理地址格式无效' };
  }
}

/**
 * 验证正则表达式
 * @param pattern 正则表达式字符串
 */
export function validateRegex(pattern: string): { valid: boolean; error?: string } {
  if (!pattern || typeof pattern !== 'string') {
    return { valid: false, error: '正则表达式不能为空' };
  }

  try {
    new RegExp(pattern);
    return { valid: true };
  } catch (e) {
    return { valid: false, error: `无效的正则表达式: ${e instanceof Error ? e.message : '未知错误'}` };
  }
}

/**
 * 验证十六进制密钥
 * @param key 十六进制字符串
 * @param expectedLength 期望长度（字节）
 */
export function validateHexKey(key: string, expectedLength = 16): { valid: boolean; error?: string } {
  if (!key || typeof key !== 'string') {
    return { valid: false, error: '密钥不能为空' };
  }

  const cleaned = key.replace(/\s/g, '');

  if (!/^[0-9a-fA-F]+$/.test(cleaned)) {
    return { valid: false, error: '密钥必须只包含十六进制字符 (0-9, a-f)' };
  }

  if (cleaned.length !== expectedLength * 2) {
    return { valid: false, error: `密钥长度必须为 ${expectedLength * 2} 个十六进制字符` };
  }

  return { valid: true };
}

/**
 * 验证 Base64 字符串
 * @param str Base64 字符串
 */
export function validateBase64(str: string): { valid: boolean; error?: string } {
  if (!str || typeof str !== 'string') {
    return { valid: false, error: '字符串不能为空' };
  }

  const cleaned = str.replace(/\s/g, '');

  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(cleaned)) {
    return { valid: false, error: '无效的 Base64 格式' };
  }

  try {
    atob(cleaned);
    return { valid: true };
  } catch {
    return { valid: false, error: '无效的 Base64 编码' };
  }
}

/**
 * 批量验证 URL
 * @param urls URL 列表
 */
export function validateUrls(urls: string[]): { valid: string[]; invalid: { url: string; error: string }[] } {
  const valid: string[] = [];
  const invalid: { url: string; error: string }[] = [];

  for (const url of urls) {
    const result = validateUrl(url);
    if (result.valid) {
      valid.push(url.trim());
    } else {
      invalid.push({ url, error: result.error || '无效的 URL' });
    }
  }

  return { valid, invalid };
}
