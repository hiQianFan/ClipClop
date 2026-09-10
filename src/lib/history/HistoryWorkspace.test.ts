// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ClipDetail, ClipSummary, HistoryPage } from "./types";

const host = vi.hoisted(() => ({
  restoreBrowsePosition: true,
  settingsError: false,
  queryHistory: vi.fn(),
  getClipPage: vi.fn(),
  setClipFavorite: vi.fn(),
  pasteClip: vi.fn(),
  platform: "windows" as "windows" | "macos",
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
}));

const settings = () => ({
  retention_days: 30, history_limit: 500, move_used_to_top: true,
  restore_browse_position: host.restoreBrowsePosition, preserve_search_conditions: false,
  trim_whitespace: true, file_preview_enabled: false, launch_at_login: false,
  hotkey: "Control+Shift+C", theme: "system", language: "en",
  tray_click_action: "recent", check_updates: true, last_update_check: null,
  skipped_update_version: null,
});

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async (name: string, callback: (event: { payload: unknown }) => void) => { host.listeners.set(name, callback); return () => host.listeners.delete(name); }) }));
vi.mock("./api", () => ({
  canPreviewClip: () => false,
  setClipFavorite: host.setClipFavorite,
  copyClip: vi.fn(), deleteClip: vi.fn(), getClip: vi.fn(async (id: string) => detail(id)),
  getClipAsset: vi.fn(), getClipFileAsset: vi.fn(), getClipThumbnail: vi.fn(async () => ({ data_url: null, byte_size: null, access_denied: false, is_directory: false })),
  getHistoryFacets: vi.fn(async () => ({ type_total: 0, type_counts: {}, sources: [] })),
  getPreviewCapability: vi.fn(async () => ({ provider: "unavailable", reason: "not_installed", version: null })),
  getRuntimeMode: vi.fn(async () => "real"), enterDemoMode: vi.fn(async () => "demo"), exitDemoMode: vi.fn(async () => "real"),
  getSourceAppIcon: vi.fn(), hidePanel: vi.fn(), openClipLink: vi.fn(), pasteClip: host.pasteClip,
  performPagerHaptic: vi.fn(), previewClip: vi.fn(), queryHistory: host.queryHistory, getClipPage: host.getClipPage,
}));
vi.mock("$lib/settings/api", () => ({
  getSettings: vi.fn(async () => {
    if (host.settingsError) throw new Error("settings unavailable");
    return settings();
  }), updateSettings: vi.fn(async () => settings()),
  applyTheme: vi.fn(), previewTheme: vi.fn(), openFilePreviewSettings: vi.fn(),
  openLogDir: vi.fn(), openQuicklookInstallPage: vi.fn(), openRepository: vi.fn(), quitApp: vi.fn(),
}));
vi.mock("$lib/settings/shortcuts", async (importOriginal) => ({
  ...await importOriginal<typeof import("$lib/settings/shortcuts")>(), currentPlatform: () => host.platform,
}));
vi.mock("$lib/onboarding/api", () => ({
  getOnboardingState: vi.fn(async () => ({ completed_revision: 1, current_step: "overview", visited_steps: ["overview"], selected_example: "image" })),
  getAutoPastePermissionStatus: vi.fn(async () => ({ status: "ready", app_location: "applications", app_path: "/Applications/ClipClop.app" })),
  openAutoPasteSettings: vi.fn(), revealCurrentApp: vi.fn(), shouldRestartAfterPermissionCheck: vi.fn(() => false),
}));
vi.mock("$lib/updater/api", () => ({ DEVELOPMENT_VERSION: "0.0.0-dev", listReleaseNotes: vi.fn(async () => []), openLatestRelease: vi.fn() }));
vi.mock("$lib/updater/store.svelte", () => ({
  updateStore: { appVersion: "0.8.6", update: null, phase: "idle", progress: null, busy: false, errorSource: null, displayStatus: null, skippedVersion: null, hydrate: vi.fn(), check: vi.fn(), download: vi.fn(), cancel: vi.fn(), install: vi.fn(), skip: vi.fn(), retry: vi.fn() },
}));

import HistoryWorkspace from "./HistoryWorkspace.svelte";

const item = (id: string): ClipSummary => ({ id, content_type: "text", preview: id, source_app: null, created_at: "2026-01-01T00:00:00Z", last_used_at: "2026-01-01T00:00:00Z", byte_size: 1, metadata: {} });
const detail = (id: string): ClipDetail => ({ ...item(id), plain_text: id, flavors: [] });
const page = (number: number): HistoryPage => ({ items: [item(number === 1 ? "latest" : "older")], page: number, page_size: 10, total: 11, total_pages: 2 });

beforeEach(() => {
  window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
  Element.prototype.getAnimations = vi.fn(() => []);
  host.restoreBrowsePosition = true;
  host.settingsError = false;
  host.platform = "windows";
  host.listeners.clear();
  host.pasteClip.mockReset();
  host.getClipPage.mockReset().mockResolvedValue(1);
  host.setClipFavorite.mockReset().mockResolvedValue(undefined);
  host.queryHistory.mockReset().mockImplementation(async (_query, target) => page(target));
});
afterEach(cleanup);

