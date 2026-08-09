import { assert, assertEqual, test } from "../runner-lib.mjs";

function makeTask(overrides) {
  return {
    id: "t-" + Math.random().toString(36).slice(2, 8),
    url: "https://example.com/x.m3u8",
    fileName: "",
    saveDir: "",
    status: "pending",
    wasInterrupted: false,
    createdAt: "2026-08-01T08:00:00.000Z",
    updatedAt: "2026-08-01T08:00:00.000Z",
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
    overrides: null,
    ...overrides,
  };
}

export { makeTask };

test("启动：加载后端数据并渲染首页空状态与页脚", async (d) => {
  await d.resetAndGo(null);

  await d.assertText("没有下载任务");
  await d.assertText("输入链接开始下载");
  await d.assertText(/v0\.6\.1/);
  await d.assertText("by iibyteCoder");

  const calls = await d.mockCallsOf("load_all_tasks");
  assert(calls.length >= 1, "启动时应调用 load_all_tasks");
});

test("标题栏窗口控制按钮触发 Tauri 命令", async (d) => {
  await d.resetAndGo(null);
  await d.assertText("没有下载任务");
  await d.mockClearCalls();

  await d.clickTitle("最小化");
  await d.clickTitle("最大化");
  await d.clickTitle("关闭");

  const calls = await d.mockCalls();
  assert(calls.some((c) => c.command === "plugin:window|minimize"), "应调用 minimize");
  assert(
    calls.some((c) => c.command === "plugin:window|toggle_maximize"),
    "应调用 toggle_maximize",
  );
  assert(calls.some((c) => c.command === "plugin:window|close"), "应调用 close");
});

test("标题栏主题按钮切换深色/浅色并持久化", async (d) => {
  await d.resetAndGo(null);
  await d.assertText("没有下载任务");

  const darkBefore = await d.eval(
    `() => document.documentElement.classList.contains('dark')`,
  );
  assert(darkBefore === true, "默认主题应为深色");

  await d.clickTitle("切换主题");
  await d.assertEval(
    `() => !document.documentElement.classList.contains('dark')`,
  );

  const state = await d.mockState();
  assertEqual(state.appSettings.theme, "light");
});

test("托盘创建失败时显示警告条", async (d) => {
  await d.resetAndGo({
    trayStatus: { created: false, error: "mock-tray-error" },
  });

  await d.assertText("系统托盘创建失败");
  await d.assertText("mock-tray-error");
});

test("启动检测到中断任务：弹窗恢复并逐个 resume", async (d) => {
  await d.resetAndGo({
    tasks: [
      makeTask({
        id: "t1",
        url: "https://example.com/a.m3u8",
        fileName: "中断A.m3u8",
        status: "paused",
        progress: { percent: 32, overallPercent: 32 },
      }),
      makeTask({
        id: "t2",
        url: "https://example.com/b.m3u8",
        fileName: "中断B.m3u8",
        status: "downloading",
        progress: { percent: 55, overallPercent: 55 },
      }),
      makeTask({
        id: "t3",
        url: "https://example.com/c.mp4",
        fileName: "已完成C.mp4",
        status: "completed",
      }),
    ],
  });

  await d.assertText("恢复中断的下载");
  await d.assertText("检测到 2 个未完成的下载任务");
  await d.assertText("中断A.m3u8");
  await d.assertText("中断B.m3u8");

  await d.mockClearCalls();
  await d.clickText("全部恢复");
  await d.assertNoText("恢复中断的下载");

  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().filter(c => c.command === "start_download").length === 2`,
  );
  const resumeCalls = await d.mockCallsOf("start_download");
  assertEqual(
    resumeCalls.map((c) => c.args.taskId).sort(),
    ["t1", "t2"],
  );
});

test("从首页导航到设置页并可返回", async (d) => {
  await d.resetAndGo(null);
  await d.assertText("没有下载任务");

  await d.clickTitle("设置");
  await d.assertText("语言与外观");
  await d.assertText("语言与外观");

  await d.clickTitle("返回");
  await d.assertText("没有下载任务");
});
