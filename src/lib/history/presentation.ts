import type { ClipDetail, ClipSummary } from "./types";

type NumberFormatter = (value: number, options?: Intl.NumberFormatOptions) => string;
type MetadataLabels = {
  dimensions: string;
  size: string;
  file: string;
  files: string;
  hostname: string;
  type: string;
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
    const sizes = files.map((_, index) => fileByteSizes[index] ?? detail.metadata.file_sizes?.[index]);
    if (files.length > 1) {
      facts.push({ label: labels.files, value: formatNumber(files.length) });
      if (sizes.every((size): size is number => typeof size === "number")) {
        facts.push({ label: labels.size, value: formatBytes(sizes.reduce((total, size) => total + size, 0), formatNumber) });
      } else if (typeof sizes[fileIndex] === "number") {
        facts.push({ label: labels.size, value: formatBytes(sizes[fileIndex], formatNumber) });
      }
    } else {
      const name = fileName(files[0] ?? detail.preview);
      const extension = name.includes(".") ? name.slice(name.lastIndexOf(".") + 1) : "";
      facts.push({ label: labels.type, value: extension ? extension.toUpperCase() : labels.file });
      if (typeof sizes[0] === "number") facts.push({ label: labels.size, value: formatBytes(sizes[0], formatNumber) });
    }
  } else if (detail.content_type === "color") {
    facts.push({ label: labels.type, value: "HEX" });
  } else {
    const count = detail.metadata.char_count ?? Array.from(detail.plain_text ?? "").length;
    if (detail.content_type === "link") {
      try {
        const url = new URL(detail.plain_text ?? detail.preview);
        if (["http:", "https:"].includes(url.protocol) && url.hostname) facts.push({ label: labels.hostname, value: url.hostname });
      } catch { /* Invalid links simply omit the hostname fact. */ }
    }
    facts.push({ label: labels.characters, value: formatNumber(count) });
    if (detail.content_type !== "link") facts.push({ label: labels.size, value: formatBytes(detail.byte_size, formatNumber) });
  }
  return facts.slice(0, 2);
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
