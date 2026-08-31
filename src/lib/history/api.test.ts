import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { canPreviewClip, getPreviewCapability, getSourceAppIcon, openClipLink, previewClip, queryHistory } from "./api";

describe("history host contracts", () => {
  beforeEach(() => invoke.mockReset());

  it("scopes source icon lookup to a persisted clip id", async () => {
    invoke.mockResolvedValue({ data_url: null, byte_size: null });
    await getSourceAppIcon("clip-1");
    expect(invoke).toHaveBeenCalledWith("get_source_app_icon", { id: "clip-1" });
  });

  it("previews through one complete host action", async () => {
    invoke.mockResolvedValue("native_opened");
    await expect(previewClip("clip-1", 2)).resolves.toBe("native_opened");
    expect(invoke).toHaveBeenCalledWith("preview_clip", { id: "clip-1", index: 2 });
  });

  it("reads preview capability from the host", async () => {
    invoke.mockResolvedValue({ provider: "powertoys_peek", reason: null });
    await expect(getPreviewCapability()).resolves.toEqual({ provider: "powertoys_peek", reason: null });
    expect(invoke).toHaveBeenCalledWith("get_preview_capability");
  });

  it("limits PowerToys Peek to files without narrowing macOS Quick Look", () => {
    expect(canPreviewClip({ provider: "powertoys_peek", reason: null }, "file")).toBe(true);
    expect(canPreviewClip({ provider: "powertoys_peek", reason: null }, "text")).toBe(false);
    expect(canPreviewClip({ provider: "macos_quicklook", reason: null }, "text")).toBe(true);
    expect(canPreviewClip({ provider: "unavailable", reason: "not_installed" }, "file")).toBe(false);
  });

  it("asks Rust to open either the full stored link or its origin", async () => {
    invoke.mockResolvedValue(undefined);
    await openClipLink("clip-1");
    expect(invoke).toHaveBeenLastCalledWith("open_clip_link", { id: "clip-1", originOnly: false });
    await openClipLink("clip-1", true);
    expect(invoke).toHaveBeenLastCalledWith("open_clip_link", { id: "clip-1", originOnly: true });
  });

  it("keeps the default history page size and lets Quick request fewer rows", async () => {
    invoke.mockResolvedValue({ items: [], page: 1, page_size: 10, total: 0, total_pages: 1 });
    await queryHistory("", 2);
    expect(invoke).toHaveBeenLastCalledWith("query_history", {
      request: expect.objectContaining({ page: 2, page_size: 10 }),
    });
    await queryHistory("", 3, undefined, 7);
    expect(invoke).toHaveBeenLastCalledWith("query_history", {
      request: expect.objectContaining({ page: 3, page_size: 7 }),
    });
  });
});
