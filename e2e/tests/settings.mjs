import { assertEqual, test } from "../runner-lib.mjs";

test("设置页分区导航：四个分区可切换", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.assertText("语言与外观");

  await d.clickText("N_m3u8DL-RE");
  await d.assertText("工具管理");
  await d.clickText("FFmpeg");
  await d.assertText("直链下载");
  await d.clickText("任务预设");
  await d.assertText("新建预设");
});

test("语言切换为 English 后 UI 本地化", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.assertText("语言与外观");

  await d.selectOption("简体中文", "English");
  await d.assertText("Language & Appearance");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.language === "en-US"`,
  );
});

test("主题下拉切到浅色并持久化", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.selectOption("深色", "浅色");
  await d.assertEval(
    `() => !document.documentElement.classList.contains('dark')`,
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.theme === "light"`,
  );
});

test("默认保存目录：手动输入与目录选择器", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.fillByPlaceholder("./downloads", "D:\\Downloads\\videos");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.default_save_dir === "D:\\\\Downloads\\\\videos"`,
  );

  await d.mockSetDialogResult("directory", "D:\\Picked");
  await d.clickTitle("选择文件夹");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.default_save_dir === "D:\\\\Picked"`,
  );
});

test("最大并发任务数滑块可调", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.focusFirst('[role="slider"]');
  await d.pressKey("ArrowRight");

  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.max_concurrent_tasks === 6`,
  );
});

test("剪贴板监控开关：开启后持久化", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.clickSwitch("剪贴板监视");
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().appSettings.clipboard_watch === true`,
  );
});

test("重置设置：确认后恢复默认并持久化", async (d) => {
  await d.resetAndGo({
    appSettings: {
      theme: "light",
      max_concurrent_tasks: 9,
      default_save_dir: "D:\\Custom",
    },
  }, "/settings");

  await d.clickText("恢复默认设置");
  await d.assertText("确认恢复默认配置？");
  await d.clickText("确认");
  await d.assertText("设置已恢复为默认值");

  await d.assertEval(
    `() => {
      const s = window.__STREAMGRAB_MOCK__.getState().appSettings;
      return s.language === "zh-CN" && s.theme === "dark" && s.max_concurrent_tasks === 5 && s.default_save_dir === "";
    }`,
  );
});

test("导出配置：选择目录后调用 export_config", async (d) => {
  await d.resetAndGo(null, "/settings");

  await d.mockSetDialogResult("directory", "C:\\Exports");
  await d.clickText("导出配置");
  await d.assertText("成功");

  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().filter(c => c.command === "export_config").length === 1`,
  );
});

test("N_m3u8DL-RE 工具卡：检测、检查最新版并下载更新", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.clickText("N_m3u8DL-RE");

  await d.assertText("已安装");
  await d.assertText("v20260628");

  await d.clickText("检查最新版本");
  await d.assertText("最新版本: 9.9.9");
  await d.clickText("下载");
  await d.assertText("N_m3u8DL-RE 下载完成");

  const calls = await d.mockCallsOf("download_tool");
  assertEqual(calls.length, 1);
  assertEqual(calls[0].args.tool, "N_m3u8DL-RE");

  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().nm3u8dl.path === "C:\\\\Tools\\\\N_m3u8DL-RE"`,
  );
});

test("N_m3u8DL-RE 下载参数：开关与正则选择保存", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.clickText("N_m3u8DL-RE");

  await d.clickSwitch("自动选择最佳流");
  await d.fillByPlaceholder("例如: res=1080", "res=1080");

  await d.assertEval(
    `() => {
      const s = window.__STREAMGRAB_MOCK__.getState().nm3u8dl;
      return s.auto_select === false && s.select_video === "res=1080";
    }`,
  );
});

test("FFmpeg 直链参数：代理地址保存", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.clickText("FFmpeg");

  await d.fillByPlaceholder(
    "http://127.0.0.1:7890，留空关闭",
    "http://127.0.0.1:8888",
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().ffmpeg.http_proxy === "http://127.0.0.1:8888"`,
  );
});
