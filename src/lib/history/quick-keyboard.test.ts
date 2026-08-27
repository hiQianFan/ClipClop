import { describe, expect, it } from "vitest";
import { routeQuickKey } from "./quick-keyboard";

describe("quick panel keyboard", () => {
  it("keeps the same 1–9,0 mapping as full history", () => {
    expect(routeQuickKey("1", 0, 10)).toEqual({ type: "copy", index: 0 });
    expect(routeQuickKey("0", 0, 10)).toEqual({ type: "copy", index: 9 });
  });

  it("moves within visible items and previews the selection with Space", () => {
    expect(routeQuickKey("ArrowUp", 0, 3)).toEqual({ type: "select", index: 0 });
    expect(routeQuickKey("ArrowDown", 2, 3)).toEqual({ type: "select", index: 2 });
    expect(routeQuickKey(" ", 1, 3)).toEqual({ type: "preview", index: 1 });
  });
});
