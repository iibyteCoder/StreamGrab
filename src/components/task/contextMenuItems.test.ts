import { describe, it, expect } from "vitest";
import { buildContextMenuItems } from "./contextMenuItems";
import type { DownloadTask } from "@/domain";

function mkTask(overrides: Partial<DownloadTask> = {}): DownloadTask {
  return {
    id: "t1",
    url: "https://example.com/v.m3u8",
    fileName: "v.mp4",
    saveDir: "/downloads",
    status: "pending",
    wasInterrupted: false,
    createdAt: "2026-08-03T00:00:00Z",
    updatedAt: "2026-08-03T00:00:00Z",
    progress: {
      percent: 0,
      overallPercent: 0,
      speed: 0,
      downloadedSize: 0,
      totalSize: 0,
      downloadedSegments: 0,
      totalSegments: 0,
      eta: 0,
      currentAction: "",
    },
    ...overrides,
  };
}

const ALL_STATUSES: DownloadTask["status"][] = [
  "pending",
  "analyzing",
  "downloading",
  "merging",
  "muxing",
  "paused",
  "completed",
  "failed",
  "cancelled",
];

describe("buildContextMenuItems", () => {
  it("四个常驻项在任何状态下均存在", () => {
    for (const status of ALL_STATUSES) {
      const keys = buildContextMenuItems(mkTask({ status })).map((i) => i.key);
      expect(keys).toContain("redownload");
      expect(keys).toContain("copyUrl");
      expect(keys).toContain("copyFileName");
      expect(keys).toContain("openDetail");
    }
  });

  it("复制文件路径仅在 completed + outputPath 时出现", () => {
    for (const status of ALL_STATUSES) {
      const withPath = buildContextMenuItems(
        mkTask({ status, outputPath: "/downloads/v.mp4" }),
      ).map((i) => i.key);
      const noPath = buildContextMenuItems(mkTask({ status })).map(
        (i) => i.key,
      );
      if (status === "completed") {
        expect(withPath).toContain("copyFilePath");
        expect(noPath).not.toContain("copyFilePath");
      } else {
        expect(withPath).not.toContain("copyFilePath");
      }
    }
  });

  it("顺序与分隔线：重新下载居首且其后有分隔线，最后一个复制项后有分隔线，打开详情居末", () => {
    const items = buildContextMenuItems(
      mkTask({ status: "completed", outputPath: "/downloads/v.mp4" }),
    );
    expect(items.map((i) => i.key)).toEqual([
      "redownload",
      "copyUrl",
      "copyFileName",
      "copyFilePath",
      "openDetail",
    ]);
    expect(items[0]?.separatorAfter).toBe(true);
    expect(items[3]?.separatorAfter).toBe(true);
    expect(items[4]?.separatorAfter).toBeFalsy();
  });
});
