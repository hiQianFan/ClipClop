import { describe, expect, it } from "vitest";
import { compareVersions, shouldAutoCheck } from "./api";

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
