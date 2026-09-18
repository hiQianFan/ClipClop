// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import type { ReleasePage } from "$lib/updater/api";
const { load } = vi.hoisted(() => ({ load: vi.fn() }));
vi.mock("$lib/updater/api", () => ({ listReleaseNotes: load, openLatestRelease: vi.fn() }));
import ReleaseNotes from "./ReleaseNotes.svelte";
const page = (start: number, count: number, hasMore: boolean): ReleasePage => ({
  hasMore, releases: Array.from({ length: count }, (_, i) => ({ version: `v0.${start + i}.0`, publishedAt: "2026-09-17T00:00:00Z", notes: `Notes ${start + i}`, notesHtml: null, url: "", isLatest: start + i === 1 })),
});
afterEach(() => { cleanup(); load.mockReset(); vi.restoreAllMocks(); });

it("appends on scroll, preserves selection, and retries without losing loaded releases", async () => {
  let reject!: (reason: Error) => void;
  load.mockResolvedValueOnce(page(1, 10, true))
    .mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }))
    .mockResolvedValueOnce(page(10, 2, false));
  render(ReleaseNotes, { updateVersion: "v0.1.0", onerror() {} });
  expect(screen.getByRole("heading", { name: "Version History" })).toBeTruthy();
  const listbox = await screen.findByRole("listbox");
  const list = listbox.closest(".release-list-scroll") as HTMLElement;
  expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
  expect(load).toHaveBeenCalledTimes(1);
  Object.defineProperties(list, {
    scrollHeight: { value: 720, configurable: true },
    clientHeight: { value: 300, configurable: true },
    scrollTop: { value: 0, writable: true, configurable: true },
  });
  await fireEvent.scroll(list);
  expect(load).toHaveBeenCalledTimes(1);
  list.scrollTop = 420;
  const options = screen.getAllByRole("option");
  options.forEach((option) => { option.scrollIntoView = vi.fn(); });
  await fireEvent.click(options[2]);
  await screen.findByText("Notes 3");
  await fireEvent.scroll(list);
  await fireEvent.scroll(list);
  expect(load).toHaveBeenCalledTimes(2);
  expect(load).toHaveBeenLastCalledWith(2, false);
  const status = screen.getByRole("status");
  expect(status.textContent).toBe("Loading releases…");
  expect(status.parentElement).toBe(list);
  expect(status.previousElementSibling).toBe(listbox);
  expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
  expect(screen.queryByText("Loading settings…")).toBeNull();
  expect(screen.getByText("Notes 3")).toBeTruthy();
  reject(new Error("offline"));
  await screen.findByRole("alert");
  expect(screen.getAllByRole("option")).toHaveLength(10);
  expect(screen.getByText("Notes 3")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(11));
  expect(screen.getByText("Notes 3")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
  await fireEvent.scroll(list);
  expect(load).toHaveBeenCalledTimes(3);
});

it("ignores an old load-more response after a manual refresh", async () => {
  let resolve!: (result: ReleasePage) => void;
  load.mockResolvedValueOnce(page(1, 10, true))
    .mockImplementationOnce(() => new Promise((done) => { resolve = done; }))
    .mockResolvedValueOnce(page(20, 1, false));
  const view = render(ReleaseNotes, { onerror() {} });
  await fireEvent.scroll((await screen.findByRole("listbox")).closest(".release-list-scroll")!);
  await view.rerender({ refreshRevision: 1 });
  await screen.findByText("Notes 20");
  expect(load).toHaveBeenLastCalledWith(1, true);
  resolve(page(11, 10, true));
  await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(1));
  expect(screen.getByText("Notes 20")).toBeTruthy();
});

it("keeps the list skeleton beside the load failure and restores details while retrying", async () => {
  let resolve!: (result: ReleasePage) => void;
  load.mockRejectedValueOnce({ code: "RELEASE_UNAVAILABLE" })
    .mockImplementationOnce(() => new Promise((done) => { resolve = done; }));
  const view = render(ReleaseNotes, { onerror() {} });
  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("Version history is temporarily unavailable");
  expect(view.container.querySelector(".release-loading")).toBeNull();
  const browser = view.container.querySelector(".release-browser")!;
  const skeleton = browser.querySelector(".release-list")!;
  expect(skeleton.getAttribute("aria-hidden")).toBe("true");
  expect(skeleton.querySelectorAll(".release-skeleton-row")).toHaveLength(7);
  expect(browser.children[1]).toBe(alert.closest(".release-load-failed"));
  expect(browser.getAttribute("aria-busy")).toBe("false");
  await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(view.container.querySelector(".release-loading")).not.toBeNull();
  expect(view.container.querySelector(".release-list")).toBe(skeleton);
  expect(browser.querySelector(".release-skeleton-body")).not.toBeNull();
  expect(load).toHaveBeenLastCalledWith(1, true);
  resolve(page(1, 1, false));
  await screen.findByRole("listbox");
  expect(view.container.querySelector(".release-skeleton-row")).toBeNull();
});
