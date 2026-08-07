/** @vitest-environment happy-dom */
/**
 * 启动自动检查更新行为测试
 *
 * 覆盖「check_update 设置项」的生效性：false 不触发网络检查、true 触发且受
 * 24h 节流保护（App.vue 启动时调用 autoCheckUpdateAtStartup）。
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { autoCheckUpdateAtStartup } from "../useUpdateChecker";

const mocks = vi.hoisted(() => ({
  fetchLatestVersion: vi.fn(),
  checkUpdate: { value: true },
}));

vi.mock("@/services", () => ({
  updateService: {
    fetchLatestVersion: mocks.fetchLatestVersion,
    getCurrentVersion: vi.fn(() => "0.6.1"),
    subscribeToProgress: vi.fn(async () => () => {}),
    isNewerThanCurrent: vi.fn(() => false),
    formatFileSize: vi.fn(),
    getPlatform: vi.fn(),
    downloadUpdate: vi.fn(),
    runInstaller: vi.fn(),
  },
}));

vi.mock("@/stores", () => ({
  useSettingsStore: () => ({
    appSettings: { check_update: mocks.checkUpdate.value },
  }),
}));

describe("autoCheckUpdateAtStartup", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    localStorage.clear();
    mocks.checkUpdate.value = true;
    mocks.fetchLatestVersion.mockResolvedValue(null);
  });

  it("check_update=false 时不触发网络检查", async () => {
    mocks.checkUpdate.value = false;
    await autoCheckUpdateAtStartup();
    expect(mocks.fetchLatestVersion).not.toHaveBeenCalled();
  });

  it("check_update=true 时触发检查", async () => {
    await autoCheckUpdateAtStartup();
    expect(mocks.fetchLatestVersion).toHaveBeenCalledOnce();
  });

  it("24h 节流：节流期内重复调用不重复请求", async () => {
    await autoCheckUpdateAtStartup();
    await autoCheckUpdateAtStartup();
    expect(mocks.fetchLatestVersion).toHaveBeenCalledOnce();
  });
});
