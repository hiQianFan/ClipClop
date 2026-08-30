import { describe, expect, it } from "vitest";
import { exitsSearch, routeWindowKey, type WindowKeyContext } from "./keyboard";

const browse: WindowKeyContext = { view: "history", mode: "browse", deletePending: false, menuOpen: false, appMenuOpen: false };
const key = (value: string, extra = {}) => ({ key: value, ctrlKey: false, metaKey: false, defaultPrevented: false, ...extra });

describe("window keyboard priority", () => {
  it("keeps window dismissal independent of DOM focus context", () => {
    expect(routeWindowKey(key("w", { metaKey: true }), { ...browse, view: "settings" })).toBe("dismiss-panel");
  });

  it("pops one Escape layer before dismissing the panel", () => {
    expect(routeWindowKey(key("Escape"), { ...browse, deletePending: true })).toBe("cancel-delete");
    expect(routeWindowKey(key("Escape"), { ...browse, menuOpen: true })).toBe("close-menu");
    expect(routeWindowKey(key("Escape"), { ...browse, appMenuOpen: true })).toBe("close-app-menu");
    expect(routeWindowKey(key("Escape"), { ...browse, mode: "search" })).toBe("return-to-browse");
    expect(routeWindowKey(key("Escape"), browse)).toBe("dismiss-panel");
  });

  it("keeps delete confirmation ahead of menus and window commands ahead of every layer", () => {
    const layered = { ...browse, deletePending: true, menuOpen: true, appMenuOpen: true };
    expect(routeWindowKey(key("Escape"), layered)).toBe("cancel-delete");
    expect(routeWindowKey(key("w", { ctrlKey: true }), layered)).toBe("dismiss-panel");
  });

  it("does not override handled events, child views, or directional keys", () => {
    expect(routeWindowKey(key("Escape", { defaultPrevented: true }), browse)).toBeNull();
    expect(routeWindowKey(key("Escape"), { ...browse, view: "settings" })).toBeNull();
    expect(routeWindowKey(key("ArrowDown"), browse)).toBeNull();
  });
});

describe("search keyboard routing", () => {
  it("enters result navigation with vertical arrows without stealing text editing keys", () => {
    expect(exitsSearch("ArrowDown")).toBe(true);
    expect(exitsSearch("ArrowUp")).toBe(true);
    expect(exitsSearch("ArrowLeft")).toBe(false);
    expect(exitsSearch("ArrowRight")).toBe(false);
  });
});
