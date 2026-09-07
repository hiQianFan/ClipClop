<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { Tooltip } from "bits-ui";
  import { RefreshCw } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import { getAutoPastePermissionStatus, type AutoPastePermissionViewStatus } from "./api";
  let { active = true, focusRequested = false, onerror, beforeRestart = async () => true, returnTo = "settings" }: {
    active?: boolean; focusRequested?: boolean; onerror: (message: string) => void;
    beforeRestart?: () => Promise<boolean>; returnTo?: "settings" | "onboarding" | "quick_start";
  } = $props();
  let status = $state<AutoPastePermissionViewStatus>("unknown");
  let checking = $state(false);
  let failed = $state(false);
  let configuring = $state(false);
  let restarting = $state(false);
  let generation = 0;
  let wasActive = false;
  let button = $state<HTMLButtonElement>();
  let focused = false;
  $effect(() => {
    if (active && !wasActive) void refresh();
    if (!active && wasActive) { generation++; checking = false; }
    wasActive = active;
    if (active && focusRequested && !focused && button) { button.focus(); focused = true; }
  });
  onDestroy(() => { generation++; });
  onMount(() => {
    const listener = listen("permission_restart_requested", () => { if (active && configuring) void restart(); }).catch(() => () => {});
    return () => { void listener.then((unlisten) => unlisten()); };
  });
  async function refresh() {
    if (!active || checking || document.visibilityState === "hidden") return;
    const request = ++generation;
    checking = true;
    try {
      const next = await getAutoPastePermissionStatus();
      if (!active || request !== generation) return;
      status = next.status; failed = false; onerror("");
    } catch (reason) {
      if (!active || request !== generation) return;
      failed = true; onerror(localizedError(reason));
    } finally { if (request === generation) checking = false; }
  }
  async function configure(kind: "accessibility" | "files") {
    try {
      await invoke("open_permission_guide", { kind });
      if (kind === "accessibility" && status !== "ready") configuring = true;
    }
    catch (reason) { onerror(localizedError(reason)); }
  }
  async function restart() {
    if (restarting) return;
    restarting = true;
    try {
      if (!await beforeRestart()) { restarting = false; return; }
      localStorage.setItem("permission-restart-return", returnTo);
      await relaunch();
    } catch (reason) {
      localStorage.removeItem("permission-restart-return");
      restarting = false; onerror(localizedError(reason));
    }
  }
</script>
<svelte:window onfocus={() => void refresh()} />
<div class="row">
  <span><strong>{t("settings.autoPaste")}</strong><small>{t("settings.autoPasteHelp")}</small></span>
  <div class="actions">
    <Tooltip.Provider><Tooltip.Root>
      <Tooltip.Trigger class="refresh" aria-label={t("settings.permissionRefresh")} disabled={checking} aria-busy={checking} onclick={() => void refresh()}><RefreshCw size={16} aria-hidden="true" /></Tooltip.Trigger>
      <Tooltip.Portal><Tooltip.Content sideOffset={6}><div class="tooltip">{t("settings.permissionRefresh")}</div></Tooltip.Content></Tooltip.Portal>
    </Tooltip.Root></Tooltip.Provider>
    <button bind:this={button} class:primary={configuring} class:ready={!configuring && !failed && status === "ready"} disabled={restarting} onclick={() => configuring ? void restart() : failed ? void refresh() : void configure("accessibility")}>
      {configuring ? t("permission.restart") : failed ? t("settings.permissionRetry") : status === "ready" ? t("settings.permissionReady") : status === "unknown" && checking ? t("settings.permissionChecking") : t("settings.permissionGrant")}
    </button>
  </div>
</div>
<div class="row">
  <span><strong>{t("settings.filePreview")}</strong><small>{t("settings.filePreviewHelpShort")}</small></span>
  <button onclick={() => void configure("files")}>{t("settings.grant")}</button>
</div>
{#if configuring}<p role="status">{t("permission.restartHint")}</p>{/if}
<style>
  .row{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}.row>span{flex:1;display:flex;flex-direction:column;gap:3px}strong{font-size:var(--fs-body)}small,p{color:var(--text-3);font-size:var(--fs-ui);line-height:1.5}.actions{display:flex;align-items:center;gap:8px}button,.actions :global(.refresh){min-height:32px;padding:0 12px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);white-space:nowrap}button:hover,.actions :global(.refresh:hover){color:var(--text-1);background:var(--bg-hover)}button:focus-visible,.actions :global(.refresh:focus-visible){outline:2px solid var(--text-1);outline-offset:2px}button:disabled,.actions :global(.refresh:disabled){opacity:.45}.actions :global(.refresh){display:flex;align-items:center;justify-content:center;width:32px;padding:0}button.primary{border-color:var(--action);color:var(--action-on);background:var(--action);font-weight:650}button.primary:hover{color:var(--action-on);background:var(--action-hover)}button.ready{color:var(--success);background:color-mix(in srgb,var(--success) 14%,transparent);font-weight:600}button.ready:hover{color:var(--success);background:color-mix(in srgb,var(--success) 22%,transparent)}.tooltip{padding:6px 10px;background:var(--bg-raised);color:var(--text-1);border:1px solid var(--hairline);border-radius:var(--radius-md);font-size:var(--fs-ui)}
</style>
