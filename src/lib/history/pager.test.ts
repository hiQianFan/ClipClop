import { describe, expect, it } from "vitest";
import { draggedPage, visiblePageTicks } from "./pager";

describe("pager drag", () => {
  it("turns every 48px into one page without a gesture cap", () => {
    expect(draggedPage(6, -47, 20)).toBe(6);
    expect(draggedPage(6, -480, 20)).toBe(16);
    expect(draggedPage(6, 144, 20)).toBe(3);
    expect(draggedPage(6, 480, 20)).toBe(1);
  });

  it("shows at most three real pages on either side", () => {
    expect(visiblePageTicks(1, 13)).toEqual([1, 2, 3, 4]);
    expect(visiblePageTicks(7, 13)).toEqual([4, 5, 6, 7, 8, 9, 10]);
    expect(visiblePageTicks(13, 13)).toEqual([10, 11, 12, 13]);
    expect(visiblePageTicks(1, 0)).toEqual([]);
  });
});
