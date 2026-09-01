// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import OnboardingView from "./OnboardingView.svelte";
import type { OnboardingState } from "./api";

const initial: OnboardingState = {
  completed_revision: 1,
  current_step: null,
  visited_steps: [],
  selected_example: null,
};

afterEach(cleanup);

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
});
