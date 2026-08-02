import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";
export type LanguagePreference = "system" | "zh-CN" | "en";

export type Settings = {
  retention_days: 1 | 7 | 30 | 90 | 365 | null;
  history_limit: 100 | 500 | 1000 | 5000 | null;
  move_used_to_top: boolean;
  launch_at_login: boolean;
  hotkey: string;
  theme: Theme;
  language: LanguagePreference;
  check_updates: boolean;
  last_update_check: string | null;
};

export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (settings: Settings) => invoke<Settings>("update_settings", { settings });
export const recordUpdateCheck = () => invoke<string>("record_update_check");
export const openLogDir = () => invoke<void>("open_log_dir");
export const quitApp = () => invoke<void>("quit_app");

export function applyTheme(theme: Theme) {
  if (theme === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}
