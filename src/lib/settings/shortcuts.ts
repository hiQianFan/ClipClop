export type ShortcutPlatform = "macos" | "windows";

export type ShortcutValidation =
  | { valid: true; shortcut: string }
  | { valid: false; message: string };

const modifierKeys = new Set(["Alt", "Control", "Meta", "Shift"]);
const supportedNamedKeys = new Map([
  ["ArrowUp", "ArrowUp"], ["ArrowDown", "ArrowDown"],
  ["ArrowLeft", "ArrowLeft"], ["ArrowRight", "ArrowRight"],
  ["Backspace", "Backspace"], ["Delete", "Delete"],
  ["Home", "Home"], ["End", "End"],
  ["PageUp", "PageUp"], ["PageDown", "PageDown"],
  ["Enter", "Enter"], ["Escape", "Escape"], ["Tab", "Tab"],
  [" ", "Space"], ["Spacebar", "Space"],
]);

export const defaultShortcut = (platform: ShortcutPlatform) =>
  platform === "macos" ? "Control+Command+C" : "Ctrl+Alt+C";

export function currentPlatform(): ShortcutPlatform {
  if (typeof navigator !== "undefined") {
    const value = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
    if (value.includes("mac")) return "macos";
  }
  return "windows";
}

function mainKey(event: Pick<KeyboardEvent, "key" | "code">): string | null {
  if (modifierKeys.has(event.key)) return null;
  if (/^Key[A-Z]$/.test(event.code)) return event.code.slice(3);
  if (/^Digit[0-9]$/.test(event.code)) return event.code.slice(5);
  if (/^F(?:[1-9]|1[0-2])$/.test(event.key)) return event.key;
  if (event.key.length === 1 && /[a-z0-9]/i.test(event.key)) return event.key.toUpperCase();
  return supportedNamedKeys.get(event.key) ?? null;
}

export function shortcutFromKeyboardEvent(
  event: Pick<KeyboardEvent, "altKey" | "code" | "ctrlKey" | "key" | "metaKey" | "shiftKey">,
  platform: ShortcutPlatform,
): ShortcutValidation {
  const key = mainKey(event);
  if (!key) return { valid: false, message: "请同时按下修饰键和一个字母、数字或功能键。" };

  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push(platform === "macos" ? "Control" : "Ctrl");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  if (event.metaKey) modifiers.push(platform === "macos" ? "Command" : "Super");
  return validateShortcut([...modifiers, key].join("+"), platform);
}

export function validateShortcut(shortcut: string, platform: ShortcutPlatform): ShortcutValidation {
  const parts = shortcut.split("+").map((part) => part.trim());
  const key = parts.at(-1) ?? "";
  const modifierParts = parts.slice(0, -1);
  const modifiers = new Set(modifierParts);
  const accepted = platform === "macos"
    ? new Set(["Control", "Alt", "Shift", "Command"])
    : new Set(["Ctrl", "Alt", "Shift", "Super"]);

  if (!supportedMainKey(key) || modifiers.size === 0 || modifiers.size !== modifierParts.length
    || [...modifiers].some((part) => !accepted.has(part))) {
    return { valid: false, message: "快捷键必须包含至少一个修饰键和一个主键。" };
  }

  if (isReservedShortcut(modifiers, key, platform)) {
    return { valid: false, message: "该组合是常用系统或窗口快捷键，请选择其他组合。" };
  }
  return { valid: true, shortcut: [...parts.slice(0, -1), key].join("+") };
}

function supportedMainKey(key: string) {
  return /^[A-Z0-9]$/.test(key)
    || /^F(?:[1-9]|1[0-2])$/.test(key)
    || new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Backspace", "Delete",
      "Home", "End", "PageUp", "PageDown", "Enter", "Escape", "Tab", "Space"]).has(key);
}

function hasModifiers(actual: Set<string>, expected: string[]) {
  return actual.size === expected.length && expected.every((item) => actual.has(item));
}

function isReservedShortcut(modifiers: Set<string>, key: string, platform: ShortcutPlatform) {
  if (platform === "macos") {
    return (hasModifiers(modifiers, ["Command"])
        && new Set(["A", "C", "F", "H", "M", "Q", "S", "Tab", "V", "W", "X", "Z", "Space"]).has(key))
      || (hasModifiers(modifiers, ["Control"]) && key === "Space")
      || (hasModifiers(modifiers, ["Command", "Shift"]) && key === "W")
      || (hasModifiers(modifiers, ["Control", "Command"]) && key === "Q")
      || (hasModifiers(modifiers, ["Alt", "Command"]) && key === "Escape")
      || (hasModifiers(modifiers, ["Command", "Shift"]) && new Set(["3", "4", "5"]).has(key));
  }
  return (hasModifiers(modifiers, ["Ctrl"])
      && new Set(["A", "C", "F", "S", "V", "W", "X", "Z", "Space"]).has(key))
    || (hasModifiers(modifiers, ["Alt"]) && new Set(["F4", "Space", "Tab"]).has(key))
    || (hasModifiers(modifiers, ["Super"]) && new Set(["D", "E", "L", "R", "S", "Tab", "V"]).has(key))
    || (hasModifiers(modifiers, ["Ctrl", "Alt"]) && key === "Delete");
}

export function shortcutKeycaps(shortcut: string, platform: ShortcutPlatform): string[] {
  const labels: Record<string, string> = platform === "macos"
    ? { Control: "⌃", Ctrl: "⌃", Alt: "⌥", Shift: "⇧", Command: "⌘", Super: "⌘",
        ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→", Enter: "↩", Backspace: "⌫", Delete: "⌦", Space: "Space" }
    : { Control: "Ctrl", Ctrl: "Ctrl", Alt: "Alt", Shift: "Shift", Command: "Win", Super: "Win",
        ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→", Enter: "Enter", Backspace: "Backspace", Delete: "Delete", Space: "Space" };
  return shortcut.split("+").map((part) => labels[part] ?? part);
}
