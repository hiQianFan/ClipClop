<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import "../app.css";
  import { scheduleAutomaticUpdateCheck } from "$lib/updater/api";
  import { applyTheme, getSettings } from "$lib/settings/api";
  import { setLanguagePreference } from "$lib/i18n/index.svelte";
  let { children } = $props();
  let ready = $state(false);

  onMount(() => {
    let cancelUpdate = () => {};
    let timeout = 0;
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
      window.clearTimeout(timeout);
      cancelUpdate();
    };
  });
</script>

{#if ready}{@render children()}{/if}
