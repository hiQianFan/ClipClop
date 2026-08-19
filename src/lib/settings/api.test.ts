import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { openFilePreviewSettings } from "./api";

describe("file preview settings IPC", () => {
  beforeEach(() => invoke.mockReset());

  it("uses the restricted settings commands", async () => {
    invoke.mockResolvedValue(undefined);
    await openFilePreviewSettings();
    expect(invoke).toHaveBeenLastCalledWith("open_file_preview_settings");
  });
});
