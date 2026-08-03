import { describe, it, expect } from "vitest";
import { extractLinks, classifyLink, parsePastedText } from "../parseLinks";

describe("extractLinks", () => {
  it("trim 并只保留 http(s) 行", () => {
    expect(
      extractLinks("  https://a/1.m3u8  \nftp://x\nhttp://b/2.mp4\nnotaurl"),
    ).toEqual(["https://a/1.m3u8", "http://b/2.mp4"]);
  });
  it("按出现顺序去重", () => {
    expect(extractLinks("https://a/1\nhttps://a/1\nhttps://a/2")).toEqual([
      "https://a/1",
      "https://a/2",
    ]);
  });
  it("空文本返回空数组", () => {
    expect(extractLinks("  \n  ")).toEqual([]);
  });
});

describe("classifyLink", () => {
  it("hls 标记 streaming", () => {
    const r = classifyLink("https://a/x.m3u8");
    expect(r.detectedType).toBe("hls");
    expect(r.streaming).toBe(true);
  });
  it("mp4 直链非 streaming，且提取文件名", () => {
    const r = classifyLink("https://a/dir/movie.mp4");
    expect(r.detectedType).toBe("httpVideo");
    expect(r.streaming).toBe(false);
    expect(r.fileName).toBe("movie");
  });
});

describe("parsePastedText", () => {
  it("剔除 unknown 并计数，保留有效链接", () => {
    const { links, skipped } = parsePastedText(
      "https://a/x.m3u8\nhttps://a/page.html\nhttps://a/y.mp4",
    );
    expect(skipped).toBe(1);
    expect(links.map((l) => l.url)).toEqual([
      "https://a/x.m3u8",
      "https://a/y.mp4",
    ]);
  });
  it("全部无效时 links 为空", () => {
    const { links, skipped } = parsePastedText("https://a/page.html");
    expect(links).toEqual([]);
    expect(skipped).toBe(1);
  });
});
