import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { openFilePreviewSettings, setFilePreviewEnabled } from "./api";

describe("file preview settings IPC", () => {
  beforeEach(() => invoke.mockReset());

  it("uses the restricted settings commands", async () => {
    invoke.mockResolvedValue(undefined);
    await openFilePreviewSettings();
    expect(invoke).toHaveBeenLastCalledWith("open_file_preview_settings");
    await setFilePreviewEnabled(true);
    expect(invoke).toHaveBeenLastCalledWith("set_file_preview_enabled", { enabled: true });
  });
});
