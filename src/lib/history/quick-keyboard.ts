export type QuickKeyAction =
  | { type: "select"; index: number }
  | { type: "copy"; index: number }
  | { type: "preview"; index: number }
  | { type: "page"; page: number; edge: "first" | "last" }
  | { type: "close" };

export function routeQuickKey(key: string, index: number, length: number, canPreview = true, page = 1, totalPages = 1): QuickKeyAction | null {
  if (key === "ArrowRight") return page < totalPages ? { type: "page", page: page + 1, edge: "first" } : null;
  if (key === "ArrowLeft") return page > 1 ? { type: "page", page: page - 1, edge: "last" } : null;
  if (key === "PageDown") return page < totalPages ? { type: "page", page: page + 1, edge: "first" } : null;
  if (key === "PageUp") return page > 1 ? { type: "page", page: page - 1, edge: "last" } : null;
  if (length === 0) return key === "Escape" ? { type: "close" } : null;
  if (key === "ArrowDown") return index >= length - 1 && page < totalPages
    ? { type: "page", page: page + 1, edge: "first" }
    : { type: "select", index: Math.min(index + 1, length - 1) };
  if (key === "ArrowUp") return index <= 0 && page > 1
    ? { type: "page", page: page - 1, edge: "last" }
    : { type: "select", index: Math.max(index, 1) - 1 };
  if (key === "Home") return { type: "select", index: 0 };
  if (key === "End") return { type: "select", index: length - 1 };
  if (key === " " && canPreview) return { type: "preview", index: Math.max(index, 0) };
  if (key === "Enter") return { type: "copy", index: Math.max(index, 0) };
  if (key === "Escape") return { type: "close" };
  if (/^[0-9]$/.test(key)) {
    const target = key === "0" ? 9 : Number(key) - 1;
    return target < length ? { type: "copy", index: target } : null;
  }
  return null;
}
