import { describe, it, expect } from "vitest";
import {
  cleanOverrides,
  resolveLinkToTask,
  seedPresetOverrides,
} from "./resolveLinkToTask";
import type { BatchDefaults, StagedLink } from "./staging-types";
import type { TaskOverrides, UrlType } from "@/domain";

const T = (s: string) => s as UrlType;

function mkLink(over: Partial<TaskOverrides> = {}): StagedLink {
  return {
    id: "1",
    url: "https://x/a.m3u8",
    detectedType: T("hls"),
    fileName: "a",
    saveDir: "",
    overrides: over as TaskOverrides,
    status: "pending",
  };
}
const BATCH: BatchDefaults = { saveDir: "D:/batch", autoStart: true };

describe("cleanOverrides", () => {
  it("剔除空字段，全空返回 undefined", () => {
    expect(cleanOverrides({} as TaskOverrides)).toBeUndefined();
    expect(cleanOverrides({ maxSpeed: "" } as TaskOverrides)).toBeUndefined();
  });
  it("保留非空字段", () => {
    const o = cleanOverrides({ maxSpeed: "5M" } as TaskOverrides);
    expect(o?.maxSpeed).toBe("5M");
  });
});

describe("resolveLinkToTask", () => {
  it("saveDir 继承顺序：行 > 批次 > 全局", () => {
    expect(resolveLinkToTask(mkLink(), BATCH, "D:/global").saveDir).toBe(
      "D:/batch",
    );
    expect(
      resolveLinkToTask({ ...mkLink(), saveDir: "D:/row" }, BATCH, "D:/global")
        .saveDir,
    ).toBe("D:/row");
    expect(
      resolveLinkToTask(mkLink(), { saveDir: "", autoStart: true }, "D:/global")
        .saveDir,
    ).toBe("D:/global");
  });
  it("空 overrides → undefined", () => {
    expect(
      resolveLinkToTask(mkLink(), BATCH, "D:/global").overrides,
    ).toBeUndefined();
  });
  it("hasSchedule 由 scheduledStartAt 决定", () => {
    expect(
      resolveLinkToTask(
        mkLink({ scheduledStartAt: "2026-01-01T00:00:00" }),
        BATCH,
        "D:/global",
      ).hasSchedule,
    ).toBe(true);
    expect(resolveLinkToTask(mkLink(), BATCH, "D:/global").hasSchedule).toBe(
      false,
    );
  });
  it("fileName 空时 undefined", () => {
    expect(
      resolveLinkToTask({ ...mkLink(), fileName: "  " }, BATCH, "D:/global")
        .fileName,
    ).toBeUndefined();
  });
});

describe("seedPresetOverrides", () => {
  const preset = {
    maxSpeed: "5M",
    selection: { video: "res:1080" },
  } as TaskOverrides;
  it("流媒体行接受预设初值", () => {
    expect(seedPresetOverrides(preset, T("hls"))).toEqual({
      maxSpeed: "5M",
      selection: { video: "res:1080" },
    });
  });
  it("直链/未知/null 返回空对象", () => {
    expect(seedPresetOverrides(preset, T("httpVideo"))).toEqual({});
    expect(seedPresetOverrides(preset, T("unknown"))).toEqual({});
    expect(seedPresetOverrides(preset, null)).toEqual({});
  });
  it("selection 不共享引用（拷贝）", () => {
    const a = seedPresetOverrides(preset, T("hls"));
    const b = seedPresetOverrides(preset, T("hls"));
    expect(a.selection).not.toBe(b.selection);
    expect(a.selection).toEqual(b.selection);
  });
  it("null 预设返回空对象", () => {
    expect(seedPresetOverrides(null, T("hls"))).toEqual({});
  });
});
