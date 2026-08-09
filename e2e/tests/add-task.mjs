import { assert, assertEqual, test } from "../runner-lib.mjs";
import { makeTask } from "./app-shell.mjs";

const HLS_URL = "https://example.com/live/index.m3u8";
const MP4_URL = "https://example.com/movie.mp4";
const PASTE_PLACEHOLDER = "粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 直链）";

async function openAddDialog(d) {
  await d.clickText("添加任务");
  await d.assertText("添加下载任务");
}

test("单条 HLS：粘贴 → 解析 → 配置 → 添加并自动开始", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");

  await d.assertText("HLS");
  await d.assertText("已解析");
  await d.assertEval(
    `() => document.querySelector('input[placeholder="自动从 URL 提取"]').value === "index"`,
  );

  await d.clickText("完成");
  await d.assertNoText("添加下载任务");
  await d.assertText("已添加 1 个任务");
  await d.assertText("index");
  // TOAST_LIMIT=1：开始下载 toast 会被「已添加」顶掉，改断言状态与命令
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getState().tasks[0].status === "downloading"`,
  );
  await d.assertEval(
    `() => window.__STREAMGRAB_MOCK__.getCalls().some(c => c.command === "start_download")`,
  );

  const parseCalls = await d.mockCallsOf("parse_url");
  assertEqual(parseCalls.length, 1);
  assertEqual(parseCalls[0].args.url, HLS_URL);

  await d.assertEval(
    `() => {
      const t = window.__STREAMGRAB_MOCK__.getState().tasks;
      return t.length === 1 && t[0].url === ${JSON.stringify(HLS_URL)};
    }`,
  );
});

test("直链视频：无需解析直接进入配置步", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, MP4_URL);
  await d.clickText("解析并添加");

  await d.assertText("直链视频");
  assertEqual((await d.mockCallsOf("parse_url")).length, 0);

  await d.clickText("完成");
  await d.assertText("已添加 1 个任务");
  await d.assertText("movie");
});

test("无效链接被剔除并 toast 汇报", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(
    PASTE_PLACEHOLDER,
    `not-a-url\nhttps://example.com/page.html\n${MP4_URL}`,
  );
  await d.clickText("解析并添加");

  await d.assertText("1 个链接无法识别已跳过");
  await d.assertText("直链视频");

  await d.clickText("完成");
  const state = await d.mockState();
  assertEqual(state.tasks.length, 1);
  assertEqual(state.tasks[0].url, MP4_URL);
});

test("全部无效时提示未识别到有效链接", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(
    PASTE_PLACEHOLDER,
    "https://example.com/page.html\nftp://example.com/x.m3u8",
  );
  await d.clickText("解析并添加");

  // ftp:// 不进入 extractLinks（仅 http/https），实际只跳过 page.html 一条
  await d.assertText("1 个链接无法识别已跳过");
  await d.assertText("解析并添加");
});

test("多条链接：「全部添加」批量入库", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, `${HLS_URL}\n${MP4_URL}`);
  await d.clickText("解析并添加");

  await d.assertText("1/2");
  await d.clickText("全部添加");
  await d.assertText("已添加 2 个任务");
  await d.assertText("index");
  await d.assertText("movie");

  const state = await d.mockState();
  assertEqual(state.tasks.length, 2);
});

test("重复 URL：确认后仍可添加（自动重命名）", async (d) => {
  await d.resetAndGo({
    tasks: [
      makeTask({
        id: "dup",
        url: HLS_URL,
        fileName: "index.m3u8",
        status: "completed",
      }),
    ],
  });
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");
  await d.assertText("已解析");
  await d.clickText("完成");

  await d.assertText("链接已存在");
  await d.clickText("仍然下载");
  await d.assertText("已添加 1 个任务");

  const state = await d.mockState();
  assertEqual(state.tasks.length, 2);
  assert(state.tasks[1].fileName !== "index.m3u8", "冲突任务应自动重命名");
});

test("重复 URL：取消则跳过", async (d) => {
  await d.resetAndGo({
    tasks: [
      makeTask({
        id: "dup",
        url: HLS_URL,
        fileName: "index.m3u8",
        status: "pending",
      }),
    ],
  });
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");
  await d.clickText("完成");
  await d.assertText("链接已存在");
  await d.clickText("取消");

  await d.assertNoText("添加下载任务");
  const state = await d.mockState();
  assertEqual(state.tasks.length, 1);
});

