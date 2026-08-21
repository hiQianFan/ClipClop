// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import OnboardingView from "./OnboardingView.svelte";
import type { OnboardingState } from "./api";

const initial: OnboardingState = {
  completed_revision: 1,
  current_step: null,
  visited_steps: [],
  selected_example: null,
};

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
