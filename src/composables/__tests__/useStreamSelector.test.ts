/** @vitest-environment happy-dom */
/**
 * 流选择器默认行为：视频默认选中最高画质（带宽优先，其次分辨率）
 */

import { describe, expect, it } from "vitest";
import { nextTick, ref } from "vue";
import { pickBestVideo, useStreamSelector } from "../useStreamSelector";
import type { StreamInfo, VideoStream } from "@/domain";

function makeVideo(
  overrides: Partial<VideoStream> & { id: string },
): VideoStream {
  return {
    bandwidth: 1000000,
    codecs: "avc1",
    language: "und",
    name: "",
    groupId: null,
    selected: null,
    resolution: "640x360",
    width: 640,
    height: 360,
    frameRate: 25,
    videoRange: "SDR",
    ...overrides,
  };
}

describe("pickBestVideo", () => {
  it("带宽最高者胜出，与列表顺序无关", () => {
    const low = makeVideo({ id: "low", bandwidth: 1_000_000 });
    const high = makeVideo({ id: "high", bandwidth: 8_000_000 });
    expect(pickBestVideo([low, high])?.id).toBe("high");
    expect(pickBestVideo([high, low])?.id).toBe("high");
  });

  it("带宽相同时取分辨率更高者", () => {
    const sd = makeVideo({
      id: "sd",
      bandwidth: 5_000_000,
      width: 1280,
      height: 720,
    });
    const hd = makeVideo({
      id: "hd",
      bandwidth: 5_000_000,
      width: 1920,
      height: 1080,
    });
    expect(pickBestVideo([sd, hd])?.id).toBe("hd");
  });

  it("空列表返回 undefined", () => {
    expect(pickBestVideo([])).toBeUndefined();
  });
});

describe("useStreamSelector 默认选择", () => {
  it("自动勾选最高画质视频流与默认音频流", async () => {
    const streamInfo = ref<StreamInfo | null>(null);
    const selector = useStreamSelector(streamInfo);

    streamInfo.value = {
      videos: [
        makeVideo({ id: "v-low", bandwidth: 1_000_000 }),
        makeVideo({ id: "v-high", bandwidth: 9_000_000 }),
      ],
      audios: [
        {
          id: "a-alt",
          bandwidth: 128000,
          codecs: "mp4a",
          language: "en",
          name: "",
          groupId: null,
          selected: null,
          channels: "2ch",
          sampleRate: 48000,
          isDefault: false,
        },
        {
          id: "a-default",
          bandwidth: 192000,
          codecs: "mp4a",
          language: "zh",
          name: "",
          groupId: null,
          selected: null,
          channels: "2ch",
          sampleRate: 48000,
          isDefault: true,
        },
      ],
      subtitles: [],
      duration: 3600,
      segmentCount: 120,
      isLive: false,
      isEncrypted: false,
    };
    await nextTick();

    expect(selector.selectedVideos.value).toEqual(new Set(["v-high"]));
    expect(selector.selectedAudios.value).toEqual(new Set(["a-default"]));
    expect(selector.selectedSubtitles.value.size).toBe(0);
  });
});
