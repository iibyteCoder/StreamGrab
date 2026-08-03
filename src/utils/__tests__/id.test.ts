/**
 * ID 生成工具测试
 */

import { describe, expect, it } from "vitest";
import { generateId } from "../id";

describe("generateId", () => {
  it("生成 UUID 格式", () => {
    expect(generateId()).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );
  });

  it("支持前缀", () => {
    expect(generateId("preset-")).toMatch(/^preset-[0-9a-f-]{36}$/);
  });

  it("大量生成不重复", () => {
    const ids = new Set(Array.from({ length: 1000 }, () => generateId()));
    expect(ids.size).toBe(1000);
  });
});
