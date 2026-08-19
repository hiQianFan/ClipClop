import { describe, expect, it } from "vitest";
import { cacheSet, clipPreview, detailText, fileName, formatBytes, metadataFacts, shouldReadOriginalFile } from "./presentation";
import type { ClipDetail } from "./types";

describe("clip view helpers", () => {
  it("formats file paths and byte sizes", () => {
    expect(fileName("file:///Users/me/My%20File.txt")).toBe("My%20File.txt");
    expect(fileName("C:\\Temp\\note.txt")).toBe("note.txt");
    expect(formatBytes(1536)).toBe("1.5 KB");
  });

  it("reads an original file only for file content with the switch enabled", () => {
    expect(shouldReadOriginalFile("file", false)).toBe(false);
    expect(shouldReadOriginalFile("file", true)).toBe(true);
    expect(shouldReadOriginalFile("image", false)).toBe(false);
    expect(shouldReadOriginalFile("image", true)).toBe(false);
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

  it("matches detail text to the whitespace setting", () => {
    const detail = { plain_text: "  hello\n", preview: "preview" } as ClipDetail;
    expect(detailText(detail, false)).toBe("  hello\n");
    expect(detailText(detail, true)).toBe("hello");
  });

  it("uses runtime file sizes without mutating persisted detail metadata", () => {
    const detail: ClipDetail = {
      id: "clip-1",
      content_type: "file",
      preview: "a",
      source_app: null,
      created_at: "2026-08-06T00:00:00Z",
      byte_size: 0,
      metadata: { files: ["a"], file_sizes: [null] },
      plain_text: null,
      flavors: [],
    };
    const facts = metadataFacts(
      detail,
      0,
      [2048],
      { dimensions: "Dimensions", size: "Size", file: "File", characters: "Characters" },
      String,
    );
    expect(facts).toContainEqual({ label: "Size", value: "2 KB" });
    expect(detail.metadata.file_sizes).toEqual([null]);
  });
});
