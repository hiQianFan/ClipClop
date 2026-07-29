import { describe, expect, it } from "vitest";
import { escapeAction } from "./interaction";

describe("history interaction modes", () => {
  it("pops exactly one explicit Escape layer", () => {
    expect(escapeAction("confirmation")).toBe("cancel-confirmation");
    expect(escapeAction("menu")).toBe("close-menu");
    expect(escapeAction("search")).toBe("exit-to-browse");
    expect(escapeAction("file-tablist")).toBe("exit-to-browse");
    expect(escapeAction("browse")).toBe("hide-panel");
  });
});
