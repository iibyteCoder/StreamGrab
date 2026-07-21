/**
 * 格式化工具测试
 */

import { describe, expect, it } from "vitest";
import {
  extractFileName,
  formatDurationHMS,
  formatFileSize,
  formatPercent,
  formatSpeed,
  sanitizeFileName,
  splitFilename,
} from "./format";

describe("formatFileSize", () => {
  it("零字节", () => {
    expect(formatFileSize(0)).toBe("0 B");
  });

  it("单位换算（1024 进制）", () => {
    expect(formatFileSize(1024)).toBe("1 KB");
    expect(formatFileSize(1024 * 1024)).toBe("1 MB");
    expect(formatFileSize(1024 * 1024 * 1024)).toBe("1 GB");
  });

  it("小数精度", () => {
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(1536, 0)).toBe("2 KB");
  });
});

describe("formatSpeed", () => {
  it("零速度", () => {
    expect(formatSpeed(0)).toBe("0 B/s");
  });

  it("带单位速度", () => {
    expect(formatSpeed(1024 * 1024)).toBe("1 MB/s");
  });
});

describe("formatDurationHMS", () => {
  it("不足一分钟 → M:SS", () => {
    expect(formatDurationHMS(45)).toBe("0:45");
  });

  it("分钟级 → M:SS", () => {
    expect(formatDurationHMS(90)).toBe("1:30");
    expect(formatDurationHMS(169)).toBe("2:49");
  });

  it("小时级 → H:MM:SS", () => {
    expect(formatDurationHMS(3661)).toBe("1:01:01");
    expect(formatDurationHMS(3600)).toBe("1:00:00");
  });

  it("非法输入兜底", () => {
    expect(formatDurationHMS(-5)).toBe("0:00");
    expect(formatDurationHMS(Number.NaN)).toBe("0:00");
  });
});

describe("formatPercent", () => {
  it("常规值与边界裁剪", () => {
    expect(formatPercent(42.345)).toBe("42.3%");
    expect(formatPercent(150)).toBe("100.0%");
    expect(formatPercent(-10)).toBe("0.0%");
    expect(formatPercent(Number.NaN)).toBe("0%");
  });
});

describe("文件名工具", () => {
  it("extractFileName 从 URL 提取无扩展名文件名", () => {
    expect(extractFileName("https://example.com/path/episode-01.mp4")).toBe(
      "episode-01",
    );
    expect(extractFileName("not-a-url")).toBe("video");
  });

  it("sanitizeFileName 替换非法字符", () => {
    expect(sanitizeFileName('a<b>c:d"e.mp4')).toBe("a_b_c_d_e.mp4");
    expect(sanitizeFileName("a b")).toBe("a_b");
  });

  it("splitFilename 分割名称与扩展名", () => {
    expect(splitFilename("video.mp4")).toEqual({
      stem: "video",
      ext: "mp4",
    });
    expect(splitFilename("noext")).toEqual({ stem: "noext", ext: "" });
    expect(splitFilename("a.b.c")).toEqual({ stem: "a.b", ext: "c" });
  });
});
