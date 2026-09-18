import { describe, expect, it, vi } from "vitest";
import { compareVersions, isTransientNetworkError, shouldAutoCheck, listReleaseNotes } from "./api";

const NOW = Date.parse("2026-07-19T12:00:00Z");

describe("shouldAutoCheck", () => {
  it("checks when enabled and no previous check exists", () => {
    expect(shouldAutoCheck(true, null, NOW)).toBe(true);
  });

  it("does not check when the preference is disabled", () => {
    expect(shouldAutoCheck(false, null, NOW)).toBe(false);
  });

  it("throttles successful checks for 24 hours", () => {
    expect(shouldAutoCheck(true, "2026-07-19T11:00:00Z", NOW)).toBe(false);
    expect(shouldAutoCheck(true, "2026-07-18T12:00:00Z", NOW)).toBe(true);
  });

  it("recovers from an invalid persisted timestamp", () => {
    expect(shouldAutoCheck(true, "invalid", NOW)).toBe(true);
  });
});

describe("compareVersions", () => {
  it("orders newer versions above older ones", () => {
    expect(compareVersions("0.1.2", "0.1.1")).toBeGreaterThan(0);
    expect(compareVersions("0.2.0", "0.1.9")).toBeGreaterThan(0);
    expect(compareVersions("1.0.0", "0.9.9")).toBeGreaterThan(0);
  });

  it("orders older versions below newer ones (the downgrade case)", () => {
    // The bug: on 0.1.2 a stale cache advertised 0.1.1 as available.
    expect(compareVersions("0.1.1", "0.1.2")).toBeLessThan(0);
  });

  it("treats identical versions as equal", () => {
    expect(compareVersions("0.1.2", "0.1.2")).toBe(0);
  });

  it("returns 0 (cannot confirm newer) for unparseable input", () => {
    expect(compareVersions("__clipclop_unknown__", "0.1.2")).toBe(0);
    expect(compareVersions("0.1.2", "not.a.version")).toBe(0);
  });
});

describe("isTransientNetworkError", () => {
  it("retries the real-world GitHub CDN failures we observed", () => {
    // Both messages came straight from the on-device diagnostic log.
    expect(isTransientNetworkError(new Error("error sending request for url (https://…)"))).toBe(true);
    expect(isTransientNetworkError(new Error("error decoding response body"))).toBe(true);
    expect(isTransientNetworkError(new Error("operation timed out"))).toBe(true);
    expect(isTransientNetworkError("connection reset by peer")).toBe(true);
  });

  it("does not retry terminal control-flow errors", () => {
    expect(isTransientNetworkError({ code: "UPDATE_CHANGED" })).toBe(false);
    expect(isTransientNetworkError({ code: "UPDATE_UNSUPPORTED" })).toBe(false);
  });

  it("does not retry signature verification failures", () => {
    expect(isTransientNetworkError(new Error("signature verification failed"))).toBe(false);
    expect(isTransientNetworkError(new Error("failed to verify the update signature"))).toBe(false);
  });

  it("does not retry unknown non-network errors", () => {
    expect(isTransientNetworkError(new Error("permission denied"))).toBe(false);
  });
});

it("loads releases once, slices local pages, retries failures, and resets the cache on refresh", async () => {
  const releases = Array.from({ length: 12 }, (_, index) => ({
    version: `v1.${index}.0`, publishedAt: "2026-09-17T00:00:00Z", notes: `Notes ${index}`, notesHtml: null, url: `https://github.com/hiQianFan/ClipClop/releases/tag/v1.${index}.0`, isLatest: index === 0,
  }));
  const fetcher = vi.fn()
    .mockResolvedValueOnce(new Response(JSON.stringify({ schemaVersion: 1, releases })))
    .mockRejectedValueOnce(new Error("offline"))
    .mockResolvedValueOnce(new Response(JSON.stringify({ schemaVersion: 1, releases: releases.slice(0, 1) })));
  vi.stubGlobal("fetch", fetcher);
  try {
    const first = await listReleaseNotes(1, true);
    expect(fetcher.mock.calls[0][0]).toBe("https://clipclop.io/releases.json");
    expect(first.hasMore).toBe(true);
    expect(first.releases[0].isLatest).toBe(true);
    expect(await listReleaseNotes(1)).toBe(first);
    expect(fetcher).toHaveBeenCalledTimes(1);
    const last = await listReleaseNotes(2);
    expect(last.hasMore).toBe(false);
    expect(last.releases).toHaveLength(2);
    expect(await listReleaseNotes(2)).toBe(last);
    await expect(listReleaseNotes(1, true)).rejects.toThrow("offline");
    expect(await listReleaseNotes(1)).toEqual({ releases: [releases[0]], hasMore: false });
    expect(fetcher).toHaveBeenCalledTimes(3);
  } finally { vi.unstubAllGlobals(); }
});

it("rejects invalid release feeds as unavailable", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({ schemaVersion: 2, releases: [] }))));
  try {
    await expect(listReleaseNotes(1, true)).rejects.toMatchObject({ code: "RELEASE_UNAVAILABLE" });
  } finally { vi.unstubAllGlobals(); }
});

it("rejects releases outside the website feed contract", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({ schemaVersion: 1, releases: [
    { version: "v1.0.0", publishedAt: "invalid", notes: "", url: "https://github.com/hiQianFan/ClipClop/releases/tag/v1.0.0" },
  ] }))));
  try {
    await expect(listReleaseNotes(1, true)).rejects.toMatchObject({ code: "RELEASE_UNAVAILABLE" });
  } finally { vi.unstubAllGlobals(); }
});

it("falls back to Markdown when release HTML is unsafe", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({ schemaVersion: 1, releases: [
    { version: "v1.0.0", publishedAt: "2026-09-17T00:00:00Z", notes: "Safe text", notesHtml: "<img src=x onerror=alert(1)>", url: "https://github.com/hiQianFan/ClipClop/releases/tag/v1.0.0" },
  ] }))));
  try {
    const page = await listReleaseNotes(1, true);
    expect(page.releases[0].notesHtml).toBeNull();
  } finally { vi.unstubAllGlobals(); }
});
