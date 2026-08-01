import { describe, it, expect } from "vitest";
import { rememberDir, resolveDefaultDir } from "./recentDirs";

describe("rememberDir", () => {
  it("新目录置最前", () => {
    expect(rememberDir(["A", "B"], "C")).toEqual(["C", "A", "B"]);
  });
  it("已存在则提到最前去重", () => {
    expect(rememberDir(["A", "B", "C"], "B")).toEqual(["B", "A", "C"]);
  });
  it("截断到上限 5", () => {
    expect(rememberDir(["1", "2", "3", "4", "5"], "6")).toEqual(["6", "1", "2", "3", "4"]);
  });
  it("空白目录忽略，原样返回", () => {
    const list = ["A"];
    expect(rememberDir(list, "   ")).toEqual(["A"]);
  });
  it("trim 后写入", () => {
    expect(rememberDir([], "  D:/x  ")).toEqual(["D:/x"]);
  });
});

describe("resolveDefaultDir", () => {
  it("最近记忆优先", () => {
    expect(resolveDefaultDir(["D:/recent", "D:/old"], "D:/global")).toBe("D:/recent");
  });
  it("无记忆回退全局", () => {
    expect(resolveDefaultDir([], "D:/global")).toBe("D:/global");
  });
  it("皆无返回空串", () => {
    expect(resolveDefaultDir([], "  ")).toBe("");
  });
});
