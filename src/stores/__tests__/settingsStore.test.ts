/** @vitest-environment happy-dom */
/**
 * settingsStore 行为测试
 *
 * 覆盖「应用自身配置项」的核心链路：loadSettings 三组并行加载 + 主题/语言副作用、
 * update* 调用 patch 服务并更新内存状态。
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useSettingsStore } from "../settingsStore";
import { settingsService } from "@/services";
import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_NM3U8DL_CONFIG,
  DEFAULT_FFMPEG_CONFIG,
} from "@/domain";

vi.mock("@/services", () => ({
  settingsService: {
    getAppSettings: vi.fn(),
    getNm3u8dlConfig: vi.fn(),
    getFfmpegConfig: vi.fn(),
    patchAppSettings: vi.fn(),
    patchNm3u8dlConfig: vi.fn(),
    patchFfmpegConfig: vi.fn(),
    exportConfig: vi.fn(),
    importConfig: vi.fn(),
  },
  systemService: {
    selectDirectory: vi.fn(),
    selectFile: vi.fn(),
  },
}));

vi.mock("@/locales", () => ({
  setLocale: vi.fn(),
  i18n: { global: { t: (k: string) => k } },
}));

describe("settingsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("loadSettings 并行加载三组配置并应用主题/语言副作用", async () => {
    vi.mocked(settingsService.getAppSettings).mockResolvedValue({
      ...DEFAULT_APP_SETTINGS,
      theme: "light",
      language: "en-US",
    });
    vi.mocked(settingsService.getNm3u8dlConfig).mockResolvedValue({
      ...DEFAULT_NM3U8DL_CONFIG,
    });
    vi.mocked(settingsService.getFfmpegConfig).mockResolvedValue({
      ...DEFAULT_FFMPEG_CONFIG,
    });

    const store = useSettingsStore();
    await store.loadSettings();

    expect(settingsService.getAppSettings).toHaveBeenCalledOnce();
    expect(settingsService.getNm3u8dlConfig).toHaveBeenCalledOnce();
    expect(settingsService.getFfmpegConfig).toHaveBeenCalledOnce();
    expect(store.appSettings.theme).toBe("light");
    expect(store.appSettings.language).toBe("en-US");
    // 主题副作用：light → 移除 dark class
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("updateAppSettings 调用 patch 并更新内存状态（minimize_to_tray）", async () => {
    vi.mocked(settingsService.patchAppSettings).mockResolvedValue({
      ...DEFAULT_APP_SETTINGS,
      minimize_to_tray: true,
    });

    const store = useSettingsStore();
    await store.updateAppSettings({ minimize_to_tray: true });

    expect(settingsService.patchAppSettings).toHaveBeenCalledWith({
      minimize_to_tray: true,
    });
    expect(store.appSettings.minimize_to_tray).toBe(true);
  });

  it("updateNm3u8dlConfig / updateFfmpegConfig 调用 patch 并更新", async () => {
    vi.mocked(settingsService.patchNm3u8dlConfig).mockResolvedValue({
      ...DEFAULT_NM3U8DL_CONFIG,
      thread_count: 16,
    });
    vi.mocked(settingsService.patchFfmpegConfig).mockResolvedValue({
      ...DEFAULT_FFMPEG_CONFIG,
      timeout: 90,
    });

    const store = useSettingsStore();
    await store.updateNm3u8dlConfig({ thread_count: 16 });
    await store.updateFfmpegConfig({ timeout: 90 });

    expect(store.nm3u8dlConfig.thread_count).toBe(16);
    expect(store.ffmpegConfig.timeout).toBe(90);
    // 并发数默认值锁定（后端契约一致）
    expect(DEFAULT_APP_SETTINGS.max_concurrent_tasks).toBe(5);
  });
});
