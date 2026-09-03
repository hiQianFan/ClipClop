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
  last_used_at: "2026-01-02T00:00:00Z",
  byte_size: 2,
  metadata: { files: ["/tmp/one.txt", "/tmp/two.txt"] },
  flavors: [],
};

const page: HistoryPage = { items: [detail], page: 1, page_size: 10, total: 1, total_pages: 1 };

describe("ClipPreview file tabs", () => {
  it("shows the app mark when clipboard history is empty", () => {
    const { container } = render(ClipPreview, { props: {
      detail: null, selectedId: null, page: { ...page, items: [], total: 0, total_pages: 0 }, noMatches: false, pending: false,
      assetUrl: null, thumbnailUrl: null, fileAccessDenied: false, sourceIconUrl: null,
      fileThumbnailUrls: [], fileByteSizes: [], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin() {}, oninert() {},
    } });
    expect(container.querySelector('.brand-empty img')?.getAttribute('src')).toBe('/app-icon-rounded.png');
  });

  it("uses the thumbnail while the full image preview is pending", () => {
    const image = { ...detail, id: "image", content_type: "image" as const, metadata: { width: 10, height: 10 } };
    const { container } = render(ClipPreview, { props: {
      detail: image, selectedId: image.id, page: { ...page, items: [image] }, noMatches: false, pending: false,
      assetUrl: null, thumbnailUrl: "thumbnail", fileAccessDenied: false, sourceIconUrl: null,
      fileThumbnailUrls: [], fileByteSizes: [], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin() {}, oninert() {},
    } });
    expect(container.querySelector("img.asset.thumbnail")?.getAttribute("src")).toBe("thumbnail");
  });

  it("presents copied image files as visual content without repeating their path", () => {
    const photo = { ...detail, metadata: { files: ["/Users/me/Pictures/photo.jpeg"] } };
    const { container } = render(ClipPreview, { props: {
      detail: photo, selectedId: photo.id, page: { ...page, items: [photo] }, noMatches: false, pending: false,
      assetUrl: "photo-preview", thumbnailUrl: null, fileAccessDenied: false, sourceIconUrl: null,
      fileThumbnailUrls: [], fileByteSizes: [1024], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin() {}, oninert() {},
    } });

    expect(container.querySelector("img.asset")?.getAttribute("src")).toBe("photo-preview");
    expect(container.querySelector(".meta-file")).toBeNull();
  });

  it("activates adjacent files once, does not loop, and forwards Escape", async () => {
    const onfile = vi.fn();
    const onfilekeydown = vi.fn();
    render(ClipPreview, { props: {
      detail, selectedId: detail.id, page, noMatches: false, pending: false, assetUrl: null, thumbnailUrl: null,
      fileAccessDenied: false, sourceIconUrl: null, fileThumbnailUrls: [null, null],
      fileByteSizes: [null, null], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile,
      onfilekeydown, onfilefocus() {}, onopenorigin() {}, oninert() {},
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

  it("keeps first-copy and last-used times visible", () => {
    const { container } = render(ClipPreview, { props: {
      detail, selectedId: detail.id, page, noMatches: false, pending: false, assetUrl: null, thumbnailUrl: null,
      fileAccessDenied: false, sourceIconUrl: null, fileThumbnailUrls: [null, null],
      fileByteSizes: [null, null], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin() {}, oninert() {},
    } });

    expect(container.textContent).toContain("First copied");
    expect(container.textContent).toContain("Last used");
    expect(container.querySelector(".file-path")?.textContent).toBe("/tmp/one.txt");
    expect(container.querySelector(".meta-file")).toBeNull();
  });

  it("offers Full Disk Access when a file read is denied", async () => {
    const onfileaccess = vi.fn();
    render(ClipPreview, { props: {
      detail, selectedId: detail.id, page, noMatches: false, pending: false, assetUrl: null, thumbnailUrl: null,
      fileAccessDenied: true, sourceIconUrl: null, fileThumbnailUrls: [null, null],
      fileByteSizes: [null, null], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin() {}, onfileaccess, oninert() {},
    } });

    await fireEvent.click(screen.getByRole("button", { name: "Open Full Disk Access Settings" }));
    expect(onfileaccess).toHaveBeenCalledOnce();
  });

  it("renders a keyboard-accessible domain action and neutral remote source icon", async () => {
    const onopenorigin = vi.fn();
    const link: ClipDetail = {
      ...detail,
      id: "link",
      content_type: "link",
      preview: "https://docs.example.com/path",
      plain_text: "https://docs.example.com/path",
      source_app: { id: "com.apple.is-remote-clipboard", name: "Universal Clipboard" },
      metadata: { char_count: 29 },
    };
    const { container } = render(ClipPreview, { props: {
      detail: link, selectedId: link.id, page: { ...page, items: [link] }, noMatches: false, pending: false, assetUrl: null, thumbnailUrl: null,
      fileAccessDenied: false, sourceIconUrl: null, fileThumbnailUrls: [],
      fileByteSizes: [], fileIndex: 0, trimWhitespace: false,
      previousFileShortcut: "Command+ArrowLeft", nextFileShortcut: "Command+ArrowRight", onfile() {},
      onfilekeydown() {}, onfilefocus() {}, onopenorigin, oninert() {},
    } });

    const domain = container.querySelector<HTMLButtonElement>("button.domain");
    expect(domain?.textContent).toBe("docs.example.com");
    expect(domain?.getAttribute("aria-label")).toBe("Open docs.example.com");
    domain?.focus();
    expect(document.activeElement).toBe(domain);
    await fireEvent.click(domain!);
    expect(onopenorigin).toHaveBeenCalledTimes(1);
    expect(container.querySelector(".app-device")).toBeTruthy();
  });
});
