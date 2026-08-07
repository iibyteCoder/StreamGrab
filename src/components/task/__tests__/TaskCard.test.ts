/** @vitest-environment happy-dom */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import TaskCard from "../TaskCard.vue";
import TaskContextMenu from "../TaskContextMenu.vue";
import { i18n } from "@/locales";
import type { DownloadTask } from "@/domain";

const mocks = vi.hoisted(() => ({
  writeText: vi.fn(),
  toastSuccess: vi.fn(),
}));

vi.mock("@/services", () => ({
  systemService: {
    fileExists: vi.fn().mockResolvedValue(true),
    openInExplorer: vi.fn().mockResolvedValue(undefined),
    openFileInExplorer: vi.fn().mockResolvedValue(undefined),
    deleteFileOrFolder: vi.fn().mockResolvedValue(undefined),
  },
  clipboardService: {
    readText: vi.fn().mockResolvedValue(""),
    writeText: mocks.writeText,
    onFocus: vi.fn(),
  },
}));

vi.mock("@/composables", () => ({
  useTasks: () => ({ removeTask: vi.fn() }),
  useDownloader: () => ({
    startDownload: vi.fn(),
    stopDownload: vi.fn(),
    pauseDownload: vi.fn(),
    resumeDownload: vi.fn(),
  }),
  useToast: () => ({
    success: mocks.toastSuccess,
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
    remove: vi.fn(),
    clear: vi.fn(),
    toasts: [],
  }),
}));

vi.mock("@/stores", () => ({
  useTaskStore: () => ({
    getTaskLogs: () => [],
    retryTask: vi.fn(),
    getTaskById: () => undefined,
  }),
}));

function mkTask(overrides: Partial<DownloadTask> = {}): DownloadTask {
  return {
    id: "t1",
    url: "https://example.com/v.m3u8",
    fileName: "v.mp4",
    saveDir: "/downloads",
    status: "completed",
    wasInterrupted: false,
    createdAt: "2026-08-03T00:00:00Z",
    updatedAt: "2026-08-03T00:00:00Z",
    progress: {
      percent: 100,
      overallPercent: 100,
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

function mountCard(task: DownloadTask) {
  return mount(TaskCard, {
    props: { task },
    global: { plugins: [i18n] },
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.writeText.mockResolvedValue(undefined);
});

describe("TaskCard 右键菜单接线", () => {
  it("copyUrl → 写入 task.url + 成功 toast", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyUrl");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("https://example.com/v.m3u8");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFileName → 写入 task.fileName + 成功 toast", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFileName");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("v.mp4");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFilePath → 有 outputPath 时写入路径 + 成功 toast", async () => {
    const wrapper = mountCard(mkTask({ outputPath: "/downloads/v.mp4" }));
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFilePath");
    await flushPromises();
    expect(mocks.writeText).toHaveBeenCalledWith("/downloads/v.mp4");
    expect(mocks.toastSuccess).toHaveBeenCalledOnce();
  });

  it("copyFilePath → 无 outputPath 时静默跳过", async () => {
    const wrapper = mountCard(mkTask());
    wrapper.findComponent(TaskContextMenu).vm.$emit("copyFilePath");
    await flushPromises();
    expect(mocks.writeText).not.toHaveBeenCalled();
    expect(mocks.toastSuccess).not.toHaveBeenCalled();
  });

  it("redownload → emit('redownload', task)", async () => {
    const task = mkTask();
    const wrapper = mountCard(task);
    wrapper.findComponent(TaskContextMenu).vm.$emit("redownload");
    await nextTick();
    expect(wrapper.emitted("redownload")).toEqual([[task]]);
  });

  it("左键点击卡片 → 仍 emit('click', task)（回归）", async () => {
    const task = mkTask();
    const wrapper = mountCard(task);
    await wrapper.find(".task-card").trigger("click");
    expect(wrapper.emitted("click")?.[0]).toEqual([task]);
  });

  it("根节点为单一真实元素（TransitionGroup 列表动画前提，回归）", () => {
    const wrapper = mountCard(mkTask());
    const root = wrapper.element;
    expect(root.tagName).toBe("DIV");
    // 根节点是包裹 div（非卡片本体），卡片位于其内部——
    // 区分于 ContextMenu Fragment 根导致的「无法动画」运行时警告
    expect(root.classList.contains("task-card")).toBe(false);
    expect(root.querySelector(".task-card")).not.toBeNull();
  });
});

describe("TaskCard 状态呈现（单一来源）", () => {
  it("状态图标 aria-label = 对应状态文案（无障碍）", () => {
    const wrapper = mountCard(mkTask({ status: "downloading" }));
    const icon = wrapper.find('[role="img"]');
    expect(icon.exists()).toBe(true);
    expect(icon.attributes("aria-label")).toBe("下载中");
  });

  it("状态图标 aria-label 随状态变化", () => {
    const wrapper = mountCard(mkTask({ status: "failed" }));
    expect(wrapper.find('[role="img"]').attributes("aria-label")).toBe("失败");
  });

  it("暂停任务不再重复展示状态文案（去重回归）", () => {
    const wrapper = mountCard(
      mkTask({
        status: "paused",
        progress: { ...mkTask().progress, downloadedSize: 128 },
      }),
    );
    expect(wrapper.text()).not.toContain("已暂停");
  });

  it("等待中的定时任务展示定时时间（属数据而非状态）", () => {
    const wrapper = mountCard(
      mkTask({
        status: "pending",
        overrides: { scheduledStartAt: "2026-08-07T09:30" },
      }),
    );
    expect(wrapper.text()).toContain("定时");
  });
});
