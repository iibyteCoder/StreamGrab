import { describe, expect, it } from "vitest";
import { compareVersions } from "../version";

describe("compareVersions", () => {
  it("equal versions return 0", () => {
    expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
    expect(compareVersions("v2.1.3", "2.1.3")).toBe(0);
  });

  it("greater version returns 1", () => {
    expect(compareVersions("1.1.0", "1.0.0")).toBe(1);
    expect(compareVersions("2.0.0", "1.9.9")).toBe(1);
    expect(compareVersions("v0.6.0", "v0.5.2")).toBe(1);
  });

  it("lesser version returns -1", () => {
    expect(compareVersions("1.0.0", "1.1.0")).toBe(-1);
    expect(compareVersions("0.5.2", "0.6.0")).toBe(-1);
  });

  it("handles different segment counts", () => {
    expect(compareVersions("1.0", "1.0.0")).toBe(0);
    expect(compareVersions("1.0.1", "1.0")).toBe(1);
    expect(compareVersions("0.3.0.0", "0.3.0")).toBe(0);
  });

  it("handles non-numeric gracefully", () => {
    expect(compareVersions("latest", "latest")).toBe(0);
  });

  // ===== N_m3u8DL-RE 场景：GitHub tag vs 本地检测版本 =====

  it("prerelease tag equals installed build of the same version", () => {
    // tag v0.6.0-beta 构建出的二进制报告 0.6.0 → 视为已是最新
    expect(compareVersions("v0.6.0-beta", "0.6.0")).toBe(-1);
    expect(compareVersions("v0.6.0-beta", "0.6.0") > 0).toBe(false);
  });

  it("newer prerelease tag still counts as update", () => {
    expect(compareVersions("v0.6.1-beta", "0.6.0")).toBe(1);
    expect(compareVersions("v0.7.0-beta", "0.6.0")).toBe(1);
  });

  it("ignores build metadata hash suffix", () => {
    // --version 输出 "0.6.0+df70f0b3..."（若未经后端归一化直接到达前端）
    expect(compareVersions("0.6.0+df70f0b3da0c630b", "0.6.0")).toBe(0);
  });

  // ===== FFmpeg 场景：BtbN 滚动构建日期版本 =====

  it("compares date-based versions", () => {
    expect(compareVersions("2026-08-09", "2026-08-03")).toBe(1);
    expect(compareVersions("2026-08-03", "2026-08-09")).toBe(-1);
    expect(compareVersions("2026-08-09", "2026-08-09")).toBe(0);
  });

  it("handles latest- prefixed rolling versions", () => {
    expect(compareVersions("latest-2026-08-09", "2026-08-03")).toBe(1);
    expect(compareVersions("latest-2026-08-03", "2026-08-09")).toBe(-1);
    expect(compareVersions("latest-2026-08-09", "2026-08-09")).toBe(0);
  });
});
