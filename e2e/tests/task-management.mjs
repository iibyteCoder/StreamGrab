import { assertEqual, test } from "../runner-lib.mjs";
import { makeTask } from "./app-shell.mjs";

const ACTIVE_TASKS = [
  makeTask({
    id: "t-old",
    url: "https://example.com/old.m3u8",
    fileName: "电影A.mp4",
    status: "pending",
    createdAt: "2026-07-01T00:00:00.000Z",
    updatedAt: "2026-07-01T00:00:00.000Z",
  }),
  makeTask({
    id: "t-new",
    url: "https://example.com/new.m3u8",
    fileName: "纪录片B.mp4",
    status: "downloading",
    progress: { percent: 12, overallPercent: 12 },
    createdAt: "2026-08-01T00:00:00.000Z",
    updatedAt: "2026-08-01T00:00:00.000Z",
  }),
  makeTask({
    id: "t-fail",
    url: "https://example.com/fail.m3u8",
    fileName: "失败D.mp4",
    status: "failed",
    error: "HTTP 403 Forbidden",
    createdAt: "2026-07-15T00:00:00.000Z",
    updatedAt: "2026-07-15T00:00:00.000Z",
  }),
];

const COMPLETED_TASKS = [
  makeTask({
    id: "t-done",
    url: "https://example.com/concert.m3u8",
    fileName: "演唱会C.mp4",
    status: "completed",
    saveDir: "C:\\Downloads\\StreamGrab",
    outputPath: "C:\\Downloads\\StreamGrab\\演唱会C.mp4",
    completedAt: "2026-08-02T00:00:00.000Z",
    progress: { percent: 100, overallPercent: 100, totalSize: 734003200 },
    mediaInfo: {
      resolution: "1920x1080",
      duration: 3725,
      videoCodec: "h264",
      fileFormat: "mp4",
      fileSize: 734003200,
    },
    createdAt: "2026-07-20T00:00:00.000Z",
    updatedAt: "2026-08-02T00:00:00.000Z",
  }),
  makeTask({
    id: "t-missing",
    url: "https://example.com/oldmovie.mp4",
    fileName: "旧片E.mp4",
    status: "completed",
    outputPath: "C:\\Missing\\old.mp4",
    completedAt: "2026-07-01T00:00:00.000Z",
    createdAt: "2026-06-20T00:00:00.000Z",
    updatedAt: "2026-07-01T00:00:00.000Z",
  }),
];

test("Tab 分类与数量徽章", async (d) => {
  await d.resetAndGo({ tasks: [...ACTIVE_TASKS, ...COMPLETED_TASKS] });

  await d.assertText("电影A.mp4");
  await d.assertText("纪录片B.mp4");
  await d.assertText("失败D.mp4");
  await d.assertEval(
    `() => Array.from(document.querySelectorAll('button')).some(b => (b.textContent||'').includes('进行中') && (b.textContent||'').includes('3'))`,
  );
  await d.assertEval(
    `() => Array.from(document.querySelectorAll('button')).some(b => (b.textContent||'').includes('已完成') && (b.textContent||'').includes('2'))`,
  );

  await d.clickText("已完成", { exact: false });
  await d.assertText("演唱会C.mp4");
  await d.assertText("旧片E.mp4");
  await d.assertNoText("电影A.mp4");
});

test("搜索过滤：进行中/已完成各自生效", async (d) => {
  await d.resetAndGo({ tasks: [...ACTIVE_TASKS, ...COMPLETED_TASKS] });

  await d.fillByPlaceholder("搜索任务...", "演唱会");
  await d.assertText("没有下载任务");

  await d.clickText("已完成", { exact: false });
  await d.assertText("演唱会C.mp4");
  await d.assertNoText("旧片E.mp4");
});

test("排序切换为「最早优先」后最旧任务排最前", async (d) => {
  await d.resetAndGo({ tasks: [...ACTIVE_TASKS] });

  await d.selectOption("最新优先", "最早优先");
  await d.assertEval(
    `() => document.querySelector('.task-card h4').textContent.trim() === "电影A.mp4"`,
  );
});

test("点击卡片打开详情：URL、复制链接、媒体信息", async (d) => {
  await d.resetAndGo({ tasks: [COMPLETED_TASKS[0]] });
  await d.clickText("已完成", { exact: false });

  await d.clickCard("演唱会C.mp4");
  await d.assertText("下载链接");
  await d.assertText("https://example.com/concert.m3u8");
  await d.assertText("文件信息");
  await d.assertText("1920x1080");

  await d.clickTitle("复制");
  await d.assertText("链接已复制");
  const state = await d.mockState();
  assertEqual(state.clipboardText, "https://example.com/concert.m3u8");
});

