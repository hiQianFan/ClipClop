<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import AppSelect from "$lib/components/AppSelect.svelte";
  import { getPreviewCapability, type PreviewCapability } from "$lib/history/api";
  import { openAutoPasteSettings } from "$lib/onboarding/api";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import { openFilePreviewSettings, type Settings, type TrayClickAction } from "./api";
  import type { ShortcutPlatform } from "./shortcuts";

  let { settings = $bindable(), platform, onquickstart, onerror, heading = $bindable() }: {
    settings: Settings;
    platform: ShortcutPlatform;
    onquickstart: () => void;
    onerror: (message: string) => void;
    heading?: HTMLHeadingElement;
  } = $props();

  let previewCapability = $state<PreviewCapability>({ provider: "unavailable", reason: "detection_failed" });
  const trayItems = $derived([
    { value: "recent", label: t("settings.trayClickRecent") },
    { value: "history", label: t("settings.trayClickHistory") },
  ]);

  onMount(() => {
    if (platform === "windows") void refreshPreviewCapability();
  });

  async function refreshPreviewCapability() {
    try { previewCapability = await getPreviewCapability(); }
    catch { previewCapability = { provider: "unavailable", reason: "detection_failed" }; }
  }

  function windowsPreviewHelp() {
    if (previewCapability.provider === "powertoys_peek") return t("settings.peekReady");
    if (previewCapability.reason === "elevated") return t("settings.peekElevated");
    if (previewCapability.reason === "not_installed") return t("settings.peekNotInstalled");
    return t("settings.peekUnavailable");
  }

  async function run(action: () => Promise<unknown>) {
    try { await action(); }
    catch (reason) { onerror(localizedError(reason)); }
  }
</script>

<h1 bind:this={heading} id="settings-section-title" tabindex="-1">{t("settings.general")}</h1>
<div class="row"><span><strong id="launch-label">{t("settings.launch")}</strong><small id="launch-help">{t("settings.launchHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="launch-label" aria-describedby="launch-help" bind:checked={settings.launch_at_login} /><span class="switch-track"></span></label></div>
{#if platform === "macos"}<div class="row"><span><strong>{t("settings.trayClick")}</strong><small>{t("settings.trayClickHelp")}</small></span><AppSelect value={settings.tray_click_action} items={trayItems} ariaLabel={t("settings.trayClick")} onchange={(value) => settings.tray_click_action = value as TrayClickAction} /></div>{/if}
<div class="row"><span><strong>{t("settings.quickStart")}</strong><small>{t("settings.quickStartHelp")}</small></span><button onclick={onquickstart}>{t("settings.quickStart")}</button></div>
{#if platform === "macos"}<div class="row"><span><strong>{t("settings.autoPaste")}</strong><small>{t("settings.autoPasteHelp")}</small></span><button onclick={() => void run(openAutoPasteSettings)}>{t("settings.manage")}</button></div>{/if}
{#if platform === "macos"}<div class="row"><span><strong>{t("settings.filePreview")}</strong><small>{t("settings.filePreviewHelp")}</small></span><button onclick={() => void run(openFilePreviewSettings)}>{t("settings.manage")}</button></div>{/if}
{#if platform === "windows"}<div class="row"><span><strong>{t("settings.filePreview")}</strong><small>{windowsPreviewHelp()}</small></span>{#if previewCapability.reason === "not_installed"}<button onclick={() => void run(() => openUrl("https://learn.microsoft.com/windows/powertoys/install"))}>{t("settings.peekInstall")}</button>{/if}</div>{/if}

<style>
  h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}h1:focus{outline:none}.row{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}.row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}.row>button{flex:none;min-height:32px;padding:0 12px;white-space:nowrap}strong{font-size:var(--fs-body)}small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}button{border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.switch{position:relative;flex:none;width:44px;height:44px;cursor:pointer}.switch input{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);clip-path:inset(50%);white-space:nowrap}.switch-track{position:absolute;left:4px;top:12px;width:36px;height:20px;border:1px solid color-mix(in srgb,var(--text-2) 42%,var(--bg-selected));border-radius:var(--radius-pill);background:var(--bg-selected);transition:background var(--dur-fast) ease-out,border-color var(--dur-fast) ease-out}.switch-track:after{content:"";position:absolute;left:1px;top:1px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.22);transition:transform var(--dur-fast) ease-out}.switch input:checked+.switch-track{border-color:var(--action);background:var(--action)}.switch input:checked+.switch-track:after{transform:translateX(16px);background:var(--action-on)}.switch input:focus-visible+.switch-track{outline:2px solid var(--text-2);outline-offset:3px}.switch:hover .switch-track{border-color:var(--text-2)}.switch:hover input:checked+.switch-track{border-color:var(--action)}@media(prefers-reduced-motion:reduce){.switch-track,.switch-track:after{transition:none}}@media(forced-colors:active){.switch-track{border:1px solid ButtonText;background:Canvas}.switch-track:after{background:ButtonText}.switch input:checked+.switch-track{background:Highlight}.switch input:checked+.switch-track:after{background:HighlightText}}
</style>
