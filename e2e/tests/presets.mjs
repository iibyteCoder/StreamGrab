import { assertEqual, test } from "../runner-lib.mjs";

test("预设：新建（含覆盖项）→ 编辑 → 复制 → 删除", async (d) => {
  await d.resetAndGo(null, "/settings");
  await d.clickText("任务预设");
  await d.assertText("暂无预设");

  // 新建
  await d.clickText("新建预设");
  await d.assertText("新建预设", { last: true });
  await d.fillByPlaceholder("例如：B站 1080P", "4K 高码率");
  await d.selectOption("沿用默认", "MKV");
  await d.clickText("新建预设", { last: true }); // 对话框底部保存按钮
  await d.assertText("预设已创建");

  let state = await d.mockState();
  assertEqual(state.presets.length, 1);
  assertEqual(state.presets[0].name, "4K 高码率");
  assertEqual(state.presets[0].overrides.muxFormat, "mkv");
  await d.assertText("格式: mkv");

  // 编辑
  await d.clickTitle("编辑");
  await d.assertText("编辑预设");
  await d.fillByPlaceholder("例如：B站 1080P", "4K HDR 高码率");
  await d.clickText("保存");
  await d.assertText("预设已更新");
  await d.assertText("4K HDR 高码率");

  // 复制
  await d.clickTitle("复制");
  await d.assertText("预设已复制");
  state = await d.mockState();
  assertEqual(state.presets.length, 2);

  // 删除
  await d.clickTitle("删除");
  await d.assertText("确定要删除预设");
  await d.clickText("删除");
  await d.assertText("预设已删除");
  state = await d.mockState();
  assertEqual(state.presets.length, 1);
  assertEqual((await d.mockCallsOf("delete_preset")).length, 1);
});
