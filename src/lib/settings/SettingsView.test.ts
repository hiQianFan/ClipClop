// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

const { preview, updateSettings, listReleaseNotes, openUrl, platform } = vi.hoisted(() => ({
  preview: {
    phase: "idle",
    progress: null as number | null,
    errorSource: null as null | "download" | "install" | "relaunch",
    displayStatus: null as null | "current" | "available" | "skipped",
    lastUpdateCheck: null as string | null,
  },
  updateSettings: vi.fn(async (settings) => settings),
  listReleaseNotes: vi.fn(async () => [{
    version: "0.7.3", publishedAt: "2026-08-30T00:00:00Z", notes: "Changes", notesHtml: null, isLatest: true,
  }]),
  openUrl: vi.fn(),
  platform: { value: "windows" as "windows" | "macos" },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl }));

vi.mock("./api", () => ({
  getSettings: async () => ({
    retention_days: 30, history_limit: 1000, move_used_to_top: true,
    restore_browse_position: true, preserve_search_conditions: true,
    trim_whitespace: false, file_preview_enabled: false, launch_at_login: false,
    hotkey: "Control+Shift+V", theme: "system", language: "en",
    tray_click_action: "recent", check_updates: true, last_update_check: preview.lastUpdateCheck,
    skipped_update_version: null,
  }),
  updateSettings, applyTheme: vi.fn(), previewTheme: vi.fn(),
  openFilePreviewSettings: vi.fn(), openLogDir: vi.fn(), performHaptic: vi.fn(),
}));
vi.mock("$lib/history/api", () => ({
  clearHistory: vi.fn(),
  getPreviewCapability: async () => ({ provider: "unavailable", reason: "not_installed" }),
}));
vi.mock("$lib/onboarding/api", () => ({ openAutoPasteSettings: vi.fn() }));
vi.mock("./shortcuts", async (importOriginal) => ({
  ...await importOriginal<typeof import("./shortcuts")>(),
  currentPlatform: () => platform.value,
}));
vi.mock("$lib/updater/api", () => ({
  DEVELOPMENT_VERSION: "0.0.0-dev",
  listReleaseNotes,
  openLatestRelease: vi.fn(),
}));
vi.mock("$lib/updater/store.svelte", () => ({
  updateStore: {
    get appVersion() { return "0.7.2"; },
    get update() { return { version: "0.7.3", currentVersion: "0.7.2", date: null, notes: "" }; },
    get phase() { return preview.phase; },
    get progress() { return preview.progress; },
    get busy() { return preview.phase === "downloading" || preview.phase === "installing"; },
    get errorSource() { return preview.errorSource; },
    get displayStatus() { return preview.displayStatus; },
    get skippedVersion() { return null; },
    hydrate: vi.fn(), check: vi.fn(), download: vi.fn(), cancel: vi.fn(),
    install: vi.fn(), skip: vi.fn(), retry: vi.fn(),
  },
}));

import SettingsView from "./SettingsView.svelte";

afterEach(() => {
  platform.value = "windows";
  cleanup();
});

it("keeps loading and saving owned by SettingsView across categories", async () => {
  updateSettings.mockClear();
  render(SettingsView, { props: { onclose() {}, oncleared() {}, onquickstart() {} } });
  await waitFor(() => expect(screen.getByRole("heading", { name: "General" })).toBeTruthy());
  await fireEvent.click(screen.getByRole("tab", { name: "Shortcuts" }));
  await waitFor(() => expect(screen.getByRole("heading", { name: "Shortcuts" })).toBeTruthy());
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(updateSettings).toHaveBeenCalledTimes(1));
  expect(screen.getByRole("button", { name: "Saved" })).toBeTruthy();
});

it("keeps release notes mounted while switching categories", async () => {
  listReleaseNotes.mockClear();
  render(SettingsView, { props: { initialTab: "updates", onclose() {}, oncleared() {}, onquickstart() {} } });
  await waitFor(() => expect(screen.getByText("Changes")).toBeTruthy());
  await fireEvent.click(screen.getByRole("tab", { name: "General" }));
  await fireEvent.click(screen.getByRole("tab", { name: "Software Update" }));
  await waitFor(() => expect(screen.getByText("Changes")).toBeTruthy());
  expect(listReleaseNotes).toHaveBeenCalledTimes(1);
});

it("rolls editable settings back when saving fails", async () => {
  updateSettings.mockRejectedValueOnce(new Error("disk full"));
  render(SettingsView, { props: { onclose() {}, oncleared() {}, onquickstart() {} } });
  const launch = await screen.findByRole("switch", { name: "Launch at login" }) as HTMLInputElement;
  await fireEvent.click(launch);
  expect(launch.checked).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect((screen.getByRole("switch", { name: "Launch at login" }) as HTMLInputElement).checked).toBe(false));
});

