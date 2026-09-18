// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import HistoryActionBar from "./HistoryActionBar.svelte";

const props = {
  selected: true,
  canPreview: false,
  isLink: false,
  hasPlainText: false,
  isMac: true,
  error: "",
  permissionRecovery: false,
  menuOpen: true,
  deletePending: false,
  actionMenuShortcut: "Command+K",
  deleteShortcut: "Command+Backspace",
  onmenuopenchange() {},
  ondeleteopenchange() {},
  onbrowse() {},
  onpreview() {},
  onopenlink() {},
  onpasteplain() {},
  oncopy() {},
  oncopyplain() {},
  onrequestdelete() {},
  oncanceldelete() {},
  onconfirmdelete() {},
  onpaste() {},
  onpermission() {},
  onrestorefocus() {},
};

afterEach(async () => {
  cleanup();
  // Bits UI restores body scroll after a 24 ms transition.
  await new Promise((resolve) => setTimeout(resolve, 30));
});

describe("HistoryActionBar actions", () => {
  it("removes a favorite through the existing action menu", async () => {
    const onfavorite = vi.fn();
    render(HistoryActionBar, { props: { ...props, favorite: true, onfavorite } });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Remove from favorites" }));
    expect(onfavorite).toHaveBeenCalledOnce();
  });

  it("opens the action menu on click release rather than pointer down", async () => {
    const onmenuopenchange = vi.fn();
    render(HistoryActionBar, { props: { ...props, menuOpen: false, onmenuopenchange } });
    const trigger = screen.getByRole("button", { name: /Actions/ });

    await fireEvent.pointerDown(trigger, { button: 0, pointerType: "mouse" });
    expect(onmenuopenchange).not.toHaveBeenCalled();
    await fireEvent.click(trigger);
    expect(onmenuopenchange).toHaveBeenCalledWith(true);
  });

  it("gates preview, link, and plain-text actions from capabilities", () => {
    const hidden = render(HistoryActionBar, { props });
    expect(hidden.container.textContent).not.toContain("View selected content");
    expect(hidden.container.textContent).not.toContain("Open in default browser");
    expect(hidden.container.textContent).not.toContain("Paste as plain text");
    hidden.unmount();

    const visible = render(HistoryActionBar, { props: { ...props, canPreview: true, isLink: true, hasPlainText: true } });
    expect(visible.container.textContent).toContain("View selected content");
    expect(visible.container.textContent).toContain("Open in default browser");
    expect(visible.container.textContent).toContain("Paste as plain text");
  });

  it("opens a modal confirmation with delete focused and keeps the toolbar mounted", async () => {
    const view = render(HistoryActionBar, { props: { ...props, menuOpen: false, deletePending: true } });
    const dialog = screen.getByRole("alertdialog", { name: "Delete this item from ClipClop?" });
    expect(dialog.textContent).toContain("This will not delete the original file or change the system clipboard.");
    expect(view.container.contains(dialog)).toBe(false);
    expect(view.container.querySelector(".action-menu-trigger")).not.toBeNull();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("button", { name: "Delete" })));
    const cancel = screen.getByRole("button", { name: "Cancel" });
    const remove = screen.getByRole("button", { name: "Delete" });
    await fireEvent.keyDown(cancel, { key: "ArrowRight" });
    expect(document.activeElement).toBe(remove);
    await fireEvent.keyDown(remove, { key: "ArrowLeft" });
    expect(document.activeElement).toBe(cancel);
    await view.rerender({ ...props, menuOpen: false, deletePending: true, deleting: true });
    await fireEvent.keyDown(dialog, { key: "ArrowRight" });
    expect(document.activeElement).not.toBe(remove);
  });

  it("offers an action when automatic paste permission is missing", async () => {
    const onpermission = vi.fn();
    render(HistoryActionBar, { props: { ...props, menuOpen: false, error: "Copied", permissionRecovery: true, onpermission } });
    await fireEvent.click(screen.getByRole("button", { name: "Handle permission" }));
    expect(onpermission).toHaveBeenCalledOnce();
  });

  it("wires menu actions and returns the trigger as the delete invoker", async () => {
    const onpreview = vi.fn();
    const oncopy = vi.fn();
    const onrequestdelete = vi.fn();
    render(HistoryActionBar, { props: { ...props, canPreview: true, onpreview, oncopy, onrequestdelete } });
    await fireEvent.click(screen.getByRole("menuitem", { name: /View selected content/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Actions/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Copy to clipboard" }));
    await fireEvent.click(screen.getByRole("button", { name: /Actions/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /Delete from ClipClop/ }));
    expect(onpreview).toHaveBeenCalledOnce();
    expect(oncopy).toHaveBeenCalledOnce();
    expect(onrequestdelete).toHaveBeenCalledWith(screen.getByRole("button", { name: /Actions/ }));
  });

  it("wires deletion cancellation and confirmation", async () => {
    const oncanceldelete = vi.fn();
    const onconfirmdelete = vi.fn();
    const cancelView = render(HistoryActionBar, { props: { ...props, menuOpen: false, deletePending: true, oncanceldelete, onconfirmdelete } });
    await fireEvent.click(screen.getByRole("button", { name: /Cancel/ }));
    cancelView.unmount();
    render(HistoryActionBar, { props: { ...props, deletePending: true, oncanceldelete, onconfirmdelete } });
    await fireEvent.click(screen.getByRole("button", { name: /^Delete/ }));
    expect(oncanceldelete).toHaveBeenCalledOnce();
    expect(onconfirmdelete).toHaveBeenCalledOnce();
  });
});
