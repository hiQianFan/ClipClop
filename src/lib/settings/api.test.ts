// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, emit } = vi.hoisted(() => ({ invoke: vi.fn(), emit: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ emit }));

import { applyTheme, openFilePreviewSettings, previewTheme, THEME_PREVIEW_EVENT } from "./api";

describe("file preview settings IPC", () => {
  beforeEach(() => invoke.mockReset());

  it("uses the restricted settings commands", async () => {
    invoke.mockResolvedValue(undefined);
    await openFilePreviewSettings();
    expect(invoke).toHaveBeenLastCalledWith("open_file_preview_settings");
  });
});

describe("theme preview broadcast", () => {
  beforeEach(() => {
    emit.mockReset();
    emit.mockResolvedValue(undefined);
    delete document.documentElement.dataset.theme;
  });

  it("applies an explicit theme locally and mirrors it to other windows", () => {
    previewTheme("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(emit).toHaveBeenCalledWith(THEME_PREVIEW_EVENT, { theme: "dark" });
  });

  it("clears the override for system and still broadcasts", () => {
    applyTheme("light");
    previewTheme("system");
    expect(document.documentElement.dataset.theme).toBeUndefined();
    expect(emit).toHaveBeenCalledWith(THEME_PREVIEW_EVENT, { theme: "system" });
  });

  it("keeps the local preview when the broadcast fails", async () => {
    emit.mockRejectedValue(new Error("no permission"));
    expect(() => previewTheme("dark")).not.toThrow();
    await Promise.resolve();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
