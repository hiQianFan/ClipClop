<script lang="ts">
  import { t, type StaticMessageKey } from "$lib/i18n/index.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import type { Settings } from "./api";
  import { defaultShortcut, shortcutFromKeyboardEvent, shortcutSpokenLabel, type ShortcutPlatform } from "./shortcuts";

  type ShortcutRow = { name: StaticMessageKey; description: StaticMessageKey; keys: string[][] };
  let { settings = $bindable(), platform, onstatus, heading = $bindable(), recorder = $bindable(), recording = $bindable(false) }: {
    settings: Settings;
    platform: ShortcutPlatform;
    onstatus: (message: string) => void;
    heading?: HTMLHeadingElement;
    recorder?: HTMLButtonElement;
    recording?: boolean;
  } = $props();
  let shortcutError = $state("");

  $effect(() => {
    if (!recording) shortcutError = "";
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
      { name: "shortcut.saveSettings", description: "shortcut.saveSettingsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "S"]] },
      { name: "shortcut.returnHistory", description: "shortcut.returnHistoryDesc", keys: [["Escape"]] },
    ]],
  ]);

  function record(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault(); event.stopPropagation();
    if (event.key === "Escape") { recording = false; shortcutError = ""; onstatus(t("settings.recordCancelled")); return; }
    const result = shortcutFromKeyboardEvent(event, platform);
    if (!result.valid) { shortcutError = t(result.code === "invalid_input" ? "shortcut.invalidInput" : result.code === "reserved" ? "shortcut.reserved" : "shortcut.invalidCombination"); return; }
    settings.hotkey = result.shortcut; recording = false; shortcutError = "";
    onstatus(t("settings.recorded", { shortcut: shortcutSpokenLabel(result.shortcut, platform) }));
  }

  function restore() {
    settings.hotkey = defaultShortcut(platform); recording = false; shortcutError = ""; onstatus(t("settings.restored"));
  }
</script>

<h1 bind:this={heading} id="settings-section-title" tabindex="-1">{t("settings.shortcuts")}</h1>
<p class="section-intro">{t("settings.shortcutIntro")}</p>
<p class="shortcut-help"><strong>{t("settings.shortcutHow")}</strong>{t("settings.shortcutHowHelp")}{platform === "macos" ? t("settings.macKeyHelp") : t("settings.windowsKeyHelp")}</p>
<section class="shortcut-group" aria-labelledby="global-shortcut-title"><h2 id="global-shortcut-title">{t("settings.global")}</h2><div class="shortcut-row"><span><strong>{t("settings.toggle")}</strong><small>{t("settings.toggleHelp")}</small></span><div class="shortcut-actions"><ShortcutHint shortcut={settings.hotkey} {platform} variant="keycaps" label={t("settings.currentShortcut", { shortcut: shortcutSpokenLabel(settings.hotkey, platform) })} /><button bind:this={recorder} class:recording onclick={() => { recording = true; shortcutError = ""; onstatus(t("settings.recordPrompt")); }} onkeydown={record}>{recording ? t("settings.pressShortcut") : t("settings.change")}</button><button onclick={restore} disabled={settings.hotkey === defaultShortcut(platform)}>{t("settings.restoreDefault")}</button></div></div>{#if shortcutError}<p class="inline-error" role="alert">{shortcutError}</p>{/if}</section>
{#each groups as group}<section class="shortcut-group" aria-labelledby={`shortcut-${group[0]}`}><h2 id={`shortcut-${group[0]}`}>{t(group[0])}</h2>{#each group[1] as item}<div class="shortcut-row"><span><strong>{t(item.name)}</strong><small>{t(item.description)}</small></span><div class="key-list">{#each item.keys as keys, alternativeIndex}{#if alternativeIndex}<span class="alternative" aria-label={t("common.or")}>/</span>{/if}<ShortcutHint shortcut={keys.join("+")} {platform} variant="keycaps" />{/each}</div></div>{/each}</section>{/each}

<style>
  h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}h1:focus{outline:none}.section-intro{margin:0 0 8px;color:var(--text-2);font-size:var(--fs-ui);line-height:1.5}.shortcut-help{max-width:72ch;margin:0 0 18px;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.55}.shortcut-help strong,.shortcut-row strong{color:var(--text-1);font-size:var(--fs-body)}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:var(--fs-ui);color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}.shortcut-row small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}.shortcut-actions,.key-list{flex:none;display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.shortcut-actions :global(kbd){min-width:92px;justify-content:center}.alternative{margin:0 2px;color:var(--text-3);font-size:var(--fs-meta);line-height:1.3}button{padding:8px 10px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);line-height:1.4}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}button.recording{color:var(--text-1);background:var(--bg-selected)}button:disabled{opacity:.45;cursor:not-allowed}.inline-error{margin:8px 0 0;color:var(--danger);font-size:var(--fs-ui)}
</style>
