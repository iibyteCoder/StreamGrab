/** @vitest-environment happy-dom */
/**
 * GeneralTab 行为测试
 *
 * 覆盖「最小化到托盘」开关的用户痛点链路：切换开关 → settingsStore.updateAppSettings
 * 收到正确 payload（{ minimize_to_tray: true }）。
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { nextTick } from "vue";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import { i18n } from "@/locales";
import GeneralTab from "../GeneralTab.vue";
import SettingSwitch from "../../SettingSwitch.vue";

const storeMocks = vi.hoisted(() => ({
  updateAppSettings: vi.fn(),
  appSettings: {
    language: "zh-CN",
    theme: "dark",
    default_save_dir: "",
    default_tmp_dir: "",
    show_notification: true,
    clipboard_watch: false,
    minimize_to_tray: false,
    check_update: true,
    auto_start_download: true,
    max_concurrent_tasks: 5,
    log_level: "INFO",
    log_file_path: "",
    no_log: false,
  },
}));

vi.mock("@/composables", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
    remove: vi.fn(),
    clear: vi.fn(),
    toasts: [],
  }),
  useUpdateChecker: () => ({
    isChecking: false,
    updateAvailable: false,
    currentVersion: "0.6.1",
    latestVersion: null,
    releaseNotes: null,
    selectedAsset: null,
    downloadStatus: "idle",
    downloadProgress: 0,
    downloadedSize: 0,
    totalSize: 0,
    downloadedFilePath: null,
    checkForUpdate: vi.fn(),
    downloadUpdate: vi.fn(),
    cancelDownload: vi.fn(),
    openDownloadPage: vi.fn(),
    openDownloadLocation: vi.fn(),
    runInstallerAgain: vi.fn(),
    formatFileSize: vi.fn(),
    getPlatform: vi.fn(),
  }),
}));

vi.mock("@/services", () => ({
  settingsService: { patchAppSettings: vi.fn() },
  systemService: { selectFile: vi.fn() },
}));

vi.mock("@/stores", () => ({
  useSettingsStore: () => ({
    appSettings: storeMocks.appSettings,
    updateAppSettings: storeMocks.updateAppSettings,
  }),
}));

describe("GeneralTab 最小化到托盘开关", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("切换开关 → updateAppSettings 收到 { minimize_to_tray: true }", async () => {
    i18n.global.locale.value = "zh-CN";
    const wrapper = mount(GeneralTab, {
      global: { plugins: [i18n] },
      attachTo: document.body,
    });
    await nextTick();

    // 找到「最小化到托盘」开关（label 来自 i18n zh-CN 默认）
    const switches = wrapper.findAllComponents(SettingSwitch);
    const minimize = switches.find((s) => s.props("label") === "最小化到托盘");
    expect(minimize, "应存在最小化到托盘开关").toBeDefined();

    await minimize!.vm.$emit("update:modelValue", true);
    await nextTick();

    expect(storeMocks.updateAppSettings).toHaveBeenCalledWith({
      minimize_to_tray: true,
    });
  });
});
