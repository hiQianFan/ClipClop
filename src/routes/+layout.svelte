<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import StartupError from "$lib/StartupError.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import "../app.css";
  import { scheduleAutomaticUpdateCheck } from "$lib/updater/api";
  import { applyTheme, getSettings, THEME_PREVIEW_EVENT, type LanguagePreference, type Theme } from "$lib/settings/api";
  import { setLanguagePreference } from "$lib/i18n/index.svelte";
  let { children } = $props();
  let ready = $state(false);
  let startupFailure = $state<{
    kind: string; app_version: string;
    database_version: number | null; required_version: number | null;
  } | null>(null);

  onMount(() => {
    let cancelUpdate = () => {};
    let cancelSettings = () => {};
    let cancelThemePreview = () => {};
    let destroyed = false;
    let timeout = 0;
    void listen<{ theme: Theme; language: LanguagePreference }>("settings_changed", ({ payload }) => {
      applyTheme(payload.theme);
      setLanguagePreference(payload.language);
    }).then((unlisten) => {
      if (destroyed) unlisten();
      else cancelSettings = unlisten;
    }).catch((error) => {
      console.warn("Failed to listen for settings changes", error);
    });
    void listen<{ theme: Theme }>(THEME_PREVIEW_EVENT, ({ payload }) => {
      applyTheme(payload.theme);
    }).then((unlisten) => {
      if (destroyed) unlisten();
      else cancelThemePreview = unlisten;
    }).catch((error) => {
      console.warn("Failed to listen for theme previews", error);
    });
    void invoke<typeof startupFailure>("get_startup_failure").then((failure) => {
      if (destroyed) return;
      if (failure) {
        applyTheme("system");
        setLanguagePreference("system");
        startupFailure = failure;
        return;
      }
      const timeoutFailure = new Promise<never>((_, reject) => {
        timeout = window.setTimeout(() => reject(new Error("settings bootstrap timed out")), 3_000);
      });
      return Promise.race([getSettings(), timeoutFailure]).then((settings) => {
        applyTheme(settings.theme);
        setLanguagePreference(settings.language);
      }).catch(() => {
        applyTheme("system");
        setLanguagePreference("system");
      }).then(() => {
        window.clearTimeout(timeout);
        if (destroyed) return;
        ready = true;
        if (getCurrentWindow().label === "main") cancelUpdate = scheduleAutomaticUpdateCheck();
      });
    }).catch(() => {
      if (destroyed) return;
      applyTheme("system");
      setLanguagePreference("system");
      startupFailure = { kind: "storage", app_version: "", database_version: null, required_version: null };
    });
    return () => {
      destroyed = true;
      window.clearTimeout(timeout);
      cancelSettings();
      cancelThemePreview();
      cancelUpdate();
    };
  });
</script>

{#if startupFailure}
  <StartupError failure={startupFailure} />
{:else if ready}
  {@render children()}
{/if}
