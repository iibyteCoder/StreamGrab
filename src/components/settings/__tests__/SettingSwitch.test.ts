/** @vitest-environment happy-dom */
/**
 * SettingSwitch 原语测试
 *
 * 所有设置开关的构建块：内部 Switch（reka）触发 update:modelValue → 转发为 update:modelValue。
 */

import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import SettingSwitch from "../SettingSwitch.vue";
import { Switch } from "@/components/ui/switch";

describe("SettingSwitch", () => {
  it("内部 Switch 开启 → 发出 update:modelValue=true", async () => {
    const wrapper = mount(SettingSwitch, {
      props: { modelValue: false, label: "测试开关" },
    });
    const inner = wrapper.findComponent(Switch);
    await inner.vm.$emit("update:modelValue", true);
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual([true]);
  });

  it("内部 Switch 关闭 → 发出 update:modelValue=false", async () => {
    const wrapper = mount(SettingSwitch, {
      props: { modelValue: true, label: "测试开关" },
    });
    const inner = wrapper.findComponent(Switch);
    await inner.vm.$emit("update:modelValue", false);
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual([false]);
  });

  it("渲染 label 与描述", () => {
    const wrapper = mount(SettingSwitch, {
      props: { modelValue: false, label: "标题", description: "说明" },
    });
    expect(wrapper.text()).toContain("标题");
    expect(wrapper.text()).toContain("说明");
  });
});
