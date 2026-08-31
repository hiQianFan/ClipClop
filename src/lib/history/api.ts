import { invoke } from "@tauri-apps/api/core";
import type { ClipDetail, HistoryFacets, HistoryFilters, HistoryPage } from "./types";

function historyRequest(query: string, page: number, filters?: HistoryFilters, pageSize = 10) {
  const days = filters?.time_range === "day" ? 1 : filters?.time_range === "week" ? 7 : filters?.time_range === "month" ? 30 : 0;
  return {
    query, page, page_size: pageSize,
    content_type: filters?.content_type ?? null,
    source_id: filters?.source_id ?? null,
    since: days ? new Date(Date.now() - days * 86_400_000).toISOString() : null,
  };
}

export function queryHistory(query = "", page = 1, filters?: HistoryFilters, pageSize = 10): Promise<HistoryPage> {
  return invoke("query_history", { request: historyRequest(query, page, filters, pageSize) });
}

export function getHistoryFacets(query: string, filters: HistoryFilters, sourceQuery = ""): Promise<HistoryFacets> {
  return invoke("get_history_facets", { request: historyRequest(query, 1, filters), sourceQuery });
}

export function getClip(id: string): Promise<ClipDetail> {
  return invoke("get_clip", { id });
}

export function getClipAsset(id: string): Promise<{ data_url: string | null; byte_size: number | null; access_denied: boolean }> {
  return invoke("get_clip_asset", { id });
}

export function getClipFileAsset(id: string, index: number): Promise<{ data_url: string | null; byte_size: number | null; access_denied: boolean }> {
  return invoke("get_clip_file_asset", { id, index });
}

export function getClipThumbnail(id: string): Promise<{ data_url: string | null; byte_size: number | null; access_denied: boolean }> {
  return invoke("get_clip_thumbnail", { id });
}

export type PreviewOutcome =
  | "native_opened"
  | "native_closed"
  | "not_previewable";

export type PreviewCapability = {
  provider: "macos_quicklook" | "powertoys_peek" | "unavailable";
  reason: null | "not_installed" | "elevated" | "detection_failed";
};

export function getPreviewCapability(): Promise<PreviewCapability> {
  return invoke("get_preview_capability");
}

export function canPreviewClip(capability: PreviewCapability, contentType: string | undefined) {
  return capability.provider === "macos_quicklook"
    || (capability.provider === "powertoys_peek" && contentType === "file");
}

export function previewClip(id: string, index = 0): Promise<PreviewOutcome> {
  return invoke("preview_clip", { id, index });
}

export function openClipLink(id: string, originOnly = false): Promise<void> {
  return invoke("open_clip_link", { id, originOnly });
}

export function getSourceAppIcon(id: string): Promise<{ data_url: string | null; byte_size: number | null; access_denied: boolean }> {
  return invoke("get_source_app_icon", { id });
}

export function hidePanel(): Promise<void> {
  return invoke("hide_panel");
}

export function showFullPanel(selectedId: string | null = null, settings = false): Promise<void> {
  return invoke("show_full_panel", { selectedId, settings });
}

export function setQuickSelection(id: string | null): Promise<void> {
  return invoke("set_quick_selection", { id });
}

export function performPagerHaptic(): Promise<void> {
  return invoke("perform_pager_haptic");
}

export function copyClip(id: string, plainText = false): Promise<boolean> {
  return invoke("copy_clip", { id, plainText });
}

export type PasteOutcome =
  | "pasted"
  | "copied_permission_required"
  | "copied_target_lost"
  | "copied_focus_failed"
  | "copied_injection_failed"
  | "already_in_progress"
  | "copied_unsupported_platform";

export function pasteClip(id: string, plainText = false): Promise<PasteOutcome> {
  return invoke("paste_clip", { id, plainText });
}

export function deleteClip(id: string): Promise<void> {
  return invoke("delete_clip", { id });
}

export function clearHistory(): Promise<number> {
  return invoke("clear_history");
}
