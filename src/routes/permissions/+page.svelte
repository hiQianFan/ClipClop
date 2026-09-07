<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import ActionToolbar from "$lib/components/ActionToolbar.svelte";
  let error = $state("");
  let installed = $state(false);
  onMount(async () => {
    try {
      const status = await invoke<{ app_location: string }>("get_auto_paste_permission_status");
      installed = status.app_location !== "development";
    } catch (reason) {
      error = localizedError(reason);
    }
  });
  async function run(command: string, restart = false) {
    try { await invoke(command, { restart }); } catch (reason) { error = localizedError(reason); }
  }
</script>
<main>
  <h1>{t("permission.guide")}</h1>
  <p>{t(installed ? "permission.drag" : "permission.development")}</p>
  {#if installed}<button onclick={() => void run("reveal_current_app")}>{t("onboarding.permission.reveal")}</button>{/if}
  <p>{t("permission.restartHint")}</p>
  {#if error}<p role="alert">{error}</p>{/if}
</main>
<ActionToolbar><button class="toolbar-button" onclick={() => void run("close_permission_guide")}>{t("permission.return")}</button><button class="toolbar-button primary" onclick={() => void run("close_permission_guide", true)}>{t("permission.restart")}</button></ActionToolbar>
<style>
  :global(body){display:flex;flex-direction:column}main{min-height:0;padding:24px;flex:1}h1{margin:0 0 14px;font-size:var(--fs-heading);color:var(--text-1)}p{margin:0 0 14px;font-size:var(--fs-ui);color:var(--text-2);line-height:1.6}button{min-height:32px;padding:0 12px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-2);background:transparent}button:hover{background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
</style>
