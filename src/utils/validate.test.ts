/**
 * 验证工具测试
 */

import { describe, expect, it } from "vitest";
import {
  validateFileName,
  validateFilePath,
  validateHexKey,
  validatePort,
  validateProxyUrl,
  validateRegex,
  validateUrl,
  validateUrls,
} from "./validate";

describe("validateUrl", () => {
  it("接受合法的流媒体链接", () => {
    expect(validateUrl("https://example.com/a.m3u8")).toMatchObject({
      valid: true,
      type: "m3u8",
    });
    expect(validateUrl("https://example.com/a.mpd")).toMatchObject({
      valid: true,
      type: "mpd",
    });
    expect(validateUrl("https://example.com/a.ism/manifest")).toMatchObject({
      valid: true,
      type: "mss",
    });
  });

  it("有效但类型未知的 URL", () => {
    const result = validateUrl("https://example.com/video.mp4");
    expect(result.valid).toBe(true);
    expect(result.type).toBe("unknown");
  });

  it("拒绝空值与非 http 协议", () => {
    expect(validateUrl("").valid).toBe(false);
    expect(validateUrl("   ").valid).toBe(false);
    expect(validateUrl("ftp://example.com/a").valid).toBe(false);
  });

  it("拒绝格式无效的 URL", () => {
    expect(validateUrl("http://").valid).toBe(false);
  });
});

describe("validateUrls 批量", () => {
  it("分离有效与无效", () => {
    const { valid, invalid } = validateUrls([
      "https://example.com/a.m3u8",
      "",
      "not a url",
      "https://example.com/b.mpd",
    ]);
    expect(valid).toHaveLength(2);
    expect(invalid).toHaveLength(2);
  });
});

describe("validateFilePath", () => {
  it("合法路径", () => {
    expect(validateFilePath("D:/Videos").valid).toBe(true);
    expect(validateFilePath("/home/user/videos").valid).toBe(true);
  });

  it("非法 Windows 路径", () => {
    expect(validateFilePath("D:Videos").valid).toBe(false);
    expect(validateFilePath("D:/Vi<deo").valid).toBe(false);
    expect(validateFilePath("D:/a|b").valid).toBe(false);
  });

  it("非法 Unix 路径", () => {
    expect(validateFilePath("/home//user").valid).toBe(false);
  });

  it("空路径", () => {
    expect(validateFilePath("").valid).toBe(false);
  });
});

describe("validateFileName", () => {
  it("合法文件名", () => {
    expect(validateFileName("episode-01.mp4").valid).toBe(true);
  });

  it("拒绝非法字符", () => {
    expect(validateFileName("a/b.mp4").valid).toBe(false);
    expect(validateFileName('a:b"c').valid).toBe(false);
  });

  it("拒绝 Windows 保留名", () => {
    expect(validateFileName("CON").valid).toBe(false);
    expect(validateFileName("nul.txt").valid).toBe(false);
    expect(validateFileName("CONSOLE").valid).toBe(true);
  });

  it("拒绝过长文件名", () => {
    expect(validateFileName("a".repeat(201)).valid).toBe(false);
    expect(validateFileName("a".repeat(200)).valid).toBe(true);
  });
});

describe("validatePort", () => {
  it("合法端口", () => {
    expect(validatePort(8080).valid).toBe(true);
    expect(validatePort("443").valid).toBe(true);
    expect(validatePort(1).valid).toBe(true);
    expect(validatePort(65535).valid).toBe(true);
  });

  it("越界与非数字", () => {
    expect(validatePort(0).valid).toBe(false);
    expect(validatePort(65536).valid).toBe(false);
    expect(validatePort("abc").valid).toBe(false);
  });
});

describe("validateProxyUrl", () => {
  it("支持 http/socks 协议", () => {
    expect(validateProxyUrl("http://127.0.0.1:7890").valid).toBe(true);
    expect(validateProxyUrl("socks5://127.0.0.1:1080").valid).toBe(true);
  });

  it("拒绝无协议与非法格式", () => {
    expect(validateProxyUrl("127.0.0.1:7890").valid).toBe(false);
    expect(validateProxyUrl("").valid).toBe(false);
  });
});

describe("validateRegex", () => {
  it("合法与非法正则", () => {
    expect(validateRegex("^video_\\d+$").valid).toBe(true);
    expect(validateRegex("[invalid(").valid).toBe(false);
    expect(validateRegex("").valid).toBe(false);
  });
});

describe("validateHexKey", () => {
  it("16 字节密钥（32 个十六进制字符）", () => {
    expect(validateHexKey("00112233445566778899aabbccddeeff").valid).toBe(true);
  });

  it("拒绝非十六进制与长度错误", () => {
    expect(validateHexKey("xyz").valid).toBe(false);
    expect(validateHexKey("0011").valid).toBe(false);
  });

  it("忽略空白字符", () => {
    expect(validateHexKey("00112233 44556677 8899aabb ccddeeff").valid).toBe(
      true,
    );
  });
});
