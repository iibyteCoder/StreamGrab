import { assert, assertEqual, test } from "../runner-lib.mjs";
import { makeTask } from "./app-shell.mjs";

const TASK = makeTask({
  id: "dl-1",
  url: "https://example.com/movie/index.m3u8",
  fileName: "电影.m3u8",
  status: "pending",
});

async function startTask(d) {
  await d.clickTaskAction("开始", "电影.m3u8");
  await d.assertText("开始下载: 电影.m3u8");
  await d.mockClearCalls();
}

test("开始 → 进度 → 日志 → 完成（含媒体分析）", async (d) => {
  await d.resetAndGo({ tasks: [TASK] });
  await startTask(d);

  await d.mockEmit(`download:progress:dl-1`, {
    percent: 45,
    overallPercent: 45,
    speed: 10485760,
    downloadedSize: 314572800,
    totalSize: 734003200,
    downloadedSegments: 335,
    totalSegments: 745,
    eta: 40,
    currentAction: "",
  });
  await d.assertText("45%");
  await d.assertText("300 MB / 700 MB");
  await d.assertText("剩余");

  await d.mockEmit(`download:log:dl-1`, {
    level: "info",
    message: "Downloading segment 100",
  });
  await d.mockEmit(`download:log:dl-1`, {
    level: "warn",
    message: "Retry segment 101",
  });
  await d.clickTaskAction("日志", "电影.m3u8");
  await d.assertText("任务日志");
  await d.assertText("Downloading segment 100");
  await d.assertText("Retry segment 101");
  await d.clickText("关闭");

  await d.mockEmit(`download:complete:dl-1`, {
    outputPath: "C:\\Downloads\\StreamGrab\\电影.mp4",
  });
  await d.assertText("下载完成!");
  await d.assertNoText("电影.m3u8");

  await d.clickText("已完成", { exact: false });
  await d.assertText("电影.m3u8");
  await d.clickCard("电影.m3u8");
  await d.assertText("文件信息");
  await d.assertText("1920x1080");

  const analyzeCalls = await d.mockCallsOf("analyze_media_file");
  assertEqual(
    analyzeCalls.map((c) => c.args.filePath),
    ["C:\\Downloads\\StreamGrab\\电影.mp4"],
  );
});

test("暂停 / 继续 / 停止 状态流转", async (d) => {
  await d.resetAndGo({ tasks: [TASK] });
  await startTask(d);

  await d.clickTaskAction("暂停", "电影.m3u8");
  await d.assertText("下载已暂停");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().tasks[0].status === "paused"`,
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().some(c => c.command === "pause_download")`,
  );

  await d.clickTaskAction("继续", "电影.m3u8");
  await d.assertText("下载已恢复");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().tasks[0].status === "downloading"`,
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().some(c => c.command === "start_download")`,
  );

  await d.clickTaskAction("停止", "电影.m3u8");
  await d.assertText("下载已取消");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().tasks[0].status === "cancelled"`,
  );
});

test("下载出错：展示错误并可重试", async (d) => {
  await d.resetAndGo({ tasks: [TASK] });
  await startTask(d);

  await d.mockEmit(`download:error:dl-1`, { message: "HTTP 403 Forbidden" });
  await d.assertText("下载出错: HTTP 403 Forbidden");
  await d.assertText("HTTP 403 Forbidden");
  let state = await d.mockState();
  assertEqual(state.tasks[0].status, "failed");

  await d.clickTaskAction("重试", "电影.m3u8");
  await d.assertText("开始下载: 电影.m3u8");

  const statusCalls = await d.mockCallsOf("update_task_status");
  assert(
    statusCalls.some(
      (c) => c.args.taskId === "dl-1" && c.args.status === "pending",
    ),
    "重试应先置为 pending",
  );
  assert(
    (await d.mockCallsOf("start_download")).length >= 1,
    "重试应重新启动下载",
  );
  state = await d.mockState();
  assertEqual(state.tasks[0].status, "downloading");
});

test("重启应用：进行中任务标记中断并弹恢复框", async (d) => {
  await d.resetAndGo({ tasks: [TASK] });
  await startTask(d);
  let state = await d.mockState();
  assertEqual(state.tasks[0].status, "downloading");

  await d.reload();

  await d.assertText("恢复中断的下载");
  await d.assertText("检测到 1 个未完成的下载任务");
  await d.assertText("电影.m3u8");

  await d.clickText("稍后");
  await d.assertNoText("恢复中断的下载");

  state = await d.mockState();
  assertEqual(state.tasks[0].status, "paused");
  assertEqual(state.tasks[0].wasInterrupted, true);
});

test("定时任务：未来时间不被自动启动，展示定时文案", async (d) => {
  await d.resetAndGo({
    appSettings: { auto_start_download: true },
    tasks: [
      makeTask({
        id: "sched",
        url: "https://example.com/future.m3u8",
        fileName: "未来任务.m3u8",
        status: "pending",
        overrides: { scheduledStartAt: "2099-01-01T00:00" },
      }),
    ],
  });

  await d.assertText("定时 01-01 00:00 开始");
  await new Promise((r) => setTimeout(r, 500));
  assertEqual((await d.mockCallsOf("start_download")).length, 0);
});
