<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import "../app.css";
  import { scheduleAutomaticUpdateCheck } from "$lib/updater/api";
  import { applyTheme, getSettings, THEME_PREVIEW_EVENT, type LanguagePreference, type Theme } from "$lib/settings/api";
  import { setLanguagePreference } from "$lib/i18n/index.svelte";
  let { children } = $props();
  let ready = $state(false);

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
    const timeoutFailure = new Promise<never>((_, reject) => {
      timeout = window.setTimeout(() => reject(new Error("settings bootstrap timed out")), 3_000);
    });
    void Promise.race([getSettings(), timeoutFailure]).then((settings) => {
      applyTheme(settings.theme);
      setLanguagePreference(settings.language);
    }).catch(() => {
      applyTheme("system");
      setLanguagePreference("system");
    }).then(() => {
      window.clearTimeout(timeout);
      ready = true;
      if (getCurrentWindow().label === "main") cancelUpdate = scheduleAutomaticUpdateCheck();
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

{#if ready}{@render children()}{/if}
