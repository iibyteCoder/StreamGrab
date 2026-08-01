import { detectUrlType, isStreamingType } from "@/domain/url";
import { extractFileName } from "@/utils/format";
import type { ParsedLink } from "./addTaskTypes";

/** 从粘贴文本提取合法链接行：trim、过滤 http(s)、按出现顺序去重 */
export function extractLinks(text: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of text.split("\n")) {
    const url = raw.trim();
    if (!(url.startsWith("http://") || url.startsWith("https://"))) continue;
    if (seen.has(url)) continue;
    seen.add(url);
    out.push(url);
  }
  return out;
}

/** 分类单条链接（同步，纯本地检测） */
export function classifyLink(url: string): ParsedLink {
  const detectedType = detectUrlType(url);
  return {
    url,
    detectedType,
    fileName: extractFileName(url),
    streaming: isStreamingType(detectedType),
  };
}

/** 解析粘贴文本：分类 + 剔除无法识别(unknown)，返回有效链接与跳过数 */
export function parsePastedText(text: string): {
  links: ParsedLink[];
  skipped: number;
} {
  const links: ParsedLink[] = [];
  let skipped = 0;
  for (const url of extractLinks(text)) {
    const link = classifyLink(url);
    if (link.detectedType === "unknown") {
      skipped++;
      continue;
    }
    links.push(link);
  }
  return { links, skipped };
}
