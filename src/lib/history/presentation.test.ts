import { describe, expect, it } from "vitest";
import { cacheSet, clipPreview, detailText, fileName, formatBytes, metadataFacts } from "./presentation";
import type { ClipDetail } from "./types";

const labels = { dimensions: "Dimensions", size: "Size", file: "File", files: "Files", hostname: "Domain", type: "Type", characters: "Characters" };

function detail(content_type: ClipDetail["content_type"], metadata: ClipDetail["metadata"] = {}, plain_text: string | null = null): ClipDetail {
  return {
    id: "clip", content_type, preview: plain_text ?? "preview", source_app: null,
    created_at: "2026-08-06T00:00:00Z", last_used_at: "2026-08-07T00:00:00Z",
    byte_size: 12, metadata, plain_text, flavors: [],
  };
}

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
      last_used_at: "2026-08-06T00:00:00Z",
      byte_size: 0,
      metadata: { files: ["a"], file_sizes: [null] },
      plain_text: null,
      flavors: [],
    };
    const facts = metadataFacts(
      detail,
      0,
      [2048],
      labels,
      String,
    );
    expect(facts).toContainEqual({ label: "Size", value: "2 KB" });
    expect(detail.metadata.file_sizes).toEqual([null]);
  });

  it("derives the approved facts for text, links, images, and colors", () => {
    expect(metadataFacts(detail("text", { char_count: 5 }, "hello"), 0, [], labels, String)).toEqual([
      { label: "Characters", value: "5" }, { label: "Size", value: "12 B" },
    ]);
    expect(metadataFacts(detail("text", {}, "😀"), 0, [], labels, String)[0]).toEqual({ label: "Characters", value: "1" });
    expect(metadataFacts(detail("link", {}, "https://docs.example.com/path"), 0, [], labels, String)).toEqual([
      { label: "Domain", value: "docs.example.com", action: "open-origin" }, { label: "Characters", value: "29" },
    ]);
    expect(metadataFacts(detail("link", {}, "https://example.com:8443/path"), 0, [], labels, String)[0]).toEqual({
      label: "Domain", value: "example.com:8443", action: "open-origin",
    });
    expect(metadataFacts(detail("link", {}, "not a url"), 0, [], labels, String)).toEqual([
      { label: "Characters", value: "9" },
    ]);
    expect(metadataFacts(detail("image", { width: 1920, height: 1080 }), 0, [], labels, String)).toEqual([
      { label: "Dimensions", value: "1920 × 1080" }, { label: "Size", value: "12 B" },
    ]);
    expect(metadataFacts(detail("color", {}, "#abcdef"), 0, [], labels, String)).toEqual([
      { label: "Type", value: "HEX" },
    ]);
  });

  it("shows file type/count and only claims a total when every size is known", () => {
    expect(metadataFacts(detail("file", { files: ["/tmp/report.pdf"], file_sizes: [1024] }), 0, [], labels, String)).toEqual([
      { label: "Type", value: "PDF" }, { label: "Size", value: "1 KB" },
    ]);
    expect(metadataFacts(detail("file", { files: ["/tmp/README"] }), 0, [], labels, String)).toEqual([
      { label: "Type", value: "File" },
    ]);
    expect(metadataFacts(detail("file", { files: ["a", "b"], file_sizes: [1024, 1024] }), 0, [], labels, String)).toEqual([
      { label: "Files", value: "2" }, { label: "Size", value: "2 KB" },
    ]);
    expect(metadataFacts(detail("file", { files: ["a", "b"], file_sizes: [1024, null] }), 0, [], labels, String)).toEqual([
      { label: "Files", value: "2" }, { label: "Size", value: "1 KB" },
    ]);
    expect(metadataFacts(detail("file", { files: ["a", "b"], file_sizes: [null, null] }), 0, [], labels, String)).toEqual([
      { label: "Files", value: "2" },
    ]);
  });
});
