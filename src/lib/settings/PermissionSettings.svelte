<script lang="ts">
  import { onDestroy } from "svelte";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { error as logError, info as logInfo } from "@tauri-apps/plugin-log";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import { getAutoPastePermissionStatus, openAutoPasteSettings, revealCurrentApp, shouldRestartAfterPermissionCheck, type AutoPastePermissionStatus, type AutoPastePermissionViewStatus } from "$lib/onboarding/api";
  import { openFilePreviewSettings } from "./api";

  let { onerror, heading = $bindable(), active = false, focusRequested = false }: { onerror: (message: string) => void; heading?: HTMLHeadingElement; active?: boolean; focusRequested?: boolean } = $props();
  let permission = $state<Omit<AutoPastePermissionStatus, "status"> & { status: AutoPastePermissionViewStatus }>({ status: "unknown", app_location: "development", app_path: null });
  let checking = $state(false);
  let checkFailed = $state(false);
  let awaitingPermission = $state(false);
  let restartPending = $state(false);
  let restartCancelled = $state(false);
  let restartAttempted = $state(false);
  let restartFailed = $state(false);
  let focusCheckQueued = false;
  let restartTimer: ReturnType<typeof setTimeout> | undefined;
  let wasActive = false;
  let refreshGeneration = 0;
  let autoPasteButton = $state<HTMLButtonElement>();
  let focusHandled = false;
  const statusText = $derived(checking ? t("onboarding.permission.checking") : checkFailed ? t("onboarding.permission.unknown") : permission.status === "ready" ? t("onboarding.permission.ready") : t("onboarding.permission.required"));

  $effect(() => {
    if (!focusRequested) focusHandled = false;
    if (active && !wasActive) void refresh(false);
    if (!active && wasActive) deactivate();
    wasActive = active;
  });
  onDestroy(() => clearTimeout(restartTimer));

  function deactivate() {
    refreshGeneration += 1;
    checking = false;
    focusCheckQueued = false;
    awaitingPermission = false;
    if (restartPending) cancelRestart();
  }

  async function refresh(fromFocus: boolean) {
    if (!active) return;
    if (checking) { if (fromFocus) focusCheckQueued = true; return; }
    const generation = ++refreshGeneration;
    checking = true;
    checkFailed = false;
    try {
      const next = await getAutoPastePermissionStatus();
      if (!active || generation !== refreshGeneration) return;
      const shouldRestart = shouldRestartAfterPermissionCheck(permission.status, next.status, fromFocus, awaitingPermission, restartCancelled || restartAttempted || restartPending);
      permission = next;
      if (shouldRestart) scheduleRestart();
    } catch (reason) {
      if (!active || generation !== refreshGeneration) return;
      permission.status = "unknown";
      checkFailed = true;
      onerror(localizedError(reason));
    } finally {
      if (!active || generation !== refreshGeneration) return;
      checking = false;
      if (focusRequested && !focusHandled) {
        focusHandled = true;
        requestAnimationFrame(() => autoPasteButton?.focus());
      }
      if (focusCheckQueued) { focusCheckQueued = false; void refresh(true); }
    }
  }

  async function manageAutoPaste() {
    if (!active) return;
    if (checkFailed) { await refresh(false); return; }
    clearTimeout(restartTimer);
    restartPending = false; restartCancelled = false; restartAttempted = false; restartFailed = false;
    awaitingPermission = true;
    try { await openAutoPasteSettings(); }
    catch (reason) { awaitingPermission = false; onerror(localizedError(reason)); }
  }

  function scheduleRestart() {
    if (!active) return;
    void logInfo("Accessibility permission changed from required to ready in Settings; scheduling one relaunch");
    restartPending = true;
    restartTimer = setTimeout(() => void restartApp(), 4000);
  }
  function cancelRestart() {
    clearTimeout(restartTimer); restartPending = false; restartCancelled = true;
    void logInfo("Accessibility permission relaunch cancelled by the user");
  }
  async function restartApp() {
    if (!active || restartAttempted) return;
    clearTimeout(restartTimer); restartAttempted = true; restartPending = false;
    try { await relaunch(); }
    catch (reason) { restartFailed = true; void logError(`Accessibility permission relaunch failed: ${localizedError(reason)}`); onerror(localizedError(reason)); }
  }
  async function run(action: () => Promise<unknown>) {
    try { await action(); } catch (reason) { onerror(localizedError(reason)); }
  }
  function onFocus() { if (active && awaitingPermission) void refresh(true); }
  function onKeydown(event: KeyboardEvent) { if (active && event.key === "Escape" && restartPending) { event.preventDefault(); cancelRestart(); } }
</script>

<svelte:window onfocus={onFocus} onkeydown={onKeydown} />
<h1 bind:this={heading} id="settings-section-title" tabindex="-1">{t("settings.permissions")}</h1>
<div class="row" data-permission="auto-paste">
  <span><strong>{t("settings.autoPaste")}</strong><small>{t("settings.autoPasteHelp")}</small><small aria-live="polite" aria-atomic="true">{statusText}</small></span>
  <button bind:this={autoPasteButton} class:ready={!checking && permission.status === "ready"} disabled={checking} aria-busy={checking} onclick={() => void manageAutoPaste()}>{checking ? t("settings.permissionChecking") : checkFailed ? t("settings.permissionRetry") : permission.status === "ready" ? t("settings.permissionReady") : t("settings.permissionGrant")}</button>
</div>
<div class="row">
  <span><strong>{t("settings.filePreview")}</strong><small>{t("settings.filePreviewHelpShort")}</small><small>{t("settings.filePreviewVerifyShort")}</small></span>
  <button onclick={() => void run(openFilePreviewSettings)}>{t("settings.grant")}</button>
</div>
<div class="current-app">
  <strong>{t("settings.currentApp")}</strong>
  <small>{permission.app_path ?? t("onboarding.permission.development")}</small>
  {#if permission.app_path}<button onclick={() => void run(revealCurrentApp)}>{t("onboarding.permission.reveal")}</button>{/if}
</div>
<div class="permission-feedback" aria-live="polite">
  {#if awaitingPermission && permission.status === "permission_required"}<small>{t("onboarding.permission.waiting")}</small><small>{t("onboarding.permission.oldEntry")}</small>{/if}
  {#if restartPending}<span>{t("onboarding.permission.restartPending")}</span><button onclick={() => void restartApp()}>{t("onboarding.permission.restartNow")}</button><button onclick={cancelRestart}>{t("onboarding.permission.cancelRestart")}</button>
  {:else if restartFailed || restartCancelled}<span>{restartFailed ? t("onboarding.permission.restartFailed") : t("onboarding.permission.restartCancelled")}</span><button onclick={() => { restartAttempted = false; void restartApp(); }}>{t("onboarding.permission.restart")}</button>{/if}
</div>

<style>
  h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}h1:focus{outline:none}.row{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}.row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}strong{font-size:var(--fs-body)}small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}button{flex:none;min-height:32px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);white-space:nowrap}button:hover:not(:disabled){color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}button:disabled{opacity:.45}.ready{color:var(--success);background:color-mix(in srgb,var(--success) 8%,transparent)}.current-app{min-height:68px;padding-block:12px;display:grid;grid-template-columns:1fr auto;align-items:center;gap:3px 16px;border-bottom:1px solid var(--hairline)}.current-app strong,.current-app small{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.current-app small{grid-column:1}.current-app button{grid-column:2;grid-row:1/3}.permission-feedback{min-height:32px;margin-top:12px;display:flex;align-items:center;gap:8px;flex-wrap:wrap;color:var(--text-2)}.permission-feedback small{width:100%}
</style>
