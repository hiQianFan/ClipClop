// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import AppTitleBar from "./AppTitleBar.svelte";

describe("AppTitleBar menu", () => {
  it("opens on click release rather than pointer down", async () => {
    const onopenchange = vi.fn();
    const view = render(AppTitleBar, { props: {
      history: true,
      open: false,
      settingsShortcut: "Command+,",
      quitShortcut: "Command+Q",
      onopenchange,
      onsettings() {},
      onupdates() {},
      onabout() {},
      onquit() {},
    } });
    const trigger = screen.getByRole("button", { name: /application menu/i });

    await fireEvent.pointerDown(trigger, { button: 0, pointerType: "mouse" });
    expect(onopenchange).not.toHaveBeenCalled();
    await fireEvent.click(trigger);
    expect(onopenchange).toHaveBeenCalledWith(true);
    view.unmount();
  });

  it("keeps the existing application actions wired", async () => {
    const onsettings = vi.fn();
    const onupdates = vi.fn();
    const onabout = vi.fn();
    const onquit = vi.fn();
    for (const index of [0, 1, 2, 3]) {
      const view = render(AppTitleBar, { props: {
        history: true,
        open: true,
        settingsShortcut: "Command+,",
        quitShortcut: "Command+Q",
        onopenchange() {},
        onsettings,
        onupdates,
        onabout,
        onquit,
      } });
      await fireEvent.click(screen.getAllByRole("menuitem")[index]);
      view.unmount();
    }
    // Keep jsdom alive for Bits UI's 24 ms body-scroll cleanup.
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect([onsettings, onupdates, onabout, onquit].map((callback) => callback.mock.calls.length)).toEqual([1, 1, 1, 1]);
  });
});
