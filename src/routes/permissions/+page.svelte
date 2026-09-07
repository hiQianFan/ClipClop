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
  {#if installed}
    <button class="app-drag" onpointerdown={(event) => { event.preventDefault(); void run("start_current_app_drag"); }} onclick={() => void run("reveal_current_app")}>
      <img src="/app-icon.png" alt="" draggable="false" />
      <strong>ClipClop.app</strong>
    </button>
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
</main>
<ActionToolbar><button class="toolbar-button" onclick={() => void run("close_permission_guide")}>{t("permission.return")}</button><button class="toolbar-button primary" onclick={() => void run("close_permission_guide", true)}>{t("permission.restart")}</button></ActionToolbar>
<style>
  :global(body){display:flex;flex-direction:column;background:var(--bg-shell)}main{min-height:0;padding:24px;flex:1}h1{margin:0 0 12px;font-size:var(--fs-heading);color:var(--text-1)}p{margin:0 0 12px;font-size:var(--fs-ui);color:var(--text-2);line-height:1.6}button{min-height:32px;padding:0 12px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-2);background:transparent}button:hover{background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.app-drag{-webkit-appearance:none;appearance:none;width:100%;min-height:126px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;border-style:dashed;background:var(--bg-raised);cursor:grab}.app-drag:active{cursor:grabbing}.app-drag img{width:64px;height:64px}.app-drag strong{color:var(--text-1);font-size:var(--fs-body)}
</style>
