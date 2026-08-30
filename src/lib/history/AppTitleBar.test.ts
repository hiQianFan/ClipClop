// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import AppTitleBar from "./AppTitleBar.svelte";

describe("AppTitleBar menu", () => {
  it("keeps the existing application actions wired", async () => {
    const onsettings = vi.fn();
    const onupdates = vi.fn();
    const onabout = vi.fn();
    const onquit = vi.fn();
    for (const index of [0, 1, 2, 3]) {
      const view = render(AppTitleBar, { props: {
        history: true,
        open: true,
        settingsShortcut: "⌘,",
        quitShortcut: "⌘Q",
        onopenchange() {},
        onsettings,
        onupdates,
        onabout,
        onquit,
      } });
      await fireEvent.click(screen.getAllByRole("menuitem")[index]);
      view.unmount();
    }
    expect([onsettings, onupdates, onabout, onquit].map((callback) => callback.mock.calls.length)).toEqual([1, 1, 1, 1]);
  });
});
