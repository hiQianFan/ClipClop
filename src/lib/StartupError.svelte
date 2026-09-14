<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n/index.svelte";

  let { failure }: { failure: {
    kind: string;
    app_version: string;
    database_version: number | null;
    required_version: number | null;
  } } = $props();
  let actionFailed = $state(false);

  async function act(command: string) {
    actionFailed = false;
    try { await invoke(command); }
    catch { actionFailed = true; }
  }
</script>

<main>
  <h1>{t("startup.title")}</h1>
  <p>{failure.kind === "too_new" ? t("startup.tooNew") : failure.kind === "in_use" ? t("startup.inUse") : t("startup.storage")}</p>
  <p>{t("startup.preserved")}</p>
  <p class="version">ClipClop {failure.app_version}</p>
  {#if failure.database_version !== null && failure.required_version !== null}
    <p class="version">{t("startup.versions", { database: failure.database_version, required: failure.required_version })}</p>
  {/if}
  <div class="actions">
    <button class="primary" onclick={() => act("open_website")}>{t("startup.download")}</button>
    <button onclick={() => act("open_log_dir")}>{t("startup.logs")}</button>
    <button onclick={() => act("quit_app")}>{t("startup.quit")}</button>
  </div>
  {#if actionFailed}<p role="alert">{t("startup.actionFailed")}</p>{/if}
</main>

<style>
  main { height: 100dvh; box-sizing: border-box; overflow: auto; padding: 40px; background: var(--bg-shell); color: var(--text-1); }
  h1 { font-size: 22px; margin: 0 0 20px; }
  p { line-height: 1.6; max-width: 60ch; }
  .version { color: var(--text-2); font-size: 13px; }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 24px; }
  button { padding: 9px 14px; border: 1px solid var(--hairline); border-radius: var(--radius-md); background: var(--bg-raised); color: var(--text-1); cursor: pointer; }
  button:hover { background: var(--bg-hover); }
  button:focus-visible { outline: 2px solid var(--text-1); outline-offset: 3px; }
  button.primary { background: var(--action); color: var(--action-on); }
</style>
