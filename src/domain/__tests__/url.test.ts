/**
 * URL 类型检测测试
 *
 * 特征测试：与后端 `UrlType::detect`（domain/download/url_type.rs）行为一致
 */

import { describe, expect, it } from "vitest";
import { detectUrlType, isHttpUrl, isStreamingType, needsFfmpeg } from "../url";

describe("detectUrlType", () => {
  it("识别 HLS 链接", () => {
    expect(detectUrlType("https://example.com/video/index.m3u8")).toBe("hls");
    expect(detectUrlType("https://example.com/index.m3u8?token=abc")).toBe(
      "hls",
    );
    expect(detectUrlType("https://example.com/INDEX.M3U8")).toBe("hls");
  });

  it("识别 DASH 链接", () => {
    expect(detectUrlType("https://example.com/manifest.mpd")).toBe("dash");
    expect(detectUrlType("https://example.com/manifest.mpd?filter=1")).toBe(
      "dash",
    );
  });

  it("识别 MSS 链接（含查询参数变体）", () => {
    expect(detectUrlType("https://example.com/video.ism/manifest")).toBe("mss");
    expect(detectUrlType("https://example.com/video.ism/manifest?a=1")).toBe(
      "mss",
    );
    expect(detectUrlType("https://example.com/video.isml/manifest")).toBe(
      "mss",
    );
    expect(detectUrlType("https://example.com/video.isml/manifest?a=1")).toBe(
      "mss",
    );
  });

  it("识别直链视频/音频", () => {
    expect(detectUrlType("https://example.com/movie.mp4")).toBe("httpVideo");
    expect(detectUrlType("https://example.com/movie.MP4?sig=1")).toBe(
      "httpVideo",
    );
    expect(detectUrlType("https://example.com/clip.webm")).toBe("httpVideo");
    expect(detectUrlType("https://example.com/song.mp3")).toBe("httpVideo");
    expect(detectUrlType("https://example.com/seg.ts")).toBe("httpVideo");
  });

  it("未知类型返回 unknown", () => {
    expect(detectUrlType("https://example.com/page.html")).toBe("unknown");
    expect(detectUrlType("https://example.com/")).toBe("unknown");
    expect(detectUrlType("not a url")).toBe("unknown");
    expect(detectUrlType("")).toBe("unknown");
  });

  it("去除首尾空白后检测", () => {
    expect(detectUrlType("  https://example.com/a.m3u8  ")).toBe("hls");
  });
});

describe("类型谓词", () => {
  it("needsFfmpeg 仅直链为真", () => {
    expect(needsFfmpeg("httpVideo")).toBe(true);
    expect(needsFfmpeg("hls")).toBe(false);
    expect(needsFfmpeg("unknown")).toBe(false);
  });

  it("isStreamingType 覆盖三种流媒体", () => {
    expect(isStreamingType("hls")).toBe(true);
    expect(isStreamingType("dash")).toBe(true);
    expect(isStreamingType("mss")).toBe(true);
    expect(isStreamingType("httpVideo")).toBe(false);
  });
});

describe("isHttpUrl", () => {
  it("接受 http/https", () => {
    expect(isHttpUrl("http://example.com/a.m3u8")).toBe(true);
    expect(isHttpUrl("https://example.com/a.mp4")).toBe(true);
  });

  it("拒绝其他协议与非法输入", () => {
    expect(isHttpUrl("ftp://example.com/file")).toBe(false);
    expect(isHttpUrl("example.com/no-protocol")).toBe(false);
    expect(isHttpUrl("")).toBe(false);
  });
});
