/** @vitest-environment happy-dom */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { shallowMount, flushPromises } from "@vue/test-utils";
import AddTaskDialog from "./AddTaskDialog.vue";

const mocks = vi.hoisted(() => ({
  submitPaste: vi.fn(),
  reset: vi.fn(),
}));

vi.mock("@/composables", async () => {
  const { ref } = await import("vue");
  return {
    useAddTaskWizard: () => ({
      step: ref("paste"),
      current: ref(null),
      index: ref(0),
      total: ref(0),
      isSingle: ref(true),
      isLast: ref(true),
      showAddAll: ref(false),
      isSubmitting: ref(false),
      parseDone: ref(0),
      parseTotal: ref(0),
      parsingId: ref(null),
      dirs: ref([]),
      defaultDir: ref(""),
      showDuplicate: ref(false),
      duplicateTask: ref(null),
      reset: mocks.reset,
      submitPaste: mocks.submitPaste,
      retryParse: vi.fn(),
      browseSaveDir: vi.fn(),
      addCurrent: vi.fn(),
      skip: vi.fn(),
      addAll: vi.fn(),
      confirmDuplicate: vi.fn(),
      cancelDuplicate: vi.fn(),
    }),
  };
});

describe("AddTaskDialog initialUrl 预填", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("带 initialUrl 打开 → 自动调用 submitPaste(initialUrl)", async () => {
    const wrapper = shallowMount(AddTaskDialog, {
      props: { open: false, initialUrl: "https://example.com/x.m3u8" },
    });
    await wrapper.setProps({ open: true });
    await flushPromises();
    expect(mocks.reset).toHaveBeenCalledOnce();
    expect(mocks.submitPaste).toHaveBeenCalledWith(
      "https://example.com/x.m3u8",
    );
  });

  it("不带 initialUrl 打开 → 不调用 submitPaste（回归原流程）", async () => {
    const wrapper = shallowMount(AddTaskDialog, {
      props: { open: false },
    });
    await wrapper.setProps({ open: true });
    await flushPromises();
    expect(mocks.reset).toHaveBeenCalledOnce();
    expect(mocks.submitPaste).not.toHaveBeenCalled();
  });
});
