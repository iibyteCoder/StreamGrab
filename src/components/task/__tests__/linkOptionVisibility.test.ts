import { describe, it, expect } from "vitest";
import { isOptionVisible } from "../linkOptionVisibility";
import type { UrlType } from "@/domain";

const T = (s: string) => s as UrlType;

describe("isOptionVisible", () => {
  it("通用选项对任意类型（含 null）可见", () => {
    expect(isOptionVisible("fileName", null)).toBe(true);
    expect(isOptionVisible("saveDir", T("httpVideo"))).toBe(true);
    expect(isOptionVisible("schedule", T("hls"))).toBe(true);
  });

  it("流媒体选项对直链/未知/null 不可见", () => {
    expect(isOptionVisible("maxSpeed", T("httpVideo"))).toBe(false);
    expect(isOptionVisible("muxFormat", T("unknown"))).toBe(false);
    expect(isOptionVisible("streamSelection", null)).toBe(false);
    expect(isOptionVisible("key", T("httpVideo"))).toBe(false);
  });

  it("流媒体选项对流媒体可见", () => {
    expect(isOptionVisible("maxSpeed", T("hls"))).toBe(true);
    expect(isOptionVisible("muxFormat", T("dash"))).toBe(true);
    expect(isOptionVisible("subtitlesOnly", T("mss"))).toBe(true);
    expect(isOptionVisible("customRange", T("hls"))).toBe(true);
    expect(isOptionVisible("subtitleFormat", T("dash"))).toBe(true);
    expect(isOptionVisible("streamSelection", T("mss"))).toBe(true);
    expect(isOptionVisible("key", T("hls"))).toBe(true);
  });
});
