import { describe, expect, it } from "vitest";
import { compareVersions } from "../version";

describe("compareVersions", () => {
  it("equal versions return 0", () => {
    expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
    expect(compareVersions("v2.1.3", "2.1.3")).toBe(0);
  });

  it("greater version returns 1", () => {
    expect(compareVersions("1.1.0", "1.0.0")).toBe(1);
    expect(compareVersions("2.0.0", "1.9.9")).toBe(1);
    expect(compareVersions("v0.6.0", "v0.5.2")).toBe(1);
  });

  it("lesser version returns -1", () => {
    expect(compareVersions("1.0.0", "1.1.0")).toBe(-1);
    expect(compareVersions("0.5.2", "0.6.0")).toBe(-1);
  });

  it("handles different segment counts", () => {
    expect(compareVersions("1.0", "1.0.0")).toBe(0);
    expect(compareVersions("1.0.1", "1.0")).toBe(1);
  });

  it("handles non-numeric gracefully", () => {
    // NaN segments treated as 0 by || 0
    expect(compareVersions("latest", "latest")).toBe(0);
  });
});
