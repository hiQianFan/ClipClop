import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import AppSelect from "./AppSelect.svelte";

describe("AppSelect", () => {
  it("renders the selected label and accessible trigger", () => {
    const { body } = render(AppSelect, {
      props: {
        value: "quick",
        ariaLabel: "Tray action",
        items: [
          { value: "quick", label: "Open quick panel" },
          { value: "main", label: "Open main window" },
        ],
        onchange() {},
      },
    });
    expect(body).toContain('aria-label="Tray action"');
    expect(body).toContain("Open quick panel");
  });
});
