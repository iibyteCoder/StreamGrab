/** @vitest-environment happy-dom */
/**
 * 系统通知行为测试
 *
 * 覆盖「show_notification 设置项」的生效性：false 直接短路、true 且已授权时
 * 走 Tauri notification 插件发送原生通知、未授权请求权限。
 *
 * 注意：useNotification 持有模块级权限缓存，每个用例用 vi.resetModules 重置，
 * 避免缓存跨用例泄漏导致断言顺序依赖。
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";

const mocks = vi.hoisted(() => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
  showNotification: { value: true },
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: mocks.isPermissionGranted,
  requestPermission: mocks.requestPermission,
  sendNotification: mocks.sendNotification,
}));

vi.mock("@/stores", () => ({
  useSettingsStore: () => ({
    appSettings: { show_notification: mocks.showNotification.value },
  }),
}));

/** 每个用例用全新模块（重置权限缓存）并返回 useNotification */
async function freshNotification() {
  vi.resetModules();
  const mod = await import("../useNotification");
  return mod.useNotification();
}

describe("useNotification", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mocks.showNotification.value = true;
    mocks.isPermissionGranted.mockResolvedValue(true);
    mocks.requestPermission.mockResolvedValue("granted");
    mocks.sendNotification.mockReturnValue(undefined);
  });

  it("show_notification=false 时直接短路不发送", async () => {
    mocks.showNotification.value = false;
    const { sendNotification } = await freshNotification();
    const ok = await sendNotification("标题", "内容");
    expect(ok).toBe(false);
    expect(mocks.sendNotification).not.toHaveBeenCalled();
  });

  it("show_notification=true 且已授权时发送系统通知", async () => {
    const { sendNotification } = await freshNotification();
    const ok = await sendNotification("下载完成", "文件已保存");
    expect(ok).toBe(true);
    expect(mocks.sendNotification).toHaveBeenCalledWith(
      expect.objectContaining({ title: "下载完成", body: "文件已保存" }),
    );
  });

  it("未授权则请求权限，拒绝后不发送", async () => {
    mocks.isPermissionGranted.mockResolvedValue(false);
    mocks.requestPermission.mockResolvedValue("denied");
    const { sendNotification } = await freshNotification();
    const ok = await sendNotification("标题", "内容");
    expect(ok).toBe(false);
    expect(mocks.requestPermission).toHaveBeenCalled();
    expect(mocks.sendNotification).not.toHaveBeenCalled();
  });
});
