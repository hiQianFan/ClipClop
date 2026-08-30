import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
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

describe("HistoryActionBar actions", () => {
  it("gates preview, link, and plain-text actions from capabilities", () => {
    const hidden = render(HistoryActionBar, { props }).body;
    expect(hidden).not.toContain("View selected content");
    expect(hidden).not.toContain("Open in default browser");
    expect(hidden).not.toContain("Paste as plain text");

    const visible = render(HistoryActionBar, { props: { ...props, canPreview: true, isLink: true, hasPlainText: true } }).body;
    expect(visible).toContain("View selected content");
    expect(visible).toContain("Open in default browser");
    expect(visible).toContain("Paste as plain text");
  });

  it("replaces actions with the existing delete confirmation", () => {
    const body = render(HistoryActionBar, { props: { ...props, deletePending: true } }).body;
    expect(body).toContain("Delete this item from ClipClop?");
    expect(body).not.toContain("action-menu-trigger");
  });
});
