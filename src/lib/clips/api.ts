import { invoke } from "@tauri-apps/api/core";
import type { ClipDetail, ClipPage } from "./types";

export function listClips(query = "", page = 1): Promise<ClipPage> {
  return invoke("list_clips", {
    request: { query, page, page_size: 10 },
  });
}

export function getClip(id: string): Promise<ClipDetail> {
  return invoke("get_clip", { id });
}

export function getClipAsset(id: string): Promise<{ data_url: string | null }> {
  return invoke("get_clip_asset", { id });
}

export function getClipThumbnail(id: string): Promise<{ data_url: string | null }> {
  return invoke("get_clip_thumbnail", { id });
}

export function openClip(id: string): Promise<void> {
  return invoke("open_clip", { id });
}

export function getSourceAppIcon(appId: string): Promise<{ data_url: string | null }> {
  return invoke("get_source_app_icon", { appId });
}

export function hidePanel(): Promise<void> {
  return invoke("hide_panel");
}

export function copyClip(id: string, mode: "rich" | "plain"): Promise<void> {
  return invoke("copy_clip", { id, mode });
}

export function deleteClip(id: string): Promise<void> {
  return invoke("delete_clip", { id });
}

export function clearHistory(): Promise<number> {
  return invoke("clear_history");
}
