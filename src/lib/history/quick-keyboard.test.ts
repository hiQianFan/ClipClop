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

  it("pages explicitly and crosses page boundaries with arrows", () => {
    expect(routeQuickKey("ArrowRight", 4, 10, true, 1, 3)).toEqual({ type: "page", page: 2, edge: "first" });
    expect(routeQuickKey("ArrowLeft", 4, 10, true, 2, 3)).toEqual({ type: "page", page: 1, edge: "last" });
    expect(routeQuickKey("PageDown", 4, 10, true, 1, 3)).toEqual({ type: "page", page: 2, edge: "first" });
    expect(routeQuickKey("PageUp", 4, 10, true, 2, 3)).toEqual({ type: "page", page: 1, edge: "last" });
    expect(routeQuickKey("ArrowDown", 9, 10, true, 1, 3)).toEqual({ type: "page", page: 2, edge: "first" });
    expect(routeQuickKey("ArrowUp", 0, 10, true, 2, 3)).toEqual({ type: "page", page: 1, edge: "last" });
  });

  it("stays on the current item at the first and last page boundaries", () => {
    expect(routeQuickKey("ArrowLeft", 0, 10, true, 1, 3)).toBeNull();
    expect(routeQuickKey("ArrowRight", 2, 3, true, 3, 3)).toBeNull();
    expect(routeQuickKey("PageUp", 0, 10, true, 1, 3)).toBeNull();
    expect(routeQuickKey("PageDown", 2, 3, true, 3, 3)).toBeNull();
    expect(routeQuickKey("ArrowUp", 0, 10, true, 1, 3)).toEqual({ type: "select", index: 0 });
    expect(routeQuickKey("ArrowDown", 2, 3, true, 3, 3)).toEqual({ type: "select", index: 2 });
  });

  it("leaves Space untouched when preview is unavailable", () => {
    expect(routeQuickKey(" ", 1, 3, false)).toBeNull();
  });
});
