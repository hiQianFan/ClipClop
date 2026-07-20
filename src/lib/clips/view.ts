import type { AppError, ClipDetail, ClipSummary } from "./types";

export function pasteFallbackMessage(outcome: string) {
  if (outcome === "copied_permission_required") return "已复制；请允许辅助功能权限后自动粘贴";
  if (outcome === "copied_target_lost") return "已复制；原窗口已关闭，请手动粘贴";
  if (outcome === "copied_focus_failed") return "已复制；无法恢复原窗口，请手动粘贴";
  if (outcome === "copied_injection_failed") return "已复制；系统拦截了自动粘贴，请手动粘贴";
  if (outcome === "already_in_progress") return "正在处理上一次粘贴，请稍后重试";
  return "已复制；当前平台暂不支持自动粘贴";
}

export function errorMessage(reason: unknown) {
  if (typeof reason === "object" && reason && "message" in reason) return String((reason as AppError).message);
  return String(reason ?? "未知错误");
}

export function exactTime(value: string) {
  const date = new Date(value);
  const pad = (number: number) => String(number).padStart(2, "0");
  const year = date.getFullYear() === new Date().getFullYear() ? "" : `${date.getFullYear()}-`;
  return `${year}${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0)} MB`;
}

export function metadataFacts(detail: ClipDetail, fileIndex: number) {
  const facts: Array<{ label: string; value: string }> = [];
  if (detail.content_type === "image") {
    if (detail.metadata.width && detail.metadata.height) {
      facts.push({ label: "尺寸", value: `${detail.metadata.width} × ${detail.metadata.height}` });
    }
    facts.push({ label: "大小", value: formatBytes(detail.byte_size) });
  } else if (detail.content_type === "file") {
    const files = filePaths(detail);
    facts.push({ label: "文件", value: `${fileIndex + 1}/${files.length || 1}` });
    const size = detail.metadata.file_sizes?.[fileIndex];
    if (typeof size === "number") facts.push({ label: "大小", value: formatBytes(size) });
  } else {
    const count = detail.metadata.char_count ?? detail.plain_text?.length ?? 0;
    if (count) facts.push({ label: "字符", value: count.toLocaleString() });
    facts.push({ label: "大小", value: formatBytes(detail.byte_size) });
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
  return normalized.split(/[\\/]/).pop() || normalized;
}

export function cacheSet<K, V>(cache: Map<K, V>, key: K, value: V, limit = 100) {
  if (!cache.has(key) && cache.size >= limit) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  cache.set(key, value);
}
