// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Layout from "./+layout.svelte";
import { setLanguagePreference } from "$lib/i18n/index.svelte";

const host = vi.hoisted(() => ({
  invoke: vi.fn(),
  getSettings: vi.fn(),
  schedule: vi.fn(() => () => {}),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: host.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => () => {}) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
vi.mock("$lib/updater/api", () => ({ scheduleAutomaticUpdateCheck: host.schedule }));
vi.mock("$lib/settings/api", () => ({ getSettings: host.getSettings, applyTheme: vi.fn(), THEME_PREVIEW_EVENT: "theme_preview" }));

const children = createRawSnippet(() => ({ render: () => "<p>History mounted</p>" }));

beforeEach(() => {
  vi.clearAllMocks();
  setLanguagePreference("en");
  host.getSettings.mockResolvedValue({ theme: "system", language: "en" });
});
afterEach(cleanup);

describe("startup database gate", () => {
  it("shows recovery actions without mounting history or loading settings and updates", async () => {
    host.invoke.mockResolvedValue({ kind: "too_new", app_version: "0.12.0", database_version: 12, required_version: 11 });
    render(Layout, { children });
    await screen.findByText(/Your database requires a newer version/);
    expect(screen.queryByText("History mounted")).toBeNull();
    expect(host.getSettings).not.toHaveBeenCalled();
    expect(host.schedule).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Open download page" }));
    expect(host.invoke).toHaveBeenCalledWith("open_website");
    host.invoke.mockRejectedValueOnce(new Error("failed"));
    await fireEvent.click(screen.getByRole("button", { name: "Open logs" }));
    await screen.findByRole("alert");
    await fireEvent.click(screen.getByRole("button", { name: "Quit" }));
    expect(host.invoke).toHaveBeenCalledWith("quit_app");
  });

  it("mounts normal content only after successful startup", async () => {
    host.invoke.mockResolvedValue(null);
    render(Layout, { children });
    await screen.findByText("History mounted");
    expect(host.getSettings).toHaveBeenCalledOnce();
    expect(host.schedule).toHaveBeenCalledOnce();
  });

  it("does not start normal services if startup status cannot be read", async () => {
    host.invoke.mockRejectedValue(new Error("unavailable"));
    render(Layout, { children });
    await waitFor(() => expect(screen.getByRole("heading").textContent).toContain("could not open"));
    expect(host.getSettings).not.toHaveBeenCalled();
    expect(host.schedule).not.toHaveBeenCalled();
  });
});
