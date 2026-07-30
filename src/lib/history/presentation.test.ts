import { describe, expect, it } from "vitest";
import { cacheSet, clipPreview, fileName, formatBytes } from "./presentation";

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

  it("receives localized fallback text from its caller", () => {
    expect(fileName("", "File")).toBe("File");
    expect(clipPreview({ content_type: "file", preview: "" }, "文件")).toBe("文件");
  });
});
