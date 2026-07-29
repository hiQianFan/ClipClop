<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/history/api";
  import { applyTheme, getSettings, openLogDir, updateSettings, type Settings } from "./api";
  import { currentPlatform, defaultShortcut, shortcutFromKeyboardEvent, shortcutKeycaps, shortcutSpokenLabel, type ShortcutPlatform } from "./shortcuts";
  import { DEVELOPMENT_VERSION, openLatestRelease } from "$lib/updater/api";
  import { updateStore } from "$lib/updater/store.svelte";
  import { effectiveLocale, formatNumber, languagePreference, localizedError, localizedUpdateError, setLanguagePreference, t, type StaticMessageKey } from "$lib/i18n/index.svelte";
  import { CircleAlert, CircleCheck, Download, RefreshCw } from "@lucide/svelte";

  type Tab = "general" | "shortcuts" | "updates" | "about";
  type ShortcutRow = { name: StaticMessageKey; description: StaticMessageKey; keys: string[][] };
  const tabs: Tab[] = ["general", "shortcuts", "updates", "about"];

  let { onclose, oncleared }: { onclose: () => void; oncleared: () => void } = $props();
  let settings = $state<Settings | null>(null);
  let tab = $state<Tab>("general");
  let status = $state("");
  let saving = $state(false);
  // Update task state lives in the module-level store so an in-flight check or
  // download survives this view being closed and reopened. Read-only here.
  const appVersion = $derived(updateStore.appVersion);
  const update = $derived(updateStore.update);
  const updateState = $derived(updateStore.phase);
  const updateProgress = $derived(updateStore.progress);
  const updateBusy = $derived(updateStore.busy);
  // Derived so the message re-localizes on language change instead of freezing
  // at the locale that was active when the task ran.
  const updateMessage = $derived.by(() => {
    effectiveLocale();
    switch (updateState) {
      case "checking": return t("settings.checkingLong");
      case "current": return t("settings.current");
      case "downloading":
        return updateProgress === null
          ? t("settings.downloading")
          : t("settings.downloadingProgress", { progress: formatNumber(updateProgress) });
      case "installing": return t("settings.installing");
      case "error":
        return updateStore.errorSource === "unsupported"
          ? t("settings.devUpdate")
          : updateStore.errorSource === "install"
            ? t("settings.installFailed", { error: localizedUpdateError(updateStore.errorReason) })
            : t("settings.checkFailed", { error: localizedUpdateError(updateStore.errorReason) });
      default: return update ? t("settings.found", { version: update.version }) : "";
    }
  });
  let confirmClear = $state(false);
  let recording = $state(false);
  let shortcutError = $state("");
  let savedHotkey = $state("");
  let savedLaunchAtLogin = $state(false);
  let savedLanguage = $state<Settings["language"]>(languagePreference());
  let destroyed = false;
  let navButtons = $state<HTMLButtonElement[]>([]);
  let settingsContent = $state<HTMLElement>();
  let sectionHeading = $state<HTMLHeadingElement>();
  let clearTrigger = $state<HTMLButtonElement>();
  let confirmClearButton = $state<HTMLButtonElement>();
  let recorder = $state<HTMLButtonElement>();
  const platform: ShortcutPlatform = currentPlatform();

  const panelShortcuts: ShortcutRow[] = [
    { name: "shortcut.search", description: "shortcut.searchDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "F"], ["/"]] },
    { name: "shortcut.openSettings", description: "shortcut.openSettingsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", ","]] },
    { name: "shortcut.itemActions", description: "shortcut.itemActionsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "K"], ...(platform === "windows" ? [["Shift", "F10"]] : [])] },
    { name: "shortcut.menuNavigation", description: "shortcut.menuNavigationDesc", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
    { name: "shortcut.backLayers", description: "shortcut.backLayersDesc", keys: [["Escape"]] },
    { name: "shortcut.closePanel", description: "shortcut.closePanelDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "W"]] },
  ];
  const listShortcuts: ShortcutRow[] = [
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
  ];
  const fileShortcuts: ShortcutRow[] = [
    { name: "shortcut.previousNextFile", description: "shortcut.previousNextFileDesc", keys: [["ArrowLeft"], ["ArrowRight"]] },
    { name: "shortcut.firstLastFile", description: "shortcut.firstLastFileDesc", keys: [["Home"], ["End"]] },
  ];
  const settingsShortcuts: ShortcutRow[] = [
    { name: "shortcut.switchCategory", description: "shortcut.switchCategoryDesc", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
    { name: "shortcut.enterDetail", description: "shortcut.enterDetailDesc", keys: [["ArrowRight"], ["Tab"]] },
    { name: "shortcut.returnCategory", description: "shortcut.returnCategoryDesc", keys: [["ArrowLeft"]] },
    { name: "shortcut.saveSettings", description: "shortcut.saveSettingsDesc", keys: [[platform === "macos" ? "Command" : "Ctrl", "S"]] },
    { name: "shortcut.returnHistory", description: "shortcut.returnHistoryDesc", keys: [["Escape"]] },
  ];
  const shortcutGroups: [StaticMessageKey, ShortcutRow[]][] = [
    ["shortcut.group.panel", panelShortcuts],
    ["shortcut.group.list", listShortcuts],
    ["shortcut.group.files", fileShortcuts],
    ["shortcut.group.settings", settingsShortcuts],
  ];

  onMount(() => {
    requestAnimationFrame(() => navButtons[0]?.focus());
    void load();
  });
  onDestroy(() => {
    destroyed = true;
    setLanguagePreference(savedLanguage);
  });

  $effect(() => {
    effectiveLocale();
    untrack(() => {
      status = "";
      shortcutError = "";
    });
  });

  async function load() {
    void updateStore.hydrate();
    try {
      const loaded = await getSettings();
      if (destroyed) return;
      settings = loaded;
      savedHotkey = loaded.hotkey;
      savedLaunchAtLogin = loaded.launch_at_login;
      savedLanguage = loaded.language;
    }
    catch (reason) { if (!destroyed) status = t("settings.loadFailed", { error: localizedError(reason) }); }
  }

  function selectTab(next: Tab) {
    tab = next;
    recording = false;
    shortcutError = "";
  }

  async function focusDetail() {
    await tick();
    const first = settingsContent?.querySelector<HTMLElement>(
      'input:not([disabled]), select:not([disabled]), button:not([disabled]), [tabindex="0"]',
    );
    const target = first ?? sectionHeading;
    target?.focus();
    target?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }

  async function onNavKeydown(event: KeyboardEvent) {
    const current = tabs.indexOf(tab);
    let next = current;
    if (event.key === "ArrowDown") next = (current + 1) % tabs.length;
    else if (event.key === "ArrowUp") next = (current - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else if (event.key === "ArrowRight" || (event.key === "Tab" && !event.shiftKey)) {
      event.preventDefault();
      await focusDetail();
      return;
    } else return;
    event.preventDefault();
    selectTab(tabs[next]);
    await tick();
    navButtons[next]?.focus();
  }

  function onContentKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowLeft" || recording) return;
    event.preventDefault();
    event.stopPropagation();
    const target = navButtons[tabs.indexOf(tab)];
    target?.focus();
    target?.scrollIntoView({ block: "nearest" });
  }

  async function save() {
    if (!settings || saving) return;
    saving = true;
    status = t("settings.saving");
    try {
      settings = await updateSettings(settings);
      savedHotkey = settings.hotkey;
      savedLaunchAtLogin = settings.launch_at_login;
      savedLanguage = settings.language;
      setLanguagePreference(savedLanguage);
      applyTheme(settings.theme);
      recording = false;
      shortcutError = "";
      status = t("settings.saved");
    } catch (reason) {
      settings.hotkey = savedHotkey;
      settings.launch_at_login = savedLaunchAtLogin;
      settings.language = savedLanguage;
      setLanguagePreference(savedLanguage);
      status = t("settings.saveFailed", { error: localizedError(reason) });
      if (tab === "shortcuts") recorder?.focus();
    } finally {
      saving = false;
    }
  }

  function recordShortcut(event: KeyboardEvent) {
    if (!recording || !settings) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      recording = false;
      shortcutError = "";
      status = t("settings.recordCancelled");
      return;
    }
    const result = shortcutFromKeyboardEvent(event, platform);
    if (!result.valid) {
      shortcutError = t(result.code === "invalid_input" ? "shortcut.invalidInput" : result.code === "reserved" ? "shortcut.reserved" : "shortcut.invalidCombination");
      return;
    }
    settings.hotkey = result.shortcut;
    recording = false;
    shortcutError = "";
    status = t("settings.recorded", { shortcut: shortcutSpokenLabel(result.shortcut, platform) });
  }

  function restoreDefaultShortcut() {
    if (!settings) return;
    settings.hotkey = defaultShortcut(platform);
    recording = false;
    shortcutError = "";
    status = t("settings.restored");
  }

  async function requestClear() {
    confirmClear = true;
    await tick();
    confirmClearButton?.focus();
  }

  async function cancelClear() {
    confirmClear = false;
    await tick();
    clearTrigger?.focus();
  }

  // The store owns the async lifecycle; these just trigger it. State updates flow
  // back through the derived reads above, so closing/reopening the view mid-task
  // never loses progress.
  function checkUpdates() {
    void updateStore.check();
  }

  function installUpdate() {
    void (updateState === "error" ? updateStore.retry() : updateStore.install());
  }

  async function removeAll() {
    try {
      await clearHistory(); confirmClear = false; status = t("settings.cleared"); oncleared();
    } catch (reason) { confirmClear = false; status = t("settings.clearFailed", { error: localizedError(reason) }); await tick(); clearTrigger?.focus(); }
  }

  async function openLogs() {
    try { await openLogDir(); }
    catch (reason) { status = t("settings.openLogsFailed", { error: localizedError(reason) }); }
  }

  function displayKeys(keys: string[]) { return shortcutKeycaps(keys.join("+"), platform); }
  function speakKeys(keys: string[]) { return shortcutSpokenLabel(keys.join("+"), platform); }
  function previewLanguage() {
    if (!settings) return;
    setLanguagePreference(settings.language);
  }
  function displayVersion(version: string) {
    if (version === DEVELOPMENT_VERSION) return t("update.devVersion");
    return version === "__clipclop_unknown__" ? t("common.unknown") : version;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (recording) { recording = false; shortcutError = ""; status = t("settings.recordCancelled"); }
      else if (confirmClear) void cancelClear();
      else onclose();
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault(); void save();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="settings-shell">
  <div class="settings-body">
    <div class="settings-nav" role="tablist" aria-orientation="vertical" aria-label={t("settings.categories")}>
      <button bind:this={navButtons[0]} id="settings-tab-general" role="tab" aria-controls="settings-panel" aria-selected={tab === "general"} tabindex={tab === "general" ? 0 : -1} class:active={tab === "general"} onclick={() => selectTab("general")} onkeydown={onNavKeydown}>{t("settings.general")}</button>
      <button bind:this={navButtons[1]} id="settings-tab-shortcuts" role="tab" aria-controls="settings-panel" aria-selected={tab === "shortcuts"} tabindex={tab === "shortcuts" ? 0 : -1} class:active={tab === "shortcuts"} onclick={() => selectTab("shortcuts")} onkeydown={onNavKeydown}>{t("settings.shortcuts")}</button>
      <button bind:this={navButtons[2]} id="settings-tab-updates" role="tab" aria-controls="settings-panel" aria-selected={tab === "updates"} tabindex={tab === "updates" ? 0 : -1} class:active={tab === "updates"} onclick={() => selectTab("updates")} onkeydown={onNavKeydown}>{t("settings.updates")}</button>
      <button bind:this={navButtons[3]} id="settings-tab-about" role="tab" aria-controls="settings-panel" aria-selected={tab === "about"} tabindex={tab === "about" ? 0 : -1} class:active={tab === "about"} onclick={() => selectTab("about")} onkeydown={onNavKeydown}>{t("settings.about")}</button>
    </div>
    <div bind:this={settingsContent} id="settings-panel" class="settings-content" role="tabpanel" aria-labelledby={`settings-tab-${tab}`} tabindex="-1" onkeydown={onContentKeydown}>
      {#if settings}
        {#if tab === "general"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.general")}</h1>
          <label><span><strong>{t("settings.launch")}</strong><small>{t("settings.launchHelp")}</small></span><input type="checkbox" bind:checked={settings.launch_at_login} /></label>
          <label><span><strong>{t("settings.retention")}</strong><small>{t("settings.retentionHelp")}</small></span><select bind:value={settings.retention_days}><option value={7}>{t("settings.days", { count: formatNumber(7) })}</option><option value={30}>{t("settings.days", { count: formatNumber(30) })}</option><option value={90}>{t("settings.days", { count: formatNumber(90) })}</option></select></label>
          <label><span><strong>{t("settings.appearance")}</strong><small>{t("settings.appearanceHelp")}</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">{t("settings.followSystem")}</option><option value="light">{t("settings.light")}</option><option value="dark">{t("settings.dark")}</option></select></label>
          <label><span><strong>{t("settings.language")}</strong><small>{t("settings.languageHelp")}</small></span><select bind:value={settings.language} onchange={previewLanguage}><option value="system">{t("settings.languageSystem")}</option><option value="zh-CN">{t("settings.languageChinese")}</option><option value="en">{t("settings.languageEnglish")}</option></select></label>
          <div class="row"><span><strong>{t("settings.data")}</strong><small>{t("settings.dataHelp")}</small></span><button bind:this={clearTrigger} class="danger" onclick={requestClear}>{t("settings.clearAll")}</button></div>
        {:else if tab === "shortcuts"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.shortcuts")}</h1>
          <p class="section-intro">{t("settings.shortcutIntro")}</p>
          <p class="shortcut-help">
            <strong>{t("settings.shortcutHow")}</strong>{t("settings.shortcutHowHelp")}
            {#if platform === "macos"}{t("settings.macKeyHelp")}{:else}{t("settings.windowsKeyHelp")}{/if}
          </p>
          <section class="shortcut-group" aria-labelledby="global-shortcut-title">
            <h2 id="global-shortcut-title">{t("settings.global")}</h2>
            <div class="shortcut-row editable">
              <span><strong>{t("settings.toggle")}</strong><small>{t("settings.toggleHelp")}</small></span>
              <div class="shortcut-actions">
                <kbd class="key-combination" aria-label={t("settings.currentShortcut", { shortcut: shortcutSpokenLabel(settings.hotkey, platform) })}>
                  {#each shortcutKeycaps(settings.hotkey, platform) as key, index}{#if index > 0}<span class="key-plus" aria-hidden="true">+</span>{/if}<span class="keycap" aria-hidden="true">{key}</span>{/each}
                </kbd>
                <button bind:this={recorder} class:recording onclick={() => { recording = true; shortcutError = ""; status = t("settings.recordPrompt"); }} onkeydown={recordShortcut}>{recording ? t("settings.pressShortcut") : t("settings.change")}</button>
                <button onclick={restoreDefaultShortcut} disabled={settings.hotkey === defaultShortcut(platform)}>{t("settings.restoreDefault")}</button>
              </div>
            </div>
            {#if shortcutError}<p class="inline-error" role="alert">{shortcutError}</p>{/if}
          </section>
          {#each shortcutGroups as group}
            <section class="shortcut-group" aria-labelledby={`shortcut-${group[0]}`}>
              <h2 id={`shortcut-${group[0]}`}>{t(group[0])}</h2>
              {#each group[1] as item}
                <div class="shortcut-row"><span><strong>{t(item.name)}</strong><small>{t(item.description)}</small></span><div class="key-list">{#each item.keys as keys, alternativeIndex}{#if alternativeIndex > 0}<span class="alternative" aria-label={t("common.or")}>/</span>{/if}<kbd class="key-combination" aria-label={speakKeys(keys)}>{#each displayKeys(keys) as key, keyIndex}{#if keyIndex > 0}<span class="key-plus" aria-hidden="true">+</span>{/if}<span class="keycap" aria-hidden="true">{key}</span>{/each}</kbd>{/each}</div></div>
              {/each}
            </section>
          {/each}
        {:else if tab === "updates"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.updates")}</h1>
          <div class="update-head"><span><strong>{t("settings.updateHeading")}</strong><small>{t("settings.versionHelp", { version: displayVersion(appVersion) })}</small></span><div class="update-head-controls"><label><span>{t("settings.autoCheck")}</span><input type="checkbox" bind:checked={settings.check_updates} /></label><button class="update-check-btn" disabled={updateState === "checking"} aria-busy={updateState === "checking"} onclick={checkUpdates}><RefreshCw size={14} class={updateState === "checking" ? "spin" : ""} />{updateState === "checking" ? t("settings.checking") : t("settings.check")}</button></div></div>
          {#if update}
            <div class="update-card">
              <div class="update-card-head">
                <span class="update-badge" aria-hidden="true"><Download size={17} /></span>
                <span class="update-card-title"><strong>{t("settings.updateAvailable", { version: update.version })}</strong><small>{t("settings.versionHelp", { version: displayVersion(appVersion) })}</small></span>
              </div>
              {#if update.notes}<p class="update-notes">{update.notes}</p>{/if}
              {#if updateBusy}
                <div class="update-progress">
                  <div class="progress-track"><div class="progress-fill" class:indeterminate={updateProgress === null} style={updateProgress === null ? "" : `width:${updateProgress}%`}></div></div>
                  <span class="progress-label">{updateState === "installing" ? t("settings.installing") : updateProgress === null ? t("settings.downloading") : t("settings.downloadingProgress", { progress: formatNumber(updateProgress) })}</span>
                </div>
              {:else if updateState === "error"}
                <p class="update-status error" role="alert"><CircleAlert size={14} /><span>{updateMessage}</span></p>
              {/if}
              <div class="update-actions">
                <button onclick={() => void openLatestRelease()}>{t("settings.releasePage")}</button>
                <button class="primary" disabled={updateBusy} onclick={installUpdate}>{updateState === "error" ? t("settings.retry") : t("settings.install")}</button>
              </div>
            </div>
          {:else}
            <div class="update-check">
              <span class="update-status" class:error={updateState === "error"} aria-live="polite">
                {#if updateState === "current"}<CircleCheck size={15} />{:else if updateState === "error"}<CircleAlert size={15} />{/if}
                <span>{updateMessage || t("settings.upToDate", { version: displayVersion(appVersion) })}</span>
              </span>
            </div>
          {/if}
        {:else}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1" class="visually-hidden">{t("settings.about")}</h1>
          <div class="about"><img src="/app-icon.png" alt={t("settings.iconAlt")} /><h2>ClipClop</h2><p>{t("settings.tagline")}</p><small>{t("settings.version", { version: displayVersion(appVersion) })}</small><button aria-label={t("settings.github")} onclick={() => void openUrl("https://github.com/hiQianFan/ClipClop")}>GitHub</button>
            <button class="log-door" title={t("settings.diagnosticsHelp")} onclick={() => void openLogs()}>{t("settings.diagnostics")}</button>
          </div>
        {/if}
      {:else}<div class="loading" role="status">{status || t("settings.loading")}</div>{/if}
    </div>
  </div>
  <footer>
    {#if confirmClear}<strong>{t("settings.clearConfirm")}</strong><button onclick={cancelClear}>{t("common.cancel")}</button><button bind:this={confirmClearButton} class="danger" onclick={() => void removeAll()}>{t("settings.clear")}</button>
    {:else}<span aria-live="polite" aria-atomic="true">{status}</span><button onclick={onclose}>{t("common.back")}</button>{#if tab !== "about"}<button class="primary" onclick={() => void save()} disabled={!settings || saving} aria-busy={saving}>{t("common.save")}</button>{/if}{/if}
  </footer>
</div>

<style>
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}.settings-body{min-height:0;display:grid;grid-template-columns:clamp(168px,22%,192px) minmax(0,1fr)}.settings-nav{display:flex;flex-direction:column;gap:3px;padding:14px 12px;border-right:1px solid var(--hairline)}button{padding:8px 10px;border-radius:6px;color:var(--text-2);background:transparent;font-size:12px;line-height:1.4}.settings-nav button{min-height:40px;padding:0 12px;text-align:left;font-size:13px;font-weight:600}.settings-nav button:hover,.settings-nav button.active,button:hover{color:var(--text-1);background:var(--bg-hover)}.settings-nav button.active{background:var(--bg-selected)}button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.settings-nav button:focus-visible{outline:none;box-shadow:inset 0 0 0 2px var(--text-1)}.settings-content{min-width:0;min-height:0;overflow:auto;padding:0 24px 20px}.settings-content h1{margin:18px 0 4px;font-size:18px;line-height:1.3}.settings-content h1:focus{outline:none}.section-intro{margin:0 0 8px;color:var(--text-2);font-size:12px;line-height:1.5}.shortcut-help{max-width:72ch;margin:0 0 18px;padding:9px 11px;border-radius:6px;color:var(--text-2);background:var(--bg-raised);font-size:12px;line-height:1.55}.shortcut-help strong{color:var(--text-1)}.settings-content>label,.row,.update-head{min-height:68px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}label>span,.row>span,.update-head>span,.shortcut-row>span{display:flex;flex-direction:column;gap:3px}strong{font-size:13px}small{color:var(--text-3);font-size:12px;line-height:1.4}select{min-width:116px;padding:7px;border:1px solid var(--hairline);border-radius:6px;color:var(--text-1);background:var(--bg-raised);font-size:12px}input{width:18px;height:18px}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:12px;color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-actions,.key-list{display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.key-combination{display:flex;align-items:center;gap:4px;border:0;background:transparent}.shortcut-actions .key-combination{min-width:92px;justify-content:center}.keycap{padding:3px 6px;border:1px solid var(--hairline);border-radius:4px;color:var(--text-1);background:var(--bg-raised);font:12px/1.3 ui-monospace,monospace;white-space:nowrap}.key-plus,.alternative{color:var(--text-3);font-size:11px;line-height:1.3}.alternative{margin:0 2px}.recording{color:var(--text-1);background:var(--bg-selected)}.inline-error{margin:8px 0 0;color:var(--danger);font-size:12px}.update-head label{display:flex;align-items:center;gap:8px}.update-head-controls{display:flex;align-items:center;gap:16px}.update-head-controls .update-check-btn{flex:none;padding:7px 12px;border:1px solid var(--hairline);border-radius:6px}.update-card{display:flex;flex-direction:column;gap:12px;margin-top:16px;padding:16px;border:1px solid var(--hairline);border-radius:10px;background:var(--bg-raised)}.update-card-head{display:flex;align-items:center;gap:12px}.update-badge{flex:none;display:grid;place-items:center;width:34px;height:34px;border-radius:9px;color:var(--action-on);background:var(--action)}.update-card-title{display:flex;flex-direction:column;gap:2px}.update-card-title strong{font-size:14px}.update-notes{max-height:116px;margin:0;padding:10px 12px;overflow:auto;white-space:pre-wrap;border-radius:7px;color:var(--text-2);background:var(--bg-shell);font-size:12px;line-height:1.55}.update-progress{display:flex;flex-direction:column;gap:7px}.progress-track{height:6px;overflow:hidden;border-radius:999px;background:var(--bg-selected)}.progress-fill{height:100%;border-radius:999px;background:var(--action);transition:width .25s ease}.progress-fill.indeterminate{width:35%;animation:progress-slide 1.1s ease-in-out infinite}.progress-label{color:var(--text-2);font-size:12px;font-variant-numeric:tabular-nums}.update-status{display:flex;align-items:center;gap:6px;font-size:12px;line-height:1.4}.update-status :global(svg){flex:none}.update-card .update-status{margin:0}.update-actions,.update-check{display:flex;align-items:center;gap:8px}.update-actions{justify-content:flex-end}.update-check{justify-content:space-between;margin-top:16px}.update-check-btn{display:inline-flex;align-items:center;gap:6px}.update-check-btn :global(svg.spin){animation:spin 1s linear infinite}@keyframes progress-slide{0%{transform:translateX(-120%)}100%{transform:translateX(340%)}}@keyframes spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.progress-fill.indeterminate,.update-check-btn :global(svg.spin){animation:none}}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about{position:relative}.about img{width:56px;height:56px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:12px}.log-door{position:absolute;bottom:8px;left:50%;transform:translateX(-50%);min-height:0;padding:4px 8px;color:var(--text-3);font-size:11px;font-weight:400;opacity:.7}.log-door:hover{color:var(--text-2);background:transparent;opacity:1}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}footer button{min-width:72px}footer .primary{min-width:92px}.primary{color:var(--action-on);background:var(--action)}.primary:hover:not(:disabled){color:var(--action-on);background:var(--action-hover)}.danger,.error{color:var(--danger)}.danger:hover:not(:disabled){color:var(--danger-on);background:var(--danger-fill)}button:disabled{opacity:.45;cursor:not-allowed}button:disabled:hover{background:transparent}.primary:disabled:hover{background:var(--action)}.visually-hidden{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
</style>
