import { describe, expect, it } from "vitest";
import { cacheSet, fileName, formatBytes, pasteFallbackMessage } from "./view";

describe("clip view helpers", () => {
  it("formats file paths and byte sizes", () => {
    expect(fileName("file:///Users/me/My%20File.txt")).toBe("My%20File.txt");
    expect(fileName("C:\\Temp\\note.txt")).toBe("note.txt");
    expect(formatBytes(1536)).toBe("1.5 KB");
  });

  it("evicts the oldest cached entry at the configured ceiling", () => {
    const cache = new Map([["old", 1], ["keep", 2]]);
    cacheSet(cache, "new", 3, 2);
    expect([...cache.entries()]).toEqual([["keep", 2], ["new", 3]]);
  });

  it("does not claim a second in-flight paste copied anything", () => {
    expect(pasteFallbackMessage("already_in_progress")).toBe("正在处理上一次粘贴，请稍后重试");
  });
});