it("keeps shortcut recording validation, cancellation, defaults, and saving wired", async () => {
  updateSettings.mockClear();
  render(SettingsView, { props: { initialTab: "shortcuts", onclose() {}, oncleared() {}, onquickstart() {} } });
  await screen.findByRole("heading", { name: "Shortcuts" });
  const change = screen.getByRole("button", { name: "Change" });
  await fireEvent.click(change);
  await fireEvent.keyDown(change, { key: "v", code: "KeyV", ctrlKey: true });
  expect(screen.getByRole("alert").textContent).toContain("common system or window shortcut");
  await fireEvent.keyDown(change, { key: "Escape" });
  expect(screen.getByText("Shortcut recording cancelled")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Restore default" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(updateSettings).toHaveBeenCalledWith(expect.objectContaining({ hotkey: "Ctrl+Alt+C" })));
});

it("renders the platform-specific preview entry", async () => {
  render(SettingsView, { props: { onclose() {}, oncleared() {}, onquickstart() {} } });
  expect(await screen.findByRole("button", { name: "Learn and install" })).toBeTruthy();
  cleanup();
  platform.value = "macos";
  render(SettingsView, { props: { onclose() {}, oncleared() {}, onquickstart() {} } });
  await screen.findByRole("heading", { name: "General" });
  expect(screen.getAllByRole("button", { name: "Manage" })).toHaveLength(2);
  expect(screen.queryByRole("button", { name: "Learn and install" })).toBeNull();
});

it("opens the repository from the GitHub icon", async () => {
  openUrl.mockClear();
  render(SettingsView, { props: { initialTab: "about", onclose() {}, oncleared() {}, onquickstart() {} } });
  await fireEvent.click(await screen.findByRole("button", { name: "View ClipClop on GitHub" }));
  expect(openUrl).toHaveBeenCalledWith("https://github.com/hiQianFan/ClipClop");
});

async function show(
  phase: string,
  { progress = null, errorSource = null, displayStatus = null, lastUpdateCheck = null }: {
    progress?: number | null;
    errorSource?: null | "download" | "install" | "relaunch";
    displayStatus?: null | "current" | "available" | "skipped";
    lastUpdateCheck?: string | null;
  } = {},
) {
  preview.phase = phase;
  preview.progress = progress;
  preview.errorSource = errorSource;
  preview.displayStatus = displayStatus;
  preview.lastUpdateCheck = lastUpdateCheck;
  const view = render(SettingsView, { props: { initialTab: "updates", onclose() {}, oncleared() {}, onquickstart() {} } });
  await waitFor(() => expect(screen.getByRole("heading", { name: "Software Update" })).toBeTruthy());
  expect(view.container.querySelectorAll(".update-rail")).toHaveLength(1);
}

describe("software update status rail", () => {
  it("shows skip and download when an update is available", async () => {
    await show("idle");
    expect(screen.getByRole("button", { name: "Skip this version" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Download" })).toBeTruthy();
  });

  it("shows progress and only cancel while downloading", async () => {
    await show("downloading", { progress: 42 });
    expect(screen.getByRole("progressbar").getAttribute("aria-valuetext")).toBe("42%");
    expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Skip this version" })).toBeNull();
  });

  it("keeps skip beside install after download", async () => {
    await show("downloaded", { progress: 100 });
    expect(screen.getByRole("button", { name: "Skip this version" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Install and restart" })).toBeTruthy();
  });

  it("only offers restart after a relaunch failure", async () => {
    await show("error", { errorSource: "relaunch" });
    expect(screen.getByText("Could not restart")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Restart" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Skip this version" })).toBeNull();
  });

  it("shows the last check time when current", async () => {
    await show("current", { lastUpdateCheck: "2026-08-30T14:32:00+08:00" });
    expect(screen.getAllByText("You're on the latest version").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Last checked:/).length).toBeGreaterThan(0);
  });

  it("keeps the confirmed status while checking and disables refresh", async () => {
    await show("checking", { displayStatus: "current" });
    expect(screen.getAllByText("You're on the latest version").length).toBeGreaterThan(0);
    expect(screen.getByText("Checking for updates…")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Check for updates" }).hasAttribute("disabled")).toBe(true);
  });

  it("supports downloading with unknown progress", async () => {
    await show("downloading");
    expect(screen.getByRole("progressbar").getAttribute("aria-valuetext")).toBe("Downloading update…");
    expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
  });

  for (const [source, title, detail] of [
    ["download", "Download failed", "Check your network connection"],
    ["install", "Installation failed", "The downloaded package is still available"],
  ] as const) {
    it(`shows recovery actions and limits danger styling for ${source} errors`, async () => {
      await show("error", { errorSource: source });
      const heading = screen.getByText(title);
      const explanation = screen.getByText(detail);
      expect(heading.classList.contains("error")).toBe(true);
      expect(explanation.classList.contains("error")).toBe(false);
      expect(screen.getByRole("button", { name: "Skip this version" })).toBeTruthy();
      expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy();
    });
  }
});
