// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import ShortcutSettings from "./ShortcutSettings.svelte";
import { setLanguagePreference } from "$lib/i18n/index.svelte";
import type { Settings } from "./api";
const api = vi.hoisted(() => ({
  setHotkeyRecording: vi.fn(async (_recording: boolean) => {}),
  updateSettings: vi.fn(async (settings: Settings) => settings),
}));
vi.mock("./api", () => api);
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("focuses capture, displays modifiers, preserves reset as draft and keeps conflicts open", async () => {
  setLanguagePreference("en");
  const settings = { hotkey: "Control+Command+C" } as Settings;
  render(ShortcutSettings, { settings, platform: "macos" });
  const trigger = screen.getByRole("button", { name: /Current shortcut/ });
  await fireEvent.click(trigger);
  const input = await screen.findByRole("textbox", { name: "Press shortcut…" });
  await waitFor(() => expect(document.activeElement).toBe(input));
  expect(api.setHotkeyRecording).toHaveBeenCalledWith(true);
  await fireEvent.keyDown(input, { key: "Meta", metaKey: true });
  expect(screen.getByLabelText("Command")).toBeTruthy();
  await fireEvent.keyDown(input, { key: "d", code: "KeyD", ctrlKey: true, metaKey: true });
  expect(screen.getByLabelText("Control plus Command plus D")).toBeTruthy();
  await fireEvent.keyDown(input, { key: "c", code: "KeyC", ctrlKey: true, metaKey: true });
  expect(screen.getAllByLabelText("Control plus Command plus C").length).toBeGreaterThan(0);
  await fireEvent.click(screen.getByRole("button", { name: /Restore default/i }));
  expect(api.updateSettings).not.toHaveBeenCalled();
  await fireEvent.keyDown(input, { key: "d", code: "KeyD", ctrlKey: true, metaKey: true });
  api.updateSettings.mockRejectedValueOnce({ code: "HOTKEY_UNAVAILABLE" });
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(screen.getByRole("dialog")).toBeTruthy();
  expect(settings.hotkey).toBe("Control+Command+C");
  await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.setHotkeyRecording).toHaveBeenLastCalledWith(false);
});
