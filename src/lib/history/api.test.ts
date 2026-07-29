import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { getSourceAppIcon, previewClip } from "./api";

describe("history host contracts", () => {
  beforeEach(() => invoke.mockReset());

  it("scopes source icon lookup to a persisted clip id", async () => {
    invoke.mockResolvedValue({ data_url: null, byte_size: null });
    await getSourceAppIcon("clip-1");
    expect(invoke).toHaveBeenCalledWith("get_source_app_icon", { id: "clip-1" });
  });

  it("previews through one complete host action", async () => {
    invoke.mockResolvedValue("fallback_opened");
    await expect(previewClip("clip-1", 2)).resolves.toBe("fallback_opened");
    expect(invoke).toHaveBeenCalledWith("preview_clip", { id: "clip-1", index: 2 });
  });
});
