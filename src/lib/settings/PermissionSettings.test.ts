// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import PermissionSettings from "./PermissionSettings.svelte";

const { invoke, relaunch } = vi.hoisted(() => ({ invoke: vi.fn(), relaunch: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));
vi.mock("@tauri-apps/plugin-log", () => ({ info: vi.fn(), error: vi.fn() }));

const status = (value: "ready" | "permission_required") => ({ status: value, app_location: "applications", app_path: "/Applications/ClipClop.app" });

afterEach(() => { vi.useRealTimers(); cleanup(); invoke.mockReset(); relaunch.mockReset(); });

describe("macOS permission settings", () => {
  it("keeps the ready status actionable", async () => {
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status("ready")) : Promise.resolve());
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Ready" }));
    expect(invoke).toHaveBeenCalledWith("open_auto_paste_settings");
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("reports a ready permission revoked after returning", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "ready" : "permission_required")) : Promise.resolve());
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Ready" }));
    await fireEvent.focus(window);
    expect(await screen.findByText("The current ClipClop needs Accessibility permission.")).toBeTruthy();
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("rechecks a revoked permission on focus without opening settings first", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "ready" : "permission_required")) : Promise.resolve());
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    expect(await screen.findByRole("button", { name: "Ready" })).toBeTruthy();
    await fireEvent.focus(window);
    expect(await screen.findByRole("button", { name: "Grant Access" })).toBeTruthy();
    expect(checks).toBe(2);
  });

  it("retries a failed check", async () => {
    invoke.mockRejectedValueOnce(new Error("check failed")).mockResolvedValueOnce(status("ready"));
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Check Again" }));
    expect(await screen.findByRole("button", { name: "Ready" })).toBeTruthy();
  });

  it("stays waiting when focus returns without permission", async () => {
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status("permission_required")) : Promise.resolve());
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    await fireEvent.focus(window);
    expect(await screen.findByText("Allow the current ClipClop in System Settings, then return here.")).toBeTruthy();
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("relaunches once after a granted focus check", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    relaunch.mockResolvedValue(undefined);
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    vi.useFakeTimers();
    await fireEvent.focus(window); await Promise.resolve();
    await vi.advanceTimersByTimeAsync(4000);
    expect(relaunch).toHaveBeenCalledTimes(1);
    await fireEvent.focus(window);
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it("cancels the pending restart with Escape", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    vi.useFakeTimers(); await fireEvent.focus(window); await Promise.resolve();
    await fireEvent.keyDown(window, { key: "Escape" });
    await vi.advanceTimersByTimeAsync(4000);
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("cancels a pending restart when its tab becomes inactive", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    const view = render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    vi.useFakeTimers(); await fireEvent.focus(window); await Promise.resolve();
    expect(screen.getByRole("button", { name: "Restart Now" })).toBeTruthy();
    await view.rerender({ active: false, onerror() {} });
    await vi.advanceTimersByTimeAsync(4000);
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("rechecks when its tab becomes active again", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "ready" : "permission_required")) : Promise.resolve());
    const view = render(PermissionSettings, { props: { active: true, onerror() {} } });
    expect(await screen.findByRole("button", { name: "Ready" })).toBeTruthy();
    await view.rerender({ active: false, onerror() {} });
    await view.rerender({ active: true, onerror() {} });
    expect(await screen.findByRole("button", { name: "Grant Access" })).toBeTruthy();
    expect(checks).toBe(2);
  });

  it("exposes a manual restart after relaunch fails", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    relaunch.mockRejectedValueOnce(new Error("relaunch failed"));
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    await fireEvent.focus(window);
    await fireEvent.click(await screen.findByRole("button", { name: "Restart Now" }));
    expect(await screen.findByRole("button", { name: "Restart ClipClop" })).toBeTruthy();
    await fireEvent.focus(window);
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it("queues one focus recheck while checking", async () => {
    let resolvePending!: (value: ReturnType<typeof status>) => void; let checks = 0;
    invoke.mockImplementation((command: string) => {
      if (command !== "get_auto_paste_permission_status") return Promise.resolve();
      checks += 1;
      if (checks === 1) return Promise.resolve(status("permission_required"));
      if (checks === 2) return new Promise((resolve) => { resolvePending = resolve; });
      return Promise.resolve(status("permission_required"));
    });
    render(PermissionSettings, { props: { active: true, onerror() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Grant Access" }));
    await fireEvent.focus(window); await fireEvent.focus(window);
    resolvePending(status("permission_required"));
    await waitFor(() => expect(checks).toBe(3));
  });
});
