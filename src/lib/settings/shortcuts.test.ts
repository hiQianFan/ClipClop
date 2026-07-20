import { describe, expect, it } from "vitest";
import { defaultShortcut, shortcutFromKeyboardEvent, shortcutKeycaps, shortcutSpokenLabel, validateShortcut } from "./shortcuts";
import { setLanguagePreference } from "$lib/i18n/index.svelte";

const keyEvent = (overrides: Partial<KeyboardEvent>) => ({
  altKey: false, code: "", ctrlKey: false, key: "", metaKey: false, shiftKey: false,
  ...overrides,
} as KeyboardEvent);

describe("shortcut recording", () => {
  it("records the platform modifiers in a stable format", () => {
    expect(shortcutFromKeyboardEvent(keyEvent({ code: "KeyD", key: "d", ctrlKey: true, metaKey: true }), "macos"))
      .toEqual({ valid: true, shortcut: "Control+Command+D" });
    expect(shortcutFromKeyboardEvent(keyEvent({ code: "KeyD", key: "d", ctrlKey: true, altKey: true }), "windows"))
      .toEqual({ valid: true, shortcut: "Ctrl+Alt+D" });
  });

  it("rejects modifier-only and unmodified input", () => {
    expect(shortcutFromKeyboardEvent(keyEvent({ key: "Control", ctrlKey: true }), "windows").valid).toBe(false);
    expect(shortcutFromKeyboardEvent(keyEvent({ code: "KeyC", key: "c" }), "windows").valid).toBe(false);
  });

  it("rejects reserved system combinations", () => {
    expect(validateShortcut("Command+C", "macos").valid).toBe(false);
    expect(validateShortcut("Command+Q", "macos").valid).toBe(false);
    expect(validateShortcut("Ctrl+V", "windows").valid).toBe(false);
    expect(validateShortcut("Super+V", "windows").valid).toBe(false);
    expect(validateShortcut("Alt+Space", "windows").valid).toBe(false);
    expect(validateShortcut("Command+Tab", "macos").valid).toBe(false);
    expect(validateShortcut("Control+Space", "macos").valid).toBe(false);
    expect(validateShortcut("Alt+Tab", "windows").valid).toBe(false);
    expect(validateShortcut("Super+L", "windows").valid).toBe(false);
  });

  it("rejects duplicate modifiers and unsupported main keys", () => {
    expect(validateShortcut("Ctrl+Ctrl+D", "windows").valid).toBe(false);
    expect(validateShortcut("Ctrl+?", "windows").valid).toBe(false);
    expect(validateShortcut("Ctrl+Shift", "windows").valid).toBe(false);
  });

  it("formats platform keycaps and defaults", () => {
    setLanguagePreference("zh-CN");
    expect(defaultShortcut("macos")).toBe("Control+Command+C");
    expect(defaultShortcut("windows")).toBe("Ctrl+Alt+C");
    expect(shortcutKeycaps("Control+Command+C", "macos")).toEqual(["⌃", "⌘", "C"]);
    expect(shortcutKeycaps("Ctrl+Alt+C", "windows")).toEqual(["Ctrl", "Alt", "C"]);
    expect(shortcutSpokenLabel("Control+Command+C", "macos")).toBe("Control 加 Command 加 C");
    expect(shortcutSpokenLabel("Ctrl+Shift+Enter", "windows")).toBe("Control 加 Shift 加 回车键");
  });
});
