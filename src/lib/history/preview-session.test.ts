import { afterEach, describe, expect, it, vi } from "vitest";
import { PreviewSession, type PreviewApi, type PreviewResource } from "./preview-session.svelte";
import type { ClipDetail, ClipSummary } from "./types";

const resource = (data_url: string | null, byte_size: number | null = null): PreviewResource => ({
  data_url,
  byte_size,
  access_denied: false,
});

function api(overrides: Partial<PreviewApi> = {}): PreviewApi {
  return {
    getClipAsset: vi.fn(async () => resource("image")),
    getClipFileAsset: vi.fn(async () => resource("file", 4)),
    getClipThumbnail: vi.fn(async () => resource("thumbnail")),
    getSourceAppIcon: vi.fn(async () => resource("icon")),
    ...overrides,
  };
}

const detail = (content_type: ClipDetail["content_type"]): ClipDetail => ({
  id: "clip-1",
  content_type,
  preview: "preview",
  source_app: { id: "app-1", name: "App" },
  created_at: "2026-08-06T00:00:00Z",
  byte_size: 4,
  metadata: content_type === "file" ? { files: ["/tmp/a"] } : {},
  plain_text: null,
  flavors: [],
});

afterEach(() => vi.useRealTimers());

describe("PreviewSession", () => {
  it("invalidates a pending asset before changing selection", async () => {
    vi.useFakeTimers();
    const preview = new PreviewSession(api());
    const pending = preview.loadSelection("clip-1", detail("image"), false);
    preview.resetSelection();
    await vi.runAllTimersAsync();
    await pending;
    expect(preview.assetUrl).toBeNull();
  });

  it("does not cache an in-flight asset after selection reset", async () => {
    vi.useFakeTimers();
    let resolveAsset!: (value: PreviewResource) => void;
    const getClipAsset = vi.fn(() => new Promise<PreviewResource>((resolve) => { resolveAsset = resolve; }));
    const preview = new PreviewSession(api({ getClipAsset }));
    const first = preview.loadSelection("clip-1", detail("image"), false);
    await vi.advanceTimersByTimeAsync(80);
    preview.resetSelection();
    resolveAsset(resource("stale"));
    await first;

    const second = preview.loadSelection("clip-1", detail("image"), false);
    await vi.advanceTimersByTimeAsync(80);
    expect(getClipAsset).toHaveBeenCalledTimes(2);
    resolveAsset(resource("fresh"));
    await second;
    expect(preview.assetUrl).toBe("fresh");
  });

  it("reports image failures but silently degrades file failures", async () => {
    vi.useFakeTimers();
    const failure = new Error("denied");
    const preview = new PreviewSession(api({
      getClipAsset: vi.fn(async () => { throw failure; }),
      getClipFileAsset: vi.fn(async () => { throw failure; }),
    }));

    const image = preview.loadSelection("clip-1", detail("image"), false);
    const imageResult = expect(image).rejects.toBe(failure);
    await vi.runAllTimersAsync();
    await imageResult;

    const file = preview.loadFile("clip-1", 0);
    await vi.runAllTimersAsync();
    await expect(file).resolves.toBeUndefined();
  });

  it("keeps only the latest selected file result", async () => {
    vi.useFakeTimers();
    const preview = new PreviewSession(api({
      getClipFileAsset: vi.fn(async (_id, index) => resource(`file-${index}`, index)),
    }));
    const first = preview.loadFile("clip-1", 0);
    const second = preview.loadFile("clip-1", 1);
    await vi.runAllTimersAsync();
    await Promise.all([first, second]);
    expect(preview.assetUrl).toBe("file-1");
    expect(preview.fileByteSizes[1]).toBe(1);
  });

  it("exposes file access denial separately from an unsupported preview", async () => {
    vi.useFakeTimers();
    const preview = new PreviewSession(api({
      getClipFileAsset: vi.fn(async () => ({ ...resource(null), access_denied: true })),
    }));
    const pending = preview.loadFile("clip-1", 0);
    await vi.runAllTimersAsync();
    await pending;
    expect(preview.fileAccessDenied).toBe(true);
  });

  it("invalidates old-page thumbnails before a refresh", async () => {
    let resolveThumbnail!: (value: PreviewResource) => void;
    const preview = new PreviewSession(api({
      getClipThumbnail: vi.fn(() => new Promise<PreviewResource>((resolve) => { resolveThumbnail = resolve; })),
    }));
    const items: ClipSummary[] = [{
      id: "clip-1",
      content_type: "image",
      preview: "preview",
      source_app: null,
      created_at: "2026-08-06T00:00:00Z",
      byte_size: 4,
      metadata: {},
    }];
    const pending = preview.loadPageThumbnails(items);
    await Promise.resolve();
    preview.resetPage();
    resolveThumbnail(resource("old-page"));
    await pending;
    expect(preview.thumbnailUrls).toEqual({});
  });
});
