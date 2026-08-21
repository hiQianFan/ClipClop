// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ClipPreview from "./ClipPreview.svelte";
import type { ClipDetail, HistoryPage } from "./types";

const detail: ClipDetail = {
  id: "files",
  content_type: "file",
  preview: "/tmp/one.txt",
  plain_text: null,
  source_app: null,
  created_at: "2026-01-01T00:00:00Z",
  byte_size: 2,
  metadata: { files: ["/tmp/one.txt", "/tmp/two.txt"] },
  flavors: [],
};

const page: HistoryPage = { items: [detail], page: 1, page_size: 10, total: 1, total_pages: 1 };

describe("ClipPreview file tabs", () => {
  it("activates adjacent files once, does not loop, and forwards Escape", async () => {
    const onfile = vi.fn();
    const onfilekeydown = vi.fn();
    render(ClipPreview, { props: {
      detail, selectedId: detail.id, page, pending: false, assetUrl: null,
      fileAccessDenied: false, sourceIconUrl: null, fileThumbnailUrls: [null, null],
      fileByteSizes: [null, null], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "⌘←", nextFileShortcut: "⌘→", onfile,
      onfilekeydown, onfilefocus() {}, oninert() {},
    } });

    const first = screen.getByRole("tab", { name: /one\.txt/ });
    first.focus();
    await fireEvent.keyDown(first, { key: "ArrowLeft" });
    expect(onfile).not.toHaveBeenCalled();

    await fireEvent.keyDown(first, { key: "ArrowRight" });
    expect(onfile).toHaveBeenCalledTimes(1);
    expect(onfile).toHaveBeenCalledWith(1);

    await fireEvent.keyDown(first, { key: "Escape" });
    expect(onfilekeydown).toHaveBeenCalledWith(expect.objectContaining({ key: "Escape" }));
  });
});
