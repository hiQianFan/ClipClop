// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import OnboardingView from "./OnboardingView.svelte";
import type { OnboardingState } from "./api";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const initial: OnboardingState = {
  completed_revision: 1,
  current_step: null,
  visited_steps: [],
  selected_example: null,
};

beforeEach(() => {
  invoke.mockImplementation((command: string) => {
    if (command === "get_preview_capability") {
      return Promise.resolve({ provider: "unavailable", reason: "not_installed", version: null });
    }
    return Promise.resolve(undefined);
  });
});

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("Onboarding language menu", () => {
  it("preserves special open focus, resets click focus, and lets Tab leave", async () => {
    render(OnboardingView, { props: { initial, mode: "quick_start", onfinish() {} } });
    const trigger = screen.getByRole("button", { name: /language/i });

    trigger.focus();
    await fireEvent.keyDown(trigger, { key: "ArrowUp" });
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("menuitemradio", { name: "English" })));
    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
    await waitFor(() => expect(document.activeElement).toBe(trigger));

    await fireEvent.click(trigger);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("menuitemradio", { name: "Follow system" })));
    await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(document.activeElement).not.toBe(trigger);
  });
});

describe("Onboarding practice", () => {
  it("practices the same keyboard paging used by the history list", async () => {
    render(OnboardingView, { props: { initial, mode: "quick_start", onfinish() {} } });
    await fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(screen.queryByRole("listbox", { name: "Clipboard practice sandbox" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    expect(screen.queryByRole("listbox", { name: "Clipboard practice sandbox" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    const sandbox = await screen.findByRole("listbox", { name: "Clipboard practice sandbox" });

    await waitFor(() => expect(document.activeElement).toBe(sandbox));
    await fireEvent.click(screen.getByRole("heading", { name: "Try the core workflow" }));
    expect(document.activeElement).toBe(sandbox);
    expect(screen.getByText("1/3")).toBeTruthy();
    await fireEvent.keyDown(sandbox, { key: "ArrowRight" });
    expect(screen.getByText("2/3")).toBeTruthy();
    expect(screen.getByText("Second-page example")).toBeTruthy();
    await fireEvent.keyDown(sandbox, { key: "PageDown" });
    expect(screen.getByText("3/3")).toBeTruthy();
    expect(screen.getByText("Third-page example")).toBeTruthy();
  });

  it("offers optional QuickLook setup before the Windows practice step", async () => {
    render(OnboardingView, { props: { initial, mode: "quick_start", onfinish() {} } });
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));

    expect(await screen.findByRole("heading", { name: "Get ready" })).toBeTruthy();
    expect(screen.getByText(/Optional — ClipClop still saves/)).toBeTruthy();
    await fireEvent.click(await screen.findByRole("button", { name: "Install QuickLook" }));
    expect(invoke).toHaveBeenCalledWith("open_quicklook_install_page");
    expect(screen.getByRole("button", { name: "Next step" })).toBeTruthy();
  });

  it("does not offer Space during practice when QuickLook was skipped", async () => {
    render(OnboardingView, { props: { initial, mode: "quick_start", onfinish() {} } });
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    await screen.findByRole("button", { name: "Install QuickLook" });
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    const sandbox = await screen.findByRole("listbox", { name: "Clipboard practice sandbox" });
    await fireEvent.keyDown(sandbox, { key: " ", code: "Space" });
    expect(screen.queryByText("Preview")).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("preview_onboarding_example", expect.anything());
    expect(screen.getByRole("button", { name: "Finish" })).toBeTruthy();
  });

  it("offers Space during practice when QuickLook is ready", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "get_preview_capability") {
        return Promise.resolve({ provider: "quicklook", reason: null, version: "4.5.0" });
      }
      if (command === "preview_onboarding_example") return Promise.resolve("native_opened");
      return Promise.resolve(undefined);
    });
    render(OnboardingView, { props: { initial, mode: "quick_start", onfinish() {} } });
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    await screen.findByText("Press Space to open or close system preview.");
    await fireEvent.click(screen.getByRole("button", { name: "Next step" }));
    const sandbox = await screen.findByRole("listbox", { name: "Clipboard practice sandbox" });
    expect(screen.getByText("Preview")).toBeTruthy();
    await fireEvent.keyDown(sandbox, { key: " ", code: "Space" });
    expect(invoke).toHaveBeenCalledWith("preview_onboarding_example", { example: "image", open: true });
  });
});
