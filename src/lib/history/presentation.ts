import type { ClipDetail, ClipSummary } from "./types";

type NumberFormatter = (value: number, options?: Intl.NumberFormatOptions) => string;
type MetadataLabels = {
  dimensions: string;
  size: string;
  file: string;
  characters: string;
};

export function formatBytes(bytes: number, formatNumber: NumberFormatter = (value) => String(value)) {
  if (bytes < 1024) return `${formatNumber(bytes)} B`;
  if (bytes < 1024 * 1024) {
    const digits = bytes < 10 * 1024 ? 1 : 0;
    return `${formatNumber(bytes / 1024, { minimumFractionDigits: digits, maximumFractionDigits: digits })} KB`;
  }
  const digits = bytes < 10 * 1024 * 1024 ? 1 : 0;
  return `${formatNumber(bytes / (1024 * 1024), { minimumFractionDigits: digits, maximumFractionDigits: digits })} MB`;
}

export function metadataFacts(
  detail: ClipDetail,
  fileIndex: number,
  fileByteSizes: Array<number | null>,
  labels: MetadataLabels,
  formatNumber: NumberFormatter,
) {
  const facts: Array<{ label: string; value: string }> = [];
  if (detail.content_type === "image") {
    if (detail.metadata.width && detail.metadata.height) {
      facts.push({ label: labels.dimensions, value: `${formatNumber(detail.metadata.width)} × ${formatNumber(detail.metadata.height)}` });
    }
    facts.push({ label: labels.size, value: formatBytes(detail.byte_size, formatNumber) });
  } else if (detail.content_type === "file") {
    const files = filePaths(detail);
    facts.push({ label: labels.file, value: `${formatNumber(fileIndex + 1)}/${formatNumber(files.length || 1)}` });
    const size = fileByteSizes[fileIndex] ?? detail.metadata.file_sizes?.[fileIndex];
    if (typeof size === "number") facts.push({ label: labels.size, value: formatBytes(size, formatNumber) });
  } else {
    const count = detail.metadata.char_count ?? detail.plain_text?.length ?? 0;
    if (count) facts.push({ label: labels.characters, value: formatNumber(count) });
    facts.push({ label: labels.size, value: formatBytes(detail.byte_size, formatNumber) });
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

export function fileName(path: string, fallback = "") {
  const normalized = path.replace(/^file:\/\//, "");
  return normalized.split(/[\\/]/).pop() || normalized || fallback;
}

export function clipPreview(
  item: Pick<ClipSummary, "content_type" | "preview">,
  fileFallback = "",
) {
  return item.preview || (item.content_type === "file" ? fileFallback : "");
}

export const detailText = (detail: ClipDetail, trimWhitespace: boolean) =>
  trimWhitespace ? (detail.plain_text ?? detail.preview).trim() : detail.plain_text ?? detail.preview;

export function cacheSet<K, V>(cache: Map<K, V>, key: K, value: V, limit = 100) {
  if (!cache.has(key) && cache.size >= limit) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  cache.set(key, value);
}
