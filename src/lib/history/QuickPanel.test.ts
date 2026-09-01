// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ClipSummary, HistoryPage } from "./types";

const host = vi.hoisted(() => ({
  queryHistory: vi.fn(),
  previewClip: vi.fn(),
  listeners: new Map<string, () => void>(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, callback: () => void) => {
    host.listeners.set(event, callback);
    return () => host.listeners.delete(event);
  }),
}));
vi.mock("./api", () => ({
  canPreviewClip: () => true,
  getClipThumbnail: vi.fn(async () => ({ data_url: null, byte_size: null, access_denied: false })),
  getPreviewCapability: vi.fn(async () => ({ provider: "unavailable", reason: "not_installed" })),
  hidePanel: vi.fn(),
  pasteClip: vi.fn(),
  previewClip: host.previewClip,
  performPagerHaptic: vi.fn(),
  queryHistory: host.queryHistory,
  setQuickSelection: vi.fn(),
}));
vi.mock("$lib/settings/api", () => ({ quitApp: vi.fn() }));

import QuickPanel from "./QuickPanel.svelte";

function item(index: number): ClipSummary {
  return {
    id: `clip-${index}`,
    content_type: "text",
    preview: `item ${index}`,
    source_app: null,
    created_at: "2026-01-01T00:00:00Z",
    last_used_at: "2026-01-01T00:00:00Z",
    byte_size: 6,
    metadata: {},
  };
}

