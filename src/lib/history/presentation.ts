import type { ClipDetail, ClipSummary } from "./types";
import { formatDateTime, formatNumber, localizedError, t } from "$lib/i18n/index.svelte";

export function pasteFallbackMessage(outcome: string) {
  if (outcome === "copied_permission_required") return t("paste.permission");
  if (outcome === "copied_target_lost") return t("paste.targetLost");
  if (outcome === "copied_focus_failed") return t("paste.focusFailed");
  if (outcome === "copied_injection_failed") return t("paste.injectionFailed");
  if (outcome === "already_in_progress") return t("paste.inProgress");
  return t("paste.unsupported");
}

export function errorMessage(reason: unknown) {
  return localizedError(reason);
}

export const exactTime = formatDateTime;

export function formatBytes(bytes: number) {
  if (bytes < 1024) return `${formatNumber(bytes)} B`;
  if (bytes < 1024 * 1024) {
    const digits = bytes < 10 * 1024 ? 1 : 0;
    return `${formatNumber(bytes / 1024, { minimumFractionDigits: digits, maximumFractionDigits: digits })} KB`;
  }
  const digits = bytes < 10 * 1024 * 1024 ? 1 : 0;
  return `${formatNumber(bytes / (1024 * 1024), { minimumFractionDigits: digits, maximumFractionDigits: digits })} MB`;
}

export function metadataFacts(detail: ClipDetail, fileIndex: number) {
  const facts: Array<{ label: string; value: string }> = [];
  if (detail.content_type === "image") {
    if (detail.metadata.width && detail.metadata.height) {
      facts.push({ label: t("meta.dimensions"), value: `${formatNumber(detail.metadata.width)} × ${formatNumber(detail.metadata.height)}` });
    }
    facts.push({ label: t("meta.size"), value: formatBytes(detail.byte_size) });
  } else if (detail.content_type === "file") {
    const files = filePaths(detail);
    facts.push({ label: t("meta.file"), value: `${formatNumber(fileIndex + 1)}/${formatNumber(files.length || 1)}` });
    const size = detail.metadata.file_sizes?.[fileIndex];
    if (typeof size === "number") facts.push({ label: t("meta.size"), value: formatBytes(size) });
  } else {
    const count = detail.metadata.char_count ?? detail.plain_text?.length ?? 0;
    if (count) facts.push({ label: t("meta.characters"), value: formatNumber(count) });
    facts.push({ label: t("meta.size"), value: formatBytes(detail.byte_size) });
  }
  return facts;
}

export function groupedFiles(item: ClipSummary) {
  return item.content_type === "file" ? item.metadata.files ?? [] : [];
}

export function canExpand(item: ClipSummary) {
  return groupedFiles(item).length > 1;
}

export function filePaths(record: ClipDetail) {
  return record.metadata.files ?? [];
}

export function fileName(path: string) {
  const normalized = path.replace(/^file:\/\//, "");
  return normalized.split(/[\\/]/).pop() || normalized || t("meta.file");
}

export function clipPreview(item: Pick<ClipSummary, "content_type" | "preview">) {
  return item.preview || (item.content_type === "file" ? t("meta.file") : "");
}

export function cacheSet<K, V>(cache: Map<K, V>, key: K, value: V, limit = 100) {
  if (!cache.has(key) && cache.size >= limit) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  cache.set(key, value);
}
