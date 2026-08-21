// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import Harness from "./BitsInteractionHarness.test.svelte";

describe("Bits UI interaction contracts", () => {
  it("provides managed menu, tabs, dialog focus, and progress semantics", async () => {
    render(Harness);

    const menuTrigger = screen.getByRole("button", { name: "Actions" });
    await fireEvent.click(menuTrigger);
    expect(screen.getByRole("menuitem", { name: "Copy" })).toBeTruthy();
    await fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    await waitFor(() => expect(document.activeElement).toBe(menuTrigger));
    await waitFor(() => expect(document.body.style.overflow).toBe(""));

    const first = screen.getByRole("tab", { name: "First" });
    first.focus();
    await fireEvent.keyDown(first, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Second" }).getAttribute("aria-selected")).toBe("true");

    const dialogTrigger = screen.getByRole("button", { name: "Clear" });
    dialogTrigger.focus();
    await fireEvent.click(dialogTrigger);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.hasAttribute("data-inline-confirmation")).toBe(true);
    expect(document.body.style.overflow).toBe("");
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Confirm" }));
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(document.activeElement).toBe(dialogTrigger));

    const progress = screen.getByRole("progressbar", { name: "Download progress" });
    expect(progress.getAttribute("aria-valuenow")).toBe("40");
    expect(screen.getByRole("progressbar", { name: "Preparing download" }).hasAttribute("aria-valuenow")).toBe(false);
  });
});
