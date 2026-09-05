// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import OnboardingView from "./OnboardingView.svelte";

const { invoke, relaunch } = vi.hoisted(() => ({ invoke: vi.fn(), relaunch: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));
vi.mock("@tauri-apps/plugin-log", () => ({ info: vi.fn(), error: vi.fn() }));
vi.mock("$lib/settings/shortcuts", async (load) => ({ ...(await load()), currentPlatform: () => "macos" }));

const initial = { completed_revision: 1, current_step: "auto_paste" as const, visited_steps: ["auto_paste" as const], selected_example: null };

afterEach(() => {
  vi.useRealTimers();
  cleanup();
  invoke.mockReset();
  relaunch.mockReset();
});

describe("macOS Accessibility recovery", () => {
  function status(value: "ready" | "permission_required") {
    return { status: value, app_location: "applications", app_path: "/Applications/ClipClop.app" };
  }

  it("does not request or restart when permission is initially ready", async () => {
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status("ready")) : Promise.resolve());
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    expect(await screen.findByText("Accessibility permission is ready.")).toBeTruthy();
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("reports permission revoked after a ready user returns from settings", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "ready" : "permission_required")) : Promise.resolve());
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Manage" }));
    await fireEvent.focus(window);
    expect(await screen.findByText("The current ClipClop needs Accessibility permission.")).toBeTruthy();
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("shows unknown and retries after a failed permission query", async () => {
    invoke.mockRejectedValueOnce(new Error("check failed")).mockResolvedValueOnce(status("ready"));
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    expect(await screen.findByText("ClipClop could not check Accessibility permission. Try again.")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Check again" }));
    expect(await screen.findByText("Accessibility permission is ready.")).toBeTruthy();
  });

  it("stays in the guide when focus returns without permission", async () => {
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status("permission_required")) : Promise.resolve());
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Open Accessibility Settings" }));
    await fireEvent.focus(window);
    expect(await screen.findByText("Allow the current ClipClop in System Settings, then return here.")).toBeTruthy();
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("runs the promised automatic restart once after the timer", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    relaunch.mockResolvedValue(undefined);
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Open Accessibility Settings" }));
    vi.useFakeTimers();
    await fireEvent.focus(window);
    await Promise.resolve();
    await vi.advanceTimersByTimeAsync(4000);
    expect(relaunch).toHaveBeenCalledTimes(1);
    await fireEvent.focus(window);
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it("cancels the automatic restart with Escape", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Open Accessibility Settings" }));
    vi.useFakeTimers();
    await fireEvent.focus(window);
    await Promise.resolve();
    expect(screen.getByRole("button", { name: "Cancel Restart" })).toBeTruthy();
    await fireEvent.keyDown(window, { key: "Escape" });
    await vi.advanceTimersByTimeAsync(4000);
    expect(relaunch).not.toHaveBeenCalled();
    expect(screen.getByText(/Automatic restart cancelled/)).toBeTruthy();
  });

  it("executes the pending restart before finishing the guide", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => command === "get_auto_paste_permission_status" ? Promise.resolve(status(++checks === 1 ? "permission_required" : "ready")) : Promise.resolve());
    relaunch.mockResolvedValue(undefined);
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Open Accessibility Settings" }));
    await fireEvent.focus(window);
    await screen.findByRole("button", { name: "Restart Now" });
    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));
    expect(relaunch).toHaveBeenCalledTimes(1);
  });

  it("queues one focus recheck while a check is pending", async () => {
    let resolvePending!: (value: ReturnType<typeof status>) => void;
    let checks = 0;
    invoke.mockImplementation((command: string) => {
      if (command !== "get_auto_paste_permission_status") return Promise.resolve();
      checks += 1;
      if (checks === 1) return Promise.resolve(status("permission_required"));
      if (checks === 2) return new Promise((resolve) => { resolvePending = resolve; });
      return Promise.resolve(status("permission_required"));
    });
    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    await fireEvent.click(await screen.findByRole("button", { name: "Open Accessibility Settings" }));
    await fireEvent.focus(window);
    await fireEvent.focus(window);
    resolvePending(status("permission_required"));
    await waitFor(() => expect(checks).toBe(3));
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("rechecks on focus, attempts one relaunch, and exposes failure recovery", async () => {
    let checks = 0;
    invoke.mockImplementation((command: string) => {
      if (command === "get_auto_paste_permission_status") {
        checks += 1;
        return Promise.resolve(status(checks === 1 ? "permission_required" : "ready"));
      }
      return Promise.resolve(undefined);
    });
    relaunch.mockRejectedValueOnce(new Error("relaunch failed"));

    render(OnboardingView, { props: { initial, mode: "auto_paste_recovery", onfinish() {} } });
    const open = await screen.findByRole("button", { name: "Open Accessibility Settings" });
    await fireEvent.click(open);
    expect(invoke).toHaveBeenCalledWith("open_auto_paste_settings");

    await fireEvent.focus(window);
    const restart = await screen.findByRole("button", { name: "Restart Now" });
    await fireEvent.click(restart);
    await waitFor(() => expect(relaunch).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("button", { name: "Restart ClipClop" })).toBeTruthy();

    await fireEvent.focus(window);
    expect(relaunch).toHaveBeenCalledTimes(1);
  });
});
