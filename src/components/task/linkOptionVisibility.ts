import { isStreamingType } from "@/domain/url";
import type { UrlType } from "@/domain";
import type { LinkOption } from "./staging-types";

/** 仅流媒体（HLS/DASH/MSS）行可见的选项 */
const STREAMING_ONLY: ReadonlySet<LinkOption> = new Set<LinkOption>([
  "maxSpeed",
  "customRange",
  "muxFormat",
  "subtitleFormat",
  "subtitlesOnly",
  "streamSelection",
  "key",
]);

/**
 * 某选项在给定 URL 类型下是否可见。
 * 通用项（fileName/saveDir/schedule）始终可见；
 * 流媒体专属项仅当类型为 HLS/DASH/MSS 时可见。
 */
export function isOptionVisible(
  option: LinkOption,
  urlType: UrlType | null,
): boolean {
  if (STREAMING_ONLY.has(option)) {
    return urlType !== null && isStreamingType(urlType);
  }
  return true;
}
