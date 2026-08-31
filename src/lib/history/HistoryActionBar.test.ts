// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import HistoryActionBar from "./HistoryActionBar.svelte";

const props = {
  selected: true,
  canPreview: false,
  isLink: false,
  hasPlainText: false,
  isMac: true,
  error: "",
  copied: "",
  menuOpen: true,
  deletePending: false,
  actionMenuShortcut: "⌘K",
  deleteShortcut: "⌘⌫",
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
  onrestorefocus() {},
};

afterEach(cleanup);

describe("HistoryActionBar actions", () => {
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

  it("replaces actions with the existing delete confirmation", () => {
    const view = render(HistoryActionBar, { props: { ...props, deletePending: true } });
    expect(view.container.textContent).toContain("Delete this item from ClipClop?");
    expect(view.container.querySelector(".action-menu-trigger")).toBeNull();
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
});