function page(number: number, count: number, total: number): HistoryPage {
  return {
    items: Array.from({ length: count }, (_, index) => item((number - 1) * 10 + index + 1)),
    page: number,
    page_size: 10,
    total,
    total_pages: Math.max(1, Math.ceil(total / 10)),
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

function show() {
  return render(QuickPanel, { props: { onfull() {}, onsettings() {} } });
}

beforeEach(() => {
  host.queryHistory.mockReset();
  host.previewClip.mockReset();
  host.listeners.clear();
});

afterEach(cleanup);

describe("QuickPanel pagination", () => {
  it.each([[1, 9], [10, 0]])("keeps ten slots for a %i-item single page", async (count, empty) => {
    host.queryHistory.mockResolvedValue(page(1, count, count));
    const view = show();
    await screen.findByText("item 1");
    expect(view.container.querySelectorAll(".quick-item")).toHaveLength(count);
    expect(view.container.querySelectorAll(".empty-slot")).toHaveLength(empty);
    expect(view.container.querySelector(".navigation")?.children).toHaveLength(4);
    expect(view.container.querySelector(".navigation > .scrubber")).toBeTruthy();
    expect(screen.getByText("1/1")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Previous page" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Next page" }).hasAttribute("disabled")).toBe(true);
  });

  it("pages through 23 items and leaves seven inert slots on the last page", async () => {
    host.queryHistory.mockImplementation(async (_query, target) => target === 1 ? page(1, 10, 23) : target === 2 ? page(2, 10, 23) : page(3, 3, 23));
    const view = show();
    await screen.findByText("item 1");
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    expect(view.container.querySelector(".ticks")?.classList.contains("key-visible")).toBe(true);
    await screen.findByText("item 11");
    expect(screen.getByText("2/3")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await screen.findByText("item 21");
    expect(view.container.querySelectorAll(".empty-slot")).toHaveLength(7);
    expect(screen.getByText("3/3")).toBeTruthy();
  });

  it("uses the shared scrubber to jump multiple pages and selects the target first item", async () => {
    host.queryHistory.mockImplementation(async (_query, target) => page(target, target === 4 ? 7 : 10, 37));
    const view = show();
    await screen.findByText("item 1");
    await fireEvent.wheel(view.container.querySelector(".scrubber")!, { deltaX: 144, deltaY: 0 });
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 31"));
    expect(host.queryHistory.mock.calls.slice(-3).map((call) => call[1])).toEqual([2, 3, 4]);
    expect(screen.getByText("4/4")).toBeTruthy();
  });

  it("supports PageDown and arrow-key page crossing with the requested edge selected", async () => {
    host.queryHistory.mockImplementation(async (_query, target) => target === 1 ? page(1, 10, 11) : page(2, 1, 11));
    const view = show();
    const list = await screen.findByRole("listbox");
    await screen.findByText("item 1");
    expect((screen.getByRole("option", { selected: true }) as HTMLElement).tabIndex).toBe(-1);
    await fireEvent.keyDown(list, { key: "PageDown" });
    expect(view.container.querySelector(".ticks")?.classList.contains("key-visible")).toBe(true);
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 11"));
    await fireEvent.keyDown(list, { key: "ArrowUp" });
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 10"));
  });

  it("returns row clicks to the listbox so keyboard selection owns the focus", async () => {
    host.queryHistory.mockResolvedValue(page(1, 10, 10));
    show();
    const list = await screen.findByRole("listbox");
    const second = await screen.findByText("item 2");
    await fireEvent.click(second);
    expect(document.activeElement).toBe(list);
    await fireEvent.keyDown(list, { key: "ArrowDown" });
    expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 3");
    await fireEvent.blur(window);
    expect(document.activeElement).toBe(document.body);
    await fireEvent.focus(window);
    expect(document.activeElement).toBe(list);
  });

  it("returns pointer-operated buttons to the list but preserves explicit keyboard focus", async () => {
    host.queryHistory.mockResolvedValue(page(1, 10, 10));
    const view = show();
    const list = await screen.findByRole("listbox");
    await screen.findByText("item 1");
    const settings = view.container.querySelector("nav button:nth-child(2)") as HTMLButtonElement;

    await fireEvent.pointerUp(settings);
    await new Promise(requestAnimationFrame);
    expect(document.activeElement).toBe(list);

    settings.focus();
    await fireEvent.keyDown(settings, { key: "ArrowDown" });
    expect(document.activeElement).toBe(settings);
    expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 1");
    expect(host.previewClip).not.toHaveBeenCalled();
  });

  it("matches the app left and right page shortcuts", async () => {
    host.queryHistory.mockImplementation(async (_query, target) => target === 1 ? page(1, 10, 11) : page(2, 1, 11));
    show();
    const list = await screen.findByRole("listbox");
    await screen.findByText("item 1");
    await fireEvent.keyDown(list, { key: "ArrowRight" });
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 11"));
    await fireEvent.keyDown(list, { key: "ArrowLeft" });
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("item 10"));
  });

  it("keeps the committed page when the next page fails", async () => {
    host.queryHistory.mockResolvedValueOnce(page(1, 10, 11)).mockRejectedValueOnce({ code: "STORAGE_ERROR" });
    show();
    await screen.findByText("item 1");
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await screen.findByRole("alert");
    expect(screen.getByText("item 1")).toBeTruthy();
    expect(screen.getByText("1/2")).toBeTruthy();
  });

  it("commits only the latest refresh and returns to page one when shown", async () => {
    const older = deferred<HistoryPage>();
    const newer = deferred<HistoryPage>();
    host.queryHistory.mockResolvedValueOnce(page(1, 10, 23));
    show();
    await screen.findByText("item 1");
    host.queryHistory.mockReturnValueOnce(older.promise).mockReturnValueOnce(newer.promise);
    host.listeners.get("history_changed")?.();
    host.listeners.get("quick_panel_shown")?.();
    newer.resolve({ ...page(1, 1, 1), items: [{ ...item(30), preview: "newest" }] });
    await screen.findByText("newest");
    older.resolve({ ...page(1, 1, 1), items: [{ ...item(29), preview: "stale" }] });
    await Promise.resolve();
    expect(screen.queryByText("stale")).toBeNull();
    expect(host.queryHistory).toHaveBeenLastCalledWith("", 1, undefined, 10);
  });
});