test("流解析失败：可重试成功后添加", async (d) => {
  await d.resetAndGo({
    parseOverrides: { [HLS_URL]: { error: "mock manifest error" } },
  });
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");

  await d.assertText("解析失败");

  // 修复 mock 后重试成功
  await d.clickText("高级设置");
  await d.assertText("重试解析");
  await d.mockSetParseResult(HLS_URL, {});
  await d.clickText("重试解析");
  await d.assertText("已解析");

  await d.clickText("完成");
  const state = await d.mockState();
  assertEqual(state.tasks.length, 1);
  assertEqual(state.tasks[0].url, HLS_URL);
});

test("高级设置：内联流选择后显示已选摘要", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");
  await d.assertText("已解析");

  await d.clickText("高级设置");
  await d.clickText("选择流");
  await d.assertText("共 1 个视频流、1个音频流、1个字幕流");
  await d.clickText("确认选择");
  await d.assertText(/已选：视频 .* 音频 .* 字幕 /);

  await d.clickText("完成");
  const state = await d.mockState();
  assert(state.tasks[0].overrides?.selection, "任务应携带流选择");
});

test("流选择：默认勾选最高画质（带宽优先，与列表顺序无关）", async (d) => {
  // 故意把低清流放在列表前面，验证默认选中的是带宽最高的那条
  const info = {
    videos: [
      {
        id: "v-low",
        bandwidth: 1_000_000,
        codecs: "avc1",
        language: "und",
        name: "360P",
        groupId: null,
        selected: null,
        resolution: "640x360",
        width: 640,
        height: 360,
        frameRate: 25,
        videoRange: "SDR",
      },
      {
        id: "v-high",
        bandwidth: 8_000_000,
        codecs: "avc1",
        language: "und",
        name: "1080P",
        groupId: null,
        selected: null,
        resolution: "1920x1080",
        width: 1920,
        height: 1080,
        frameRate: 25,
        videoRange: "SDR",
      },
    ],
    audios: [
      {
        id: "a1",
        bandwidth: 192000,
        codecs: "mp4a",
        language: "zh",
        name: "",
        groupId: null,
        selected: null,
        channels: "2ch",
        sampleRate: 48000,
        isDefault: true,
      },
    ],
    subtitles: [],
    duration: 3725,
    segmentCount: 745,
    isLive: false,
    isEncrypted: false,
  };

  await d.resetAndGo({ parseOverrides: { [HLS_URL]: { info } } });
  await openAddDialog(d);
  await d.fillByPlaceholder(PASTE_PLACEHOLDER, HLS_URL);
  await d.clickText("解析并添加");
  await d.assertText("已解析");

  await d.clickText("高级设置");
  await d.clickText("选择流");
  await d.assertText("共 2 个视频流、1个音频流、0个字幕流");
  await d.clickText("确认选择");
  await d.assertText("已选：视频 v-high · 音频 a1 · 字幕 自动");

  await d.clickText("完成");
  const state = await d.mockState();
  assertEqual(state.tasks[0].overrides.selection.video, "v-high");
});

test("高级设置：定时开始 → 任务保持等待并显示定时文案", async (d) => {
  await d.resetAndGo({ appSettings: { auto_start_download: true } });
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, MP4_URL);
  await d.clickText("解析并添加");
  await d.clickText("高级设置");
  await d.clickSwitch("定时开始");
  await d.assertEval(`() => !!document.querySelector('input[type="datetime-local"]')`);
  await d.fillBySelector('input[type="datetime-local"]', "2020-01-01T00:00");
  await d.clickText("完成");

  await d.assertText("已添加 1 个任务");
  await d.assertText("定时 01-01 00:00 开始");

  const state = await d.mockState();
  assertEqual(state.tasks[0].overrides.scheduledStartAt, "2020-01-01T00:00");
  assertEqual((await d.mockCallsOf("start_download")).length, 0);
});

test("拖拽文本到粘贴区可填入链接", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.dropTextOnTextarea(MP4_URL);
  await d.assertEval(
    `() => document.querySelector('textarea').value === ${JSON.stringify(MP4_URL)}`,
  );
  await d.clickText("解析并添加");
  await d.assertText("直链视频");
});

test("最近保存目录记忆：再次打开显示最近目录", async (d) => {
  await d.resetAndGo(null);
  await openAddDialog(d);

  await d.fillByPlaceholder(PASTE_PLACEHOLDER, MP4_URL);
  await d.clickText("解析并添加");
  await d.fillByPlaceholder("使用全局默认", "D:\\Downloads\\videos");
  await d.clickText("完成");
  await d.assertText("已添加 1 个任务");

  // 再次打开：粘贴并进入配置步后显示最近目录
  await openAddDialog(d);
  await d.fillByPlaceholder(PASTE_PLACEHOLDER, MP4_URL);
  await d.clickText("解析并添加");
  await d.assertText("最近：D:\\Downloads\\videos");
});
