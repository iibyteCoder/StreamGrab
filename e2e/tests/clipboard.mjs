import { assertEqual, test } from "../runner-lib.mjs";

const HLS_URL = "https://example.com/live/index.m3u8";

test("剪贴板监控默认关闭时不启动监听", async (d) => {
  await d.resetAndGo(null);
  await d.assertText("没有下载任务");

  assertEqual(await d.mockListenerCount("tauri://focus"), 0);
});

test("开启剪贴板监控后检测到 M3U8 链接并 toast", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.clickSwitch("剪贴板监视");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.clipboard_watch === true`,
  );
  await d.clickTitle("返回");
  await d.assertText("没有下载任务");

  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.listenerCount("tauri://focus") >= 1`,
  );

  await d.mockSetClipboard(HLS_URL);
  await d.mockEmit("tauri://focus", null);
  await d.assertText("已添加下载链接");
});
