/** @vitest-environment happy-dom */
import { describe, it, expect, afterEach } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick, defineComponent } from "vue";
import type { PropType } from "vue";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import TaskContextMenu from "../TaskContextMenu.vue";
import { i18n, setLocale } from "@/locales";
import type { DownloadTask } from "@/domain";

// happy-dom 的 navigator.language 为 en-US，getDefaultLocale() 会选中 en-US；
// 显式锁定 zh-CN 以保证中文文案断言与环境无关。
setLocale("zh-CN");

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

const Host = defineComponent({
  components: { ContextMenu, ContextMenuTrigger, TaskContextMenu },
  props: {
    task: { type: Object as PropType<DownloadTask>, required: true },
  },
  emits: [
    "redownload",
    "copyUrl",
    "copyFileName",
    "copyFilePath",
    "openDetail",
  ],
  template: `
    <ContextMenu>
      <ContextMenuTrigger as-child>
        <div class="trigger">target</div>
      </ContextMenuTrigger>
      <TaskContextMenu
        :task="task"
        @redownload="$emit('redownload')"
        @copy-url="$emit('copyUrl')"
        @copy-file-name="$emit('copyFileName')"
        @copy-file-path="$emit('copyFilePath')"
        @open-detail="$emit('openDetail')"
      />
    </ContextMenu>
  `,
});

let wrapper: ReturnType<typeof mount> | null = null;

afterEach(() => {
  wrapper?.unmount();
  wrapper = null;
  document.body.innerHTML = "";
});

async function openMenu(task: DownloadTask): Promise<HTMLElement[]> {
  wrapper = mount(Host, {
    props: { task },
    global: { plugins: [i18n] },
    attachTo: document.body,
  });
  await wrapper.find(".trigger").trigger("contextmenu");
  await nextTick();
  await nextTick();
  return [...document.querySelectorAll('[role="menuitem"]')] as HTMLElement[];
}

describe("TaskContextMenu 渲染", () => {
  it("非 completed 状态 → 仅 4 个常驻项（测试环境默认 zh-CN）", async () => {
    const items = await openMenu(mkTask({ status: "pending" }));
    expect(items.map((i) => i.textContent?.trim())).toEqual([
      "以此链接重新下载",
      "复制下载链接",
      "复制文件名",
      "打开详情",
    ]);
  });

  it("completed + outputPath → 含复制文件路径（5 项）", async () => {
    const items = await openMenu(
      mkTask({ status: "completed", outputPath: "/downloads/v.mp4" }),
    );
    expect(items.map((i) => i.textContent?.trim())).toEqual([
      "以此链接重新下载",
      "复制下载链接",
      "复制文件名",
      "复制文件路径",
      "打开详情",
    ]);
  });

  it("completed 但无 outputPath → 无复制文件路径（4 项）", async () => {
    const items = await openMenu(mkTask({ status: "completed" }));
    expect(items).toHaveLength(4);
    expect(items.map((i) => i.textContent?.trim())).not.toContain(
      "复制文件路径",
    );
  });

  it("点击菜单项 → emit 对应事件", async () => {
    await openMenu(mkTask({ status: "pending" }));
    const items = [
      ...document.querySelectorAll('[role="menuitem"]'),
    ] as HTMLElement[];
    items[1]!.click(); // 复制下载链接
    await nextTick();
    expect(wrapper!.emitted("copyUrl")).toHaveLength(1);
  });
});
