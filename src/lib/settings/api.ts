import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";
export type LanguagePreference = "system" | "zh-CN" | "en";
export type TrayClickAction = "recent" | "history";

export type Settings = {
  retention_days: 1 | 7 | 30 | 90 | 365 | null;
  history_limit: 100 | 500 | 1000 | 5000 | null;
  move_used_to_top: boolean;
  restore_browse_position: boolean;
  preserve_search_conditions: boolean;
  trim_whitespace: boolean;
  file_preview_enabled: boolean;
  launch_at_login: boolean;
  hotkey: string;
  theme: Theme;
  language: LanguagePreference;
  tray_click_action: TrayClickAction;
  check_updates: boolean;
  last_update_check: string | null;
  skipped_update_version: string | null;
};

export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (settings: Settings) => invoke<Settings>("update_settings", { settings });
export const recordUpdateCheck = () => invoke<string>("record_update_check");
export const skipUpdateVersion = (version: string) => invoke<void>("skip_update_version", { version });
export const openLogDir = () => invoke<void>("open_log_dir");
export const openFilePreviewSettings = () => invoke<void>("open_file_preview_settings");
export const quitApp = () => invoke<void>("quit_app");

export function applyTheme(theme: Theme) {
  if (theme === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}
