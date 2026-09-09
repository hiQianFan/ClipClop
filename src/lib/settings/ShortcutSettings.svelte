<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { localizedError, t, type StaticMessageKey } from "$lib/i18n/index.svelte";
  import ActionToolbar from "$lib/components/ActionToolbar.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import { Dialog } from "bits-ui";
  import { setHotkeyRecording, updateSettings } from "./api";
  import type { Settings } from "./api";
  import { defaultShortcut, shortcutFromKeyboardEvent, shortcutSpokenLabel, type ShortcutPlatform } from "./shortcuts";

  type ShortcutRow = { name: StaticMessageKey; description: StaticMessageKey; keys: string[][] };
  let { settings, platform, heading = $bindable(), onerror }: {
    settings: Settings;
    platform: ShortcutPlatform;
    heading?: HTMLHeadingElement;
    onerror?: (message: string) => void;
  } = $props();
  let recording = $state(false);
  let candidate = $state("");
  let recordError = $state("");
  let capture: HTMLInputElement;
  let trigger: HTMLButtonElement;
  let liveKeys = $state("");
  let saving = $state(false);
  let opening = $state(false);
  let destroyed = false;
  async function beginRecording() {
    if (opening) return;
    opening = true;
    candidate = ""; liveKeys = ""; recordError = "";
    try {
      await setHotkeyRecording(true);
      if (destroyed) { await setHotkeyRecording(false); return; }
      recording = true;
    } catch (reason) { onerror?.(localizedError(reason)); }
    finally { opening = false; }
  }
  async function endRecording() {
    if (!recording) return;
    if (saving) return;
    try { await setHotkeyRecording(false); recording = false; }
    catch (reason) { recordError = localizedError(reason); }
  }
  function onRecordKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      void endRecording();
      return;
    }
    if (event.key === "Tab") return;
    event.preventDefault();
    event.stopPropagation();
    if (event.repeat || event.isComposing) return;
    recordError = "";
    const modifiers = [
      event.ctrlKey ? (platform === "macos" ? "Control" : "Ctrl") : "",
      event.altKey ? "Alt" : "", event.shiftKey ? "Shift" : "",
      event.metaKey ? (platform === "macos" ? "Command" : "Super") : "",
    ].filter(Boolean);
    if (["Control", "Alt", "Shift", "Meta", "OS"].includes(event.key)) {
      liveKeys = modifiers.join("+"); return;
    }
    const result = shortcutFromKeyboardEvent(event, platform);
    if (result.valid) { candidate = result.shortcut; liveKeys = ""; }
    else {
      candidate = ""; liveKeys = "";
      const messages = { invalid_input: "shortcut.invalidInput", invalid_combination: "shortcut.invalidCombination" } as const;
      recordError = t(messages[result.code]);
    }
  }
  async function saveRecording() {
    if (!candidate || saving) return;
    saving = true;
    try {
      // Restore the native registration before settings_update runs its own
      // unregister/register transaction. This avoids racing the recorder lock.
      await setHotkeyRecording(false);
      settings.hotkey = (await updateSettings({ ...settings, hotkey: candidate })).hotkey;
      recording = false;
    } catch (reason) { recordError = localizedError(reason); }
    finally { saving = false; }
  }
  onDestroy(() => {
    destroyed = true;
    if (recording) void setHotkeyRecording(false).catch(() => {});
  });
  onMount(() => {
    const cancelOnBlur = () => { if (recording) void endRecording(); };
    const captureKey = (event: KeyboardEvent) => { if (recording) onRecordKeydown(event); };
    window.addEventListener("blur", cancelOnBlur);
    window.addEventListener("keydown", captureKey, true);
    return () => { window.removeEventListener("blur", cancelOnBlur); window.removeEventListener("keydown", captureKey, true); };
  });

  const groups = $derived<[StaticMessageKey, ShortcutRow[]][]>([
    ["shortcut.group.panel", [
      { name: "shortcut.search", description: "shortcut.searchDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "F"], ["/"]] },
      { name: "shortcut.openSettings", description: "shortcut.openSettingsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", ","]] },
      { name: "shortcut.itemActions", description: "shortcut.itemActionsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "K"], ...(platform === "windows" ? [["Shift", "F10"]] : [])] },
      { name: "shortcut.menuNavigation", description: "shortcut.menuNavigationDesc", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
      { name: "shortcut.backLayers", description: "shortcut.backLayersDesc", keys: [["Escape"]] },
      { name: "shortcut.closePanel", description: "shortcut.closePanelDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "W"]] },
    ]],
    ["shortcut.group.list", [
      { name: "shortcut.moveSelection", description: "shortcut.moveSelectionDesc", keys: [["ArrowUp"], ["ArrowDown"]] },
      { name: "shortcut.firstLast", description: "shortcut.firstLastDesc", keys: [["Home"], ["End"]] },
      { name: "shortcut.pages", description: "shortcut.pagesDesc", keys: [["ArrowLeft"], ["PageUp"], ["ArrowRight"], ["PageDown"]] },
      { name: "shortcut.visible", description: "shortcut.visibleDesc", keys: [["1"], ["…"], ["0"]] },
      { name: "shortcut.paste", description: "shortcut.pasteDesc", keys: [["Enter"]] },
      { name: "shortcut.pastePlain", description: "shortcut.pastePlainDesc", keys: [["Shift", "Enter"]] },
      { name: "shortcut.open", description: "shortcut.openDesc", keys: [["Space"]] },
      { name: "shortcut.copyPlain", description: "shortcut.copyPlainDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "Shift", "C"]] },
      { name: "shortcut.remove", description: "shortcut.removeDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", platform === "macos" ? "Backspace" : "Delete"]] },
      { name: "shortcut.switchFile", description: "shortcut.switchFileDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "ArrowLeft"], [platform === "macos" ? "Command" : "Ctrl", "ArrowRight"]] },
    ]],
    ["shortcut.group.files", [
      { name: "shortcut.previousNextFile", description: "shortcut.previousNextFileDesc", keys: [["ArrowLeft"], ["ArrowRight"]] },
      { name: "shortcut.firstLastFile", description: "shortcut.firstLastFileDesc", keys: [["Home"], ["End"]] },
    ]],
    ["shortcut.group.settings", [
      { name: "shortcut.switchCategory", description: "shortcut.switchCategoryDesc", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
      { name: "shortcut.enterDetail", description: "shortcut.enterDetailDesc", keys: [["ArrowRight"], ["Tab"]] },
      { name: "shortcut.returnCategory", description: "shortcut.returnCategoryDesc", keys: [["ArrowLeft"]] },
      { name: "shortcut.returnHistory", description: "shortcut.returnHistoryDesc", keys: [["Escape"]] },
    ]],
  ]);

</script>

<h1 bind:this={heading} id="settings-section-title" tabindex="-1">{t("settings.shortcuts")}</h1>
<p class="section-intro">{t("settings.shortcutReferenceIntro")}</p>
<p class="shortcut-help"><strong>{t("settings.shortcutHow")}</strong>{t("settings.shortcutHowHelp")}{platform === "macos" ? t("settings.macKeyHelp") : t("settings.windowsKeyHelp")}</p>
<section class="shortcut-group" aria-labelledby="global-shortcut-title"><h2 id="global-shortcut-title">{t("settings.global")}</h2><div class="shortcut-row"><span><strong>{t("settings.toggle")}</strong><small>{t("settings.toggleHelp")}</small></span><div class="shortcut-actions"><button bind:this={trigger} disabled={opening} class="shortcut-trigger" aria-label={t("settings.currentShortcut", { shortcut: shortcutSpokenLabel(settings.hotkey, platform) })} onclick={beginRecording}><ShortcutHint shortcut={settings.hotkey} {platform} variant="keycaps" /></button></div></div></section>
<Dialog.Root open={recording} onOpenChange={(open) => { if (!open) void endRecording(); }}>
  <Dialog.Portal>
    <Dialog.Overlay class="record-overlay" />
    <Dialog.Content class="record-dialog"
      onEscapeKeydown={(event) => { if (saving) event.preventDefault(); }}
      onInteractOutside={(event) => event.preventDefault()}
      onOpenAutoFocus={(event) => { event.preventDefault(); capture?.focus(); }}
      onCloseAutoFocus={(event) => { event.preventDefault(); trigger?.focus(); }}>
      <div class="record-copy">
        <Dialog.Title class="record-title">{t("settings.pressShortcut")}</Dialog.Title>
        <Dialog.Description class="record-description">{t("settings.recordPrompt")}</Dialog.Description>
        <div class="record-preview">
          <input bind:this={capture} readonly aria-label={t("settings.pressShortcut")}
            onkeyup={() => liveKeys = ""} onblur={() => liveKeys = ""} disabled={saving} />
          <div class="record-keys" aria-live="polite">{#if liveKeys || candidate}<ShortcutHint shortcut={liveKeys || candidate} {platform} variant="keycaps" />{:else}<span>{t("settings.pressShortcut")}</span>{/if}</div>
        </div>
        {#if recordError}<p class="record-error" role="alert">{recordError}</p>{/if}
      </div>
      <ActionToolbar>
        <button class="toolbar-button" disabled={saving} onclick={() => { candidate = defaultShortcut(platform); liveKeys = ""; recordError = ""; }}>{t("settings.restoreDefault")}</button>
        <button class="toolbar-button" disabled={saving} onclick={() => void endRecording()}>{t("common.cancel")}</button>
        <button class="toolbar-button primary" disabled={!candidate || saving} aria-busy={saving} onclick={saveRecording}>{t("common.save")}</button>
      </ActionToolbar>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
{#each groups as group}<section class="shortcut-group" aria-labelledby={`shortcut-${group[0]}`}><h2 id={`shortcut-${group[0]}`}>{t(group[0])}</h2>{#each group[1] as item}<div class="shortcut-row"><span><strong>{t(item.name)}</strong><small>{t(item.description)}</small></span><div class="key-list">{#each item.keys as keys, alternativeIndex}{#if alternativeIndex}<span class="alternative" aria-label={t("common.or")}>/</span>{/if}<ShortcutHint shortcut={keys.join("+")} {platform} variant="keycaps" />{/each}</div></div>{/each}</section>{/each}

<style>
  h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}h1:focus{outline:none}.section-intro{margin:0 0 8px;color:var(--text-2);font-size:var(--fs-ui);line-height:1.5}.shortcut-help{max-width:72ch;margin:0 0 18px;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.55}.shortcut-help strong,.shortcut-row strong{color:var(--text-1);font-size:var(--fs-body)}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:var(--fs-ui);color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}.shortcut-row small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}.shortcut-actions,.key-list{flex:none;display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.shortcut-actions :global(kbd){min-width:92px;justify-content:center}.shortcut-trigger{padding:5px;border:1px solid transparent;border-radius:var(--radius-md);background:transparent;color:inherit;line-height:1}.shortcut-trigger:hover{border-color:var(--hairline);background:var(--bg-hover)}
  :global(.record-overlay){position:fixed;inset:20px;z-index:var(--z-modal-backdrop);border-radius:var(--radius-xl);background:color-mix(in srgb,var(--text-1) 28%,transparent)}
  :global(.record-dialog){position:fixed;z-index:var(--z-modal);top:50%;left:50%;width:min(380px,calc(100vw - 64px));overflow:hidden;transform:translate(-50%,-50%);border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised);box-shadow:var(--menu-shadow)}
  .record-copy{padding:var(--space-10)}
  :global(.record-title){margin:0 0 var(--space-4);color:var(--text-1);font-size:var(--fs-heading);font-weight:680;line-height:var(--lh-tight)}
  :global(.record-description){margin:0;color:var(--text-2);font-size:var(--fs-ui);line-height:var(--lh-snug)}
  .record-preview{position:relative;margin-top:20px;min-height:64px;border:1px solid var(--hairline);border-radius:var(--radius-md);background:var(--bg-selected)}
  .record-preview:focus-within{outline:2px solid var(--text-2);outline-offset:2px}
  .record-preview input{position:absolute;inset:0;width:100%;height:100%;padding:0;border:0;opacity:0;cursor:pointer}
  .record-keys{min-height:64px;display:flex;align-items:center;justify-content:center;pointer-events:none;color:var(--text-3);font-size:var(--fs-ui)}
  .record-error{margin:12px 0 0;color:var(--danger);font-size:var(--fs-ui)}
  .shortcut-trigger:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
  .alternative{margin:0 2px;color:var(--text-3);font-size:var(--fs-meta)}
</style>
