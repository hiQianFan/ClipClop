import { describe, expect, it } from "vitest";

import worker, { handleDownload } from "./download-worker.js";

describe("download worker", () => {
  it.each([
    ["macos", "/releases/v0.8.0/ClipClop_0.8.0_universal.dmg"],
    ["windows", "/releases/v0.8.0/ClipClop_0.8.0_windows_x64-setup.exe"],
  ])("redirects %s to its versioned release", async (platform, target) => {
    const response = await handleDownload(
      new Request(`https://clipclop.mapin.net/download/${platform}`),
      async () => Response.json({ [platform]: target }),
    );

    expect(response.status).toBe(302);
    expect(response.headers.get("location")).toBe(`https://clipclop.mapin.net${target}`);
  });

  it("fails closed when metadata contains an external target", async () => {
    const response = await handleDownload(
      new Request("https://clipclop.mapin.net/download/macos"),
      async () => Response.json({ macos: "https://example.com/file.dmg" }),
    );

    expect(response.status).toBe(503);
  });

  it("does not treat Cloudflare's env argument as the metadata fetcher", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async () => Response.json({ macos: "/releases/v0.8.0/ClipClop.dmg" });
    try {
      const response = await worker.fetch(
        new Request("https://clipclop.mapin.net/download/macos"),
        { bindings: true },
      );
      expect(response.status).toBe(302);
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});