describe("closing settings", () => {
  it.each([
    [false, "latest", [1]],
    [true, "older", []],
  ])("restore position %s selects %s", async (restore, selected, closeRequests) => {
    render(HistoryWorkspace);
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("latest"));
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("older"));
    host.queryHistory.mockClear();
    host.restoreBrowsePosition = restore;
    await fireEvent.keyDown(screen.getByRole("listbox"), { key: ",", ctrlKey: true });
    await screen.findByRole("heading", { name: "General" });
    await fireEvent.click(screen.getByRole("button", { name: "Back" }));
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain(selected));
    expect(host.queryHistory.mock.calls.map((call) => call[1])).toEqual(closeRequests);
  });

  it("routes a permission request into mounted settings without clearing selection", async () => {
    host.platform = "macos";
    render(HistoryWorkspace);
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("latest"));
    await fireEvent.keyDown(screen.getByRole("listbox"), { key: ",", metaKey: true });
    await screen.findByRole("heading", { name: "General" });
    host.listeners.get("main_panel_shown")?.({ payload: { selectedId: "latest", settings: false, permissionGuide: true } });
    expect(await screen.findByRole("heading", { name: "Permissions & System Access" })).toBeTruthy();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("button", { name: "Ready" })));
    await fireEvent.click(screen.getByRole("button", { name: "Back" }));
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("latest"));
  });

  it("keeps the current page when settings cannot be refreshed", async () => {
    render(HistoryWorkspace);
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("latest"));
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("older"));
    await fireEvent.keyDown(screen.getByRole("listbox"), { key: ",", ctrlKey: true });
    await screen.findByRole("heading", { name: "General" });
    host.settingsError = true;
    host.queryHistory.mockClear();
    await fireEvent.click(screen.getByRole("button", { name: "Back" }));
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("older"));
    expect(host.queryHistory).not.toHaveBeenCalled();
  });
});

describe("favorites", () => {
  it("switches scope, keeps history position, and removes the selected favorite without pasting", async () => {
    let saved = false;
    host.setClipFavorite.mockImplementation(async (_id, value) => { saved = value; });
    host.queryHistory.mockImplementation(async (_query, target, filters) => ({
      ...page(target),
      items: filters.favorites_only ? (saved ? [{ ...item("older"), is_favorite: true }] : []) : [{ ...item(target === 1 ? "latest" : "older"), is_favorite: saved }],
      total: filters.favorites_only ? Number(saved) : 11,
      total_pages: filters.favorites_only ? Number(saved) : 2,
    }));
    render(HistoryWorkspace);
    await screen.findByRole("option");
    await fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await waitFor(() => expect(screen.getByRole("option").textContent).toContain("older"));
    await fireEvent.click(screen.getByRole("button", { name: "Add to favorites" }));
    await waitFor(() => expect(host.setClipFavorite).toHaveBeenCalledWith("older", true));
    await screen.findByRole("button", { name: "Remove from favorites" });
    await fireEvent.click(screen.getByRole("button", { name: "Favorites" }));
    await waitFor(() => expect(screen.getByRole("listbox").getAttribute("aria-busy")).toBe("false"));
    expect(screen.getByPlaceholderText("Search favorites…")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "All" }));
    await waitFor(() => expect(screen.getByRole("option").getAttribute("aria-posinset")).toBe("11"));
    await fireEvent.click(screen.getByRole("button", { name: "Favorites" }));
    await waitFor(() => expect(screen.getByRole("listbox").getAttribute("aria-busy")).toBe("false"));
    await fireEvent.click(screen.getByRole("button", { name: "Remove from favorites" }));
    await screen.findByText("No favorites yet");
    expect(host.setClipFavorite).toHaveBeenLastCalledWith("older", false);
    expect(host.pasteClip).not.toHaveBeenCalled();
  });

  it("panel routes from favorites to the target item page in all history", async () => {
    host.queryHistory.mockImplementation(async (_query, target, filters) => ({
      ...page(target),
      items: filters.favorites_only ? [{ ...item("favorite"), is_favorite: true }] : [item(target === 1 ? "latest" : "older")],
    }));
    host.getClipPage.mockResolvedValue(2);
    render(HistoryWorkspace);
    await screen.findByRole("option");
    await fireEvent.click(screen.getByRole("button", { name: "Favorites" }));
    await waitFor(() => expect(screen.getByRole("option").textContent).toContain("favorite"));
    host.listeners.get("main_panel_shown")?.({ payload: { selectedId: "older", settings: false, permissionGuide: false } });
    await waitFor(() => expect(screen.getByRole("option", { selected: true }).textContent).toContain("older"));
    expect(screen.getByRole("button", { name: "All" }).getAttribute("aria-pressed")).toBe("true");
    expect(host.getClipPage).toHaveBeenCalledWith("older");
  });
});
