import { invoke } from "@tauri-apps/api/core";
import type { ClipDetail, HistoryPage } from "./types";

export function queryHistory(query = "", page = 1): Promise<HistoryPage> {
  return invoke("query_history", {
    request: { query, page, page_size: 10 },
  });
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
  | "fallback_opened"
  | "not_previewable";

export function previewClip(id: string, index = 0): Promise<PreviewOutcome> {
  return invoke("preview_clip", { id, index });
}

export function openClipLink(id: string): Promise<void> {
  return invoke("open_clip_link", { id });
}

export function getSourceAppIcon(id: string): Promise<{ data_url: string | null; byte_size: number | null; access_denied: boolean }> {
  return invoke("get_source_app_icon", { id });
}

export function hidePanel(): Promise<void> {
  return invoke("hide_panel");
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