test("右键菜单：复制链接/文件名/路径，路径仅完成且有输出路径时出现", async (d) => {
  await d.resetAndGo({ tasks: [ACTIVE_TASKS[0], COMPLETED_TASKS[0]] });

  // 进行中任务：无「复制文件路径」
  await d.contextMenuOnCard("电影A.mp4");
  await d.assertText("以此链接重新下载");
  await d.assertText("复制下载链接");
  await d.assertNoText("复制文件路径");

  await d.clickText("复制下载链接");
  await d.assertText("已复制下载链接");
  let state = await d.mockState();
  assertEqual(state.clipboardText, "https://example.com/old.m3u8");

  // 已完成任务：有「复制文件路径」
  await d.clickText("已完成", { exact: false });
  await d.contextMenuOnCard("演唱会C.mp4");
  await d.clickText("复制文件路径");
  await d.assertText("已复制文件路径");
  state = await d.mockState();
  assertEqual(state.clipboardText, "C:\\Downloads\\StreamGrab\\演唱会C.mp4");
});

test("右键「以此链接重新下载」预填对话框并走重复确认", async (d) => {
  await d.resetAndGo({ tasks: [ACTIVE_TASKS[0]] });

  await d.contextMenuOnCard("电影A.mp4");
  await d.clickText("以此链接重新下载");
  await d.assertText("添加下载任务");
  await d.assertText("HLS");

  await d.clickText("完成");
  await d.assertText("链接已存在");
  await d.clickText("仍然下载");

  const state = await d.mockState();
  assertEqual(state.tasks.length, 2);
});

test("删除失败任务：确认后移除并调用 delete_task", async (d) => {
  await d.resetAndGo({ tasks: [ACTIVE_TASKS[2]] });

  // 失败任务删除不弹确认框（仅完成且有文件时弹）
  await d.clickTaskAction("删除", "失败D.mp4");
  await d.assertNoText("失败D.mp4");
  const state = await d.mockState();
  assertEqual(state.tasks.length, 0);
  const deleteCalls = await d.mockCallsOf("delete_task");
  assertEqual(
    deleteCalls.map((c) => c.args.taskId),
    ["t-fail"],
  );
});

test("删除已完成任务：可勾选同时删除文件", async (d) => {
  await d.resetAndGo({ tasks: [COMPLETED_TASKS[0]] });
  await d.clickText("已完成", { exact: false });

  await d.clickTaskAction("删除", "演唱会C.mp4");
  await d.assertText("同时删除下载的文件");
  await d.clickFirstCheckbox();
  await d.clickText("确认删除");

  await d.assertNoText("演唱会C.mp4");
  const deleteFileCalls = await d.mockCallsOf("delete_file_or_folder");
  assertEqual(
    deleteFileCalls.map((c) => c.args.path),
    ["C:\\Downloads\\StreamGrab\\演唱会C.mp4"],
  );
});

test("完成文件缺失时卡片提示「文件已移除」", async (d) => {
  await d.resetAndGo({
    tasks: [COMPLETED_TASKS[1]],
    fileExistsMap: { "C:\\Missing\\old.mp4": false },
  });
  await d.clickText("已完成", { exact: false });

  await d.assertText("文件已移除");
});

test("清除已完成：移除完成项并保留进行中", async (d) => {
  await d.resetAndGo({ tasks: [ACTIVE_TASKS[0], ...COMPLETED_TASKS] });

  await d.clickText("清除已完成");
  await d.assertText("电影A.mp4");
  await d.assertNoText("演唱会C.mp4");
  await d.assertNoText("旧片E.mp4");

  const state = await d.mockState();
  assertEqual(state.tasks.length, 1);
  assertEqual((await d.mockCallsOf("clear_finished_tasks")).length, 1);
});

test("开始全部：逐个启动待下载任务", async (d) => {
  await d.resetAndGo({
    tasks: [
      makeTask({
        id: "p1",
        url: "https://example.com/p1.m3u8",
        fileName: "待下载P1.mp4",
        status: "pending",
      }),
      makeTask({
        id: "p2",
        url: "https://example.com/p2.m3u8",
        fileName: "待下载P2.mp4",
        status: "pending",
      }),
    ],
  });

  await d.clickText("开始全部");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().filter(c => c.command === "start_download").length === 2`,
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().tasks.every(t => t.status === "downloading")`,
  );

  const startCalls = await d.mockCallsOf("start_download");
  assertEqual(startCalls.map((c) => c.args.taskId).sort(), ["p1", "p2"]);
});
