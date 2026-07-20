import { describe, expect, it } from "vitest";
import { shouldAutoCheck } from "./api";

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
