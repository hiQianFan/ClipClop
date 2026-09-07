// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import PermissionSettings from "./PermissionSettings.svelte";
const { invoke, relaunch } = vi.hoisted(() => ({ invoke: vi.fn(), relaunch: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));
const status = (value = "permission_required") => ({ status: value, app_location: "applications", app_path: "/Applications/ClipClop.app" });
afterEach(() => { cleanup(); vi.useRealTimers(); invoke.mockReset(); relaunch.mockReset(); localStorage.clear(); });
function setup(beforeRestart = async () => true) {
  invoke.mockImplementation((command: string) => Promise.resolve(command === "get_auto_paste_permission_status" ? status() : undefined));
  return render(PermissionSettings, { active: true, onerror: vi.fn(), beforeRestart });
}
it("allows explicit restart even when the process still reports required", async () => {
  setup();
  await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
  expect(invoke).toHaveBeenCalledWith("open_permission_guide", { kind: "accessibility" });
  await fireEvent.focus(window);
  await fireEvent.click(screen.getByRole("button", { name: "Refresh status" }));
  expect(relaunch).not.toHaveBeenCalled();
  expect(screen.queryByText("/Applications/ClipClop.app")).toBeNull();
  await fireEvent.click(screen.getByRole("button", { name: "Restart App" }));
  expect(relaunch).toHaveBeenCalledTimes(1);
  expect(localStorage.getItem("permission-restart-return")).toBe("settings");
});
it("does not restart on focus or after a timer", async () => {
  setup();
  await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
  vi.useFakeTimers();
  await fireEvent.focus(window);
  await vi.advanceTimersByTimeAsync(5000);
  expect(relaunch).not.toHaveBeenCalled();
});
it("honors cancellation before restarting", async () => {
  setup(async () => false);
  await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
  await fireEvent.click(screen.getByRole("button", { name: "Restart App" }));
  expect(relaunch).not.toHaveBeenCalled();
  expect(localStorage.getItem("permission-restart-return")).toBeNull();
});
it("allows retry after restart fails", async () => {
  setup(); relaunch.mockRejectedValueOnce(new Error("failed"));
  await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
  await fireEvent.click(screen.getByRole("button", { name: "Restart App" }));
  await fireEvent.click(screen.getByRole("button", { name: "Restart App" }));
  expect(relaunch).toHaveBeenCalledTimes(2);
});
it("ignores an in-flight result after deactivation", async () => {
  let resolve!: (result: ReturnType<typeof status>) => void;
  invoke.mockImplementation(() => new Promise((done) => { resolve = done; }));
  const view = render(PermissionSettings, { active: true, onerror() {} });
  await view.rerender({ active: false, onerror() {} });
  resolve(status("ready"));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Ready" })).toBeNull());
});
