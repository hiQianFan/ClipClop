export type QuickKeyAction =
  | { type: "select"; index: number }
  | { type: "copy"; index: number }
  | { type: "preview"; index: number }
  | { type: "close" };

export function routeQuickKey(key: string, index: number, length: number): QuickKeyAction | null {
  if (length === 0) return key === "Escape" ? { type: "close" } : null;
  if (key === "ArrowDown") return { type: "select", index: Math.min(index + 1, length - 1) };
  if (key === "ArrowUp") return { type: "select", index: Math.max(index, 1) - 1 };
  if (key === "Home") return { type: "select", index: 0 };
  if (key === "End") return { type: "select", index: length - 1 };
  if (key === " ") return { type: "preview", index: Math.max(index, 0) };
  if (key === "Enter") return { type: "copy", index: Math.max(index, 0) };
  if (key === "Escape") return { type: "close" };
  if (/^[0-9]$/.test(key)) {
    const target = key === "0" ? 9 : Number(key) - 1;
    return target < length ? { type: "copy", index: target } : null;
  }
  return null;
}
