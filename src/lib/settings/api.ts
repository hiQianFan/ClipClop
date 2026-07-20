import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";

export type Settings = {
  retention_days: 7 | 30 | 90;
  launch_at_login: boolean;
  hotkey: string;
  ignored_apps: string[];
  theme: Theme;
  check_updates: boolean;
  last_update_check: string | null;
};

export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (settings: Settings) => invoke<Settings>("update_settings", { settings });
export const recordUpdateCheck = () => invoke<string>("record_update_check");
export const quitApp = () => invoke<void>("quit_app");
export const ignoreSource = (appId: string) => invoke<Settings>("ignore_source", { appId });

export function applyTheme(theme: Theme) {
  if (theme === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}
