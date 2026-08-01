import { describe, it, expect } from "vitest";
import { cleanOverrides, resolveLinkToTask } from "./resolveLinkToTask";
import type { StagedLink } from "./addTaskTypes";
import type { TaskOverrides } from "@/domain";

function mkLink(over: Partial<TaskOverrides> = {}, saveDir = ""): StagedLink {
  return {
    id: "1",
    url: "https://x/a.m3u8",
    detectedType: "hls",
    fileName: "a",
    saveDir,
    overrides: over as TaskOverrides,
    parseFailed: false,
  };
}

describe("cleanOverrides", () => {
  it("剔除空字段，全空返回 undefined", () => {
    expect(cleanOverrides({} as TaskOverrides)).toBeUndefined();
    expect(cleanOverrides({ maxSpeed: "" } as TaskOverrides)).toBeUndefined();
  });
  it("保留非空字段", () => {
    expect(cleanOverrides({ maxSpeed: "5M" } as TaskOverrides)?.maxSpeed).toBe(
      "5M",
    );
  });
});

describe("resolveLinkToTask（两层：逐条 > 默认）", () => {
  it("saveDir：行内非空优先，否则用默认目录", () => {
    expect(resolveLinkToTask(mkLink({}, ""), "D:/default").saveDir).toBe(
      "D:/default",
    );
    expect(resolveLinkToTask(mkLink({}, "D:/row"), "D:/default").saveDir).toBe(
      "D:/row",
    );
    expect(resolveLinkToTask(mkLink({}, "  "), "D:/default").saveDir).toBe(
      "D:/default",
    );
  });
  it("两者皆空 → saveDir undefined", () => {
    expect(resolveLinkToTask(mkLink({}, ""), "").saveDir).toBeUndefined();
  });
  it("空 overrides → undefined", () => {
    expect(resolveLinkToTask(mkLink(), "D:/default").overrides).toBeUndefined();
  });
  it("hasSchedule 由 scheduledStartAt 决定", () => {
    expect(
      resolveLinkToTask(
        mkLink({ scheduledStartAt: "2026-01-01T00:00:00" }),
        "D:/",
      ).hasSchedule,
    ).toBe(true);
    expect(resolveLinkToTask(mkLink(), "D:/").hasSchedule).toBe(false);
  });
  it("fileName 空白 → undefined", () => {
    expect(
      resolveLinkToTask({ ...mkLink(), fileName: "  " }, "D:/").fileName,
    ).toBeUndefined();
  });
});
