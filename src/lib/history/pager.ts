export const PAGE_DRAG_STEP = 48;

export function draggedPage(startPage: number, deltaX: number, totalPages: number) {
  return Math.max(1, Math.min(startPage + Math.trunc(-deltaX / PAGE_DRAG_STEP), totalPages));
}

export function visiblePageTicks(currentPage: number, totalPages: number) {
  const start = Math.max(1, currentPage - 3);
  const end = Math.min(totalPages, currentPage + 3);
  return Array.from({ length: Math.max(0, end - start + 1) }, (_, index) => start + index);
}
