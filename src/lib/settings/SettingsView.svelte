<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/history/api";
  import { openAutoPasteSettings } from "$lib/onboarding/api";
  import { applyTheme, getSettings, openFilePreviewSettings, openLogDir, updateSettings, type Settings } from "./api";
  import { currentPlatform, defaultShortcut, shortcutFromKeyboardEvent, shortcutKeycaps, shortcutSpokenLabel, type ShortcutPlatform } from "./shortcuts";
  import { DEVELOPMENT_VERSION, listReleaseNotes, openLatestRelease, type ReleaseNote } from "$lib/updater/api";
  import { updateStore } from "$lib/updater/store.svelte";
  import { effectiveLocale, formatDateTime, formatNumber, localizedError, localizedUpdateError, setLanguagePreference, t, type StaticMessageKey } from "$lib/i18n/index.svelte";
  import { CircleAlert, CircleCheck, Download, RefreshCw } from "@lucide/svelte";

  type Tab = "general" | "history" | "appearance" | "shortcuts" | "updates" | "about";
  type ShortcutRow = { name: StaticMessageKey; description: StaticMessageKey; keys: string[][] };
  const tabs: Tab[] = ["general", "history", "appearance", "shortcuts", "updates", "about"];

  let { initialTab = "general", onclose, oncleared, onquickstart }: { initialTab?: Tab; onclose: () => void; oncleared: () => void; onquickstart: () => void } = $props();
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
      case "skipped": return t("settings.skippedVersion", { version: updateStore.skippedVersion ?? "" });
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
  let savedSettings = $state<Settings | null>(null);
  let destroyed = false;
  let navButtons = $state<HTMLButtonElement[]>([]);
  let settingsContent = $state<HTMLElement>();
  let sectionHeading = $state<HTMLHeadingElement>();
  let clearTrigger = $state<HTMLButtonElement>();
  let confirmClearButton = $state<HTMLButtonElement>();
  let recorder = $state<HTMLButtonElement>();
  let releases = $state<ReleaseNote[]>([]);
  let selectedRelease = $state<ReleaseNote | null>(null);
  let releasesLoading = $state(false);
  let releasesError = $state("");
  let releaseList = $state<HTMLDivElement>();
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
    tab = initialTab;
    requestAnimationFrame(() => navButtons[tabs.indexOf(tab)]?.focus());
    void load();
    void loadReleases();
  });
  onDestroy(() => {
    destroyed = true;
    if (savedSettings) {
      applyTheme(savedSettings.theme);
      setLanguagePreference(savedSettings.language);
    }
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
      savedSettings = { ...loaded };
      applyTheme(loaded.theme);
      setLanguagePreference(loaded.language);
    }
    catch (reason) { if (!destroyed) status = t("settings.loadFailed", { error: localizedError(reason) }); }
  }

  async function openAutoPasteSystemSettings() {
    try { await openAutoPasteSettings(); }
    catch (reason) { status = localizedError(reason); }
  }

  // The button is a pure shortcut to macOS Full Disk Access; it never records or
  // reflects any state. The in-app switch below is the authoritative gate and is
  // persisted through the normal save() flow like every other setting.
  async function openFilePreviewSystemSettings() {
    try { await openFilePreviewSettings(); }
    catch (reason) { status = localizedError(reason); }
  }

  async function loadReleases() {
    releasesLoading = true;
    releasesError = "";
    try {
      releases = await listReleaseNotes();
      selectedRelease = releaseForVersion(update?.version) ?? releases[0] ?? null;
    } catch (reason) {
      releasesError = localizedError(reason);
    } finally {
      releasesLoading = false;
    }
  }

  async function openReleasePage() {
    try { await openLatestRelease(); }
    catch (reason) { status = localizedError(reason); }
  }

  function releaseForVersion(version: string | undefined) {
    if (!version) return undefined;
    const normalized = version.replace(/^v/i, "");
    return releases.find((release) => release.version.replace(/^v/i, "") === normalized);
  }

  $effect(() => {
    const release = releaseForVersion(update?.version);
    if (release) selectedRelease = release;
  });

  function selectRelease(index: number) {
    const release = releases[index];
    if (!release) return;
    selectedRelease = release;
    requestAnimationFrame(() => releaseList?.querySelector<HTMLElement>(`[data-release-index="${index}"]`)?.scrollIntoView({ block: "nearest" }));
  }

  function onReleaseListClick(event: MouseEvent) {
    const option = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-release-index]") : null;
    if (option) selectRelease(Number(option.dataset.releaseIndex));
  }

  function onReleaseListKeydown(event: KeyboardEvent) {
    const current = Math.max(0, releases.findIndex((release) => release.version === selectedRelease?.version));
    const pageSize = Math.max(1, Math.floor((releaseList?.clientHeight ?? 36) / 36));
    let next = current;
    if (event.key === "ArrowDown") next += 1;
    else if (event.key === "ArrowUp") next -= 1;
    else if (event.key === "PageDown") next += pageSize;
    else if (event.key === "PageUp") next -= pageSize;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = releases.length - 1;
    else return;
    event.preventDefault();
    selectRelease(Math.min(Math.max(next, 0), releases.length - 1));
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
    if (
      event.key !== "ArrowLeft"
      || recording
      || event.defaultPrevented
      || event.target instanceof HTMLInputElement
      || event.target instanceof HTMLSelectElement
      || event.target instanceof HTMLTextAreaElement
      || (event.target instanceof Element && event.target.closest("[role='listbox']"))
    ) return;
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
      const saved = await updateSettings({ ...settings });
      settings = saved;
      savedSettings = { ...saved };
      setLanguagePreference(saved.language);
      applyTheme(saved.theme);
      recording = false;
      shortcutError = "";
      status = t("settings.saved");
    } catch (reason) {
      if (savedSettings) {
        settings = { ...savedSettings };
        applyTheme(savedSettings.theme);
        setLanguagePreference(savedSettings.language);
      }
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

  function skipUpdate() { void updateStore.skip(); }

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
      <button bind:this={navButtons[1]} id="settings-tab-history" role="tab" aria-controls="settings-panel" aria-selected={tab === "history"} tabindex={tab === "history" ? 0 : -1} class:active={tab === "history"} onclick={() => selectTab("history")} onkeydown={onNavKeydown}>{t("settings.history")}</button>
      <button bind:this={navButtons[2]} id="settings-tab-appearance" role="tab" aria-controls="settings-panel" aria-selected={tab === "appearance"} tabindex={tab === "appearance" ? 0 : -1} class:active={tab === "appearance"} onclick={() => selectTab("appearance")} onkeydown={onNavKeydown}>{t("settings.appearance")}</button>
      <button bind:this={navButtons[3]} id="settings-tab-shortcuts" role="tab" aria-controls="settings-panel" aria-selected={tab === "shortcuts"} tabindex={tab === "shortcuts" ? 0 : -1} class:active={tab === "shortcuts"} onclick={() => selectTab("shortcuts")} onkeydown={onNavKeydown}>{t("settings.shortcuts")}</button>
      <span class="nav-separator" aria-hidden="true"></span>
      <button bind:this={navButtons[4]} id="settings-tab-updates" role="tab" aria-controls="settings-panel" aria-selected={tab === "updates"} tabindex={tab === "updates" ? 0 : -1} class:active={tab === "updates"} onclick={() => selectTab("updates")} onkeydown={onNavKeydown}>{t("settings.updates")}</button>
      <button bind:this={navButtons[5]} id="settings-tab-about" role="tab" aria-controls="settings-panel" aria-selected={tab === "about"} tabindex={tab === "about" ? 0 : -1} class:active={tab === "about"} onclick={() => selectTab("about")} onkeydown={onNavKeydown}>{t("settings.about")}</button>
    </div>
    <div bind:this={settingsContent} id="settings-panel" class="settings-content" class:updates-content={tab === "updates"} role="tabpanel" aria-labelledby={`settings-tab-${tab}`} tabindex="-1" onkeydown={onContentKeydown}>
      {#if settings}
        {#if tab === "general"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.general")}</h1>
          <div class="row setting-row"><span><strong id="launch-label">{t("settings.launch")}</strong><small id="launch-help">{t("settings.launchHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="launch-label" aria-describedby="launch-help" bind:checked={settings.launch_at_login} /><span class="switch-track"></span></label></div>
          <div class="row"><span><strong>{t("settings.quickStart")}</strong><small>{t("settings.quickStartHelp")}</small></span><button onclick={onquickstart}>{t("settings.quickStart")}</button></div>
          {#if platform === "macos"}<div class="row"><span><strong>{t("settings.autoPaste")}</strong><small>{t("settings.autoPasteHelp")}</small></span><button onclick={() => void openAutoPasteSystemSettings()}>{t("settings.manage")}</button></div>{/if}
          {#if platform === "macos"}<div class="row"><span><strong id="file-preview-label">{t("settings.filePreview")}</strong><small id="file-preview-help">{t("settings.filePreviewHelp")}</small></span><div class="row-actions"><button onclick={() => void openFilePreviewSystemSettings()}>{t("settings.manage")}</button><label class="switch"><input type="checkbox" role="switch" aria-labelledby="file-preview-label" aria-describedby="file-preview-help" bind:checked={settings.file_preview_enabled} /><span class="switch-track"></span></label></div></div>{/if}
        {:else if tab === "history"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.history")}</h1>
          <label><span><strong>{t("settings.retention")}</strong><small>{t("settings.retentionHelp")}</small></span><select bind:value={settings.retention_days}><option value={1}>{t("settings.days", { count: formatNumber(1) })}</option><option value={7}>{t("settings.days", { count: formatNumber(7) })}</option><option value={30}>{t("settings.days", { count: formatNumber(30) })}</option><option value={90}>{t("settings.days", { count: formatNumber(90) })}</option><option value={365}>{t("settings.year")}</option><option value={null}>{t("settings.forever")}</option></select></label>
          <label><span><strong>{t("settings.historyLimit")}</strong><small>{t("settings.historyLimitHelp")}</small></span><select bind:value={settings.history_limit}><option value={100}>{t("settings.items", { count: formatNumber(100) })}</option><option value={500}>{t("settings.items", { count: formatNumber(500) })}</option><option value={1000}>{t("settings.items", { count: formatNumber(1000) })}</option><option value={5000}>{t("settings.items", { count: formatNumber(5000) })}</option><option value={null}>{t("settings.unlimited")}</option></select></label>
          <div class="row setting-row"><span><strong id="move-used-label">{t("settings.moveUsedToTop")}</strong><small id="move-used-help">{t("settings.moveUsedToTopHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="move-used-label" aria-describedby="move-used-help" bind:checked={settings.move_used_to_top} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="restore-pos-label">{t("settings.restoreBrowsePosition")}</strong><small id="restore-pos-help">{t("settings.restoreBrowsePositionHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="restore-pos-label" aria-describedby="restore-pos-help" bind:checked={settings.restore_browse_position} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="trim-whitespace-label">{t("settings.trimWhitespace")}</strong><small id="trim-whitespace-help">{t("settings.trimWhitespaceHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="trim-whitespace-label" aria-describedby="trim-whitespace-help" bind:checked={settings.trim_whitespace} /><span class="switch-track"></span></label></div>
          {#if settings.retention_days === null || settings.history_limit === null}<p class="retention-warning">{t("settings.retentionWarning")}</p>{/if}
          <div class="row"><span><strong>{t("settings.data")}</strong><small>{t("settings.dataHelp")}</small></span><button bind:this={clearTrigger} class="danger" onclick={requestClear}>{t("settings.clearAll")}</button></div>
        {:else if tab === "appearance"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.appearance")}</h1>
          <label><span><strong>{t("settings.theme")}</strong><small>{t("settings.appearanceHelp")}</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">{t("settings.followSystem")}</option><option value="light">{t("settings.light")}</option><option value="dark">{t("settings.dark")}</option></select></label>
          <label><span><strong>{t("settings.language")}</strong><small>{t("settings.languageHelp")}</small></span><select bind:value={settings.language} onchange={previewLanguage}><option value="system">{t("settings.languageSystem")}</option><option value="zh-CN">{t("settings.languageChinese")}</option><option value="en">{t("settings.languageEnglish")}</option></select></label>
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
          <div class="updates-layout">
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.updates")}</h1>
          <div class="update-head"><span><strong>{t("settings.updateHeading")}</strong><small>{t("settings.versionHelp", { version: displayVersion(appVersion) })}</small></span><div class="update-head-controls"><span id="auto-check-label">{t("settings.autoCheck")}</span><label class="switch compact-switch"><input type="checkbox" role="switch" aria-labelledby="auto-check-label" bind:checked={settings.check_updates} /><span class="switch-track"></span></label></div></div>
          <div class="update-check update-head-controls">
            <span class="update-status" class:error={updateState === "error"} aria-live="polite">
              {#if updateState === "current"}<CircleCheck size={15} />{:else if updateState === "error"}<CircleAlert size={15} />{/if}
              <span>{updateMessage || t("settings.upToDate", { version: displayVersion(appVersion) })}</span>
            </span>
            <div class="update-actions"><button onclick={() => void openReleasePage()}>{t("settings.releasePage")}</button><button class="update-check-btn" disabled={updateState === "checking"} aria-busy={updateState === "checking"} onclick={checkUpdates}><RefreshCw size={14} class={updateState === "checking" ? "spin" : ""} />{updateState === "checking" ? t("settings.checking") : t("settings.check")}</button></div>
          </div>
          {#if update}
            <div class="update-card">
              <div class="update-card-head">
                <span class="update-badge" aria-hidden="true"><Download size={17} /></span>
                <span class="update-card-title"><strong>{t("settings.updateAvailable", { version: update.version })}</strong><small>{t("settings.versionHelp", { version: displayVersion(appVersion) })}</small></span>
              </div>
              {#if updateBusy}
                <div class="update-progress">
                  <div class="progress-track"><div class="progress-fill" class:indeterminate={updateProgress === null} style={updateProgress === null ? "" : `width:${updateProgress}%`}></div></div>
                  <span class="progress-label">{updateState === "installing" ? t("settings.installing") : updateProgress === null ? t("settings.downloading") : t("settings.downloadingProgress", { progress: formatNumber(updateProgress) })}</span>
                </div>
              {:else if updateState === "error"}
                <p class="update-status error" role="alert"><CircleAlert size={14} /><span>{updateMessage}</span></p>
              {/if}
              <div class="update-actions">
                <button disabled={updateBusy} onclick={skipUpdate}>{t("settings.skipVersion")}</button>
                <button class="primary" disabled={updateBusy} onclick={installUpdate}>{updateState === "error" ? t("settings.retry") : t("settings.install")}</button>
              </div>
            </div>
          {/if}
          <section class="release-history" aria-label={t("settings.releaseNotes")}>
            {#if releasesLoading}
              <div class="release-browser release-loading" aria-busy="true" aria-label={t("settings.loading")}>
                <div class="release-list" aria-hidden="true">{#each Array(7) as _}<span class="release-skeleton release-skeleton-row"></span>{/each}</div>
                <div class="release-detail" aria-hidden="true"><span class="release-skeleton release-skeleton-title"></span><span class="release-skeleton release-skeleton-line"></span><span class="release-skeleton release-skeleton-line"></span><span class="release-skeleton release-skeleton-line short"></span></div>
              </div>
            {:else if releasesError}
              <div class="release-browser release-load-failed"><div class="release-load-message"><p class="inline-error" role="alert">{releasesError}</p><button onclick={() => void loadReleases()}>{t("settings.refreshReleaseNotes")}</button></div></div>
            {:else if releases.length}
              <div class="release-browser">
                <div bind:this={releaseList} class="release-list" role="listbox" aria-label={t("settings.releaseNotes")} aria-activedescendant={selectedRelease ? `release-option-${selectedRelease.version}` : undefined} tabindex="0" onclick={onReleaseListClick} onkeydown={onReleaseListKeydown}>
                  {#each releases as release, index}<div id={`release-option-${release.version}`} class="release-option" class:active={selectedRelease?.version === release.version} data-release-index={index} role="option" aria-selected={selectedRelease?.version === release.version}><strong>{release.version}</strong><small>{formatDateTime(release.publishedAt)}</small></div>{/each}
                </div>
                {#if selectedRelease}<article class="release-detail"><header><span><span class="release-detail-title"><strong>{selectedRelease.version}</strong>{#if selectedRelease.isLatest}<em>{t("settings.latestRelease")}</em>{/if}</span><small>{formatDateTime(selectedRelease.publishedAt)}</small></span></header><div class="release-body" class:raw-release-body={!selectedRelease.notesHtml}>{#if selectedRelease.notesHtml}{@html selectedRelease.notesHtml}{:else}{selectedRelease.notes}{/if}</div></article>{/if}
              </div>
            {/if}
          </section>
          </div>
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
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}.settings-body{min-height:0;display:grid;grid-template-columns:clamp(168px,22%,192px) minmax(0,1fr)}.settings-nav{display:flex;flex-direction:column;gap:3px;padding:14px 12px;border-right:1px solid var(--hairline)}button{padding:8px 10px;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);line-height:1.4}.settings-nav button{min-height:40px;padding:0 12px;text-align:left;font-size:var(--fs-body);font-weight:600}.settings-nav button:hover,.settings-nav button.active,button:hover{color:var(--text-1);background:var(--bg-hover)}.settings-nav button.active{background:var(--bg-selected)}button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.settings-nav button:focus-visible{outline:none;box-shadow:inset 0 0 0 2px var(--text-1)}.settings-content{min-width:0;min-height:0;overflow:auto;padding:0 24px 20px}.settings-content h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}.settings-content h1:focus{outline:none}.section-intro{margin:0 0 8px;color:var(--text-2);font-size:var(--fs-ui);line-height:1.5}.shortcut-help{max-width:72ch;margin:0 0 18px;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.55}.shortcut-help strong{color:var(--text-1)}.settings-content>label,.row,.update-head{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}label>span,.row>span,.update-head>span,.shortcut-row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}strong{font-size:var(--fs-body)}small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}select{min-width:116px;padding:7px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-1);background:var(--bg-raised);font-size:var(--fs-ui)}input{width:18px;height:18px}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:var(--fs-ui);color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-actions,.key-list{display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.key-combination{display:flex;align-items:center;gap:4px;border:0;background:transparent}.shortcut-actions .key-combination{min-width:92px;justify-content:center}.keycap{padding:3px 6px;border:1px solid var(--hairline);border-radius:var(--radius-sm);color:var(--text-1);background:var(--bg-raised);font:var(--fs-ui)/var(--lh-snug) ui-monospace,monospace;white-space:nowrap}.key-plus,.alternative{color:var(--text-3);font-size:var(--fs-meta);line-height:1.3}.alternative{margin:0 2px}.recording{color:var(--text-1);background:var(--bg-selected)}.inline-error{margin:8px 0 0;color:var(--danger);font-size:var(--fs-ui)}.update-head label{display:flex;align-items:center;gap:8px}.update-head-controls{display:flex;align-items:center;gap:16px}.update-head-controls .update-check-btn{flex:none;border-radius:var(--radius-md)}.update-card{display:flex;flex-direction:column;gap:12px;margin-top:16px;padding:16px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised)}.update-card-head{display:flex;align-items:center;gap:12px}.update-badge{flex:none;display:grid;place-items:center;width:34px;height:34px;border-radius:var(--radius-lg);color:var(--action-on);background:var(--action)}.update-card-title{display:flex;flex-direction:column;gap:2px}.update-card-title strong{font-size:var(--fs-emphasis)}.update-progress{display:flex;flex-direction:column;gap:7px}.progress-track{height:6px;overflow:hidden;border-radius:var(--radius-pill);background:var(--bg-selected)}.progress-fill{height:100%;border-radius:var(--radius-pill);background:var(--action);transition:width var(--dur-slow) ease}.progress-fill.indeterminate{width:35%;animation:progress-slide 1.1s ease-in-out infinite}.progress-label{color:var(--text-2);font-size:var(--fs-ui);font-variant-numeric:tabular-nums}.update-status{display:flex;align-items:center;gap:6px;font-size:var(--fs-ui);line-height:1.4}.update-status :global(svg){flex:none}.update-card .update-status{margin:0}.update-actions,.update-check{display:flex;align-items:center;gap:8px}.update-actions{justify-content:flex-end}.update-check{justify-content:space-between;margin-top:16px}.update-check-btn{display:inline-flex;align-items:center;gap:6px}.update-check-btn :global(svg.spin){animation:spin 1s linear infinite}@keyframes progress-slide{0%{transform:translateX(-120%)}100%{transform:translateX(340%)}}@keyframes spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.progress-fill.indeterminate,.update-check-btn :global(svg.spin){animation:none}}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about{position:relative}.about img{width:56px;height:56px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:var(--fs-ui)}.log-door{position:absolute;bottom:8px;left:50%;transform:translateX(-50%);min-height:0;padding:4px 8px;color:var(--text-3);font-size:var(--fs-meta);font-weight:400;opacity:.7}.log-door:hover{color:var(--text-2);background:transparent;opacity:1}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}footer button{min-width:92px;min-height:32px;padding:0 12px}.primary{color:var(--action-on);background:var(--action)}.primary:hover:not(:disabled){color:var(--action-on);background:var(--action-hover)}.danger,.error{color:var(--danger)}footer .danger{color:var(--danger-on);background:var(--danger-fill);font-weight:600}.danger:hover:not(:disabled){color:var(--danger-on);background:var(--danger-fill)}button:disabled{opacity:.45;cursor:not-allowed}button:disabled:hover{background:transparent}.primary:disabled:hover{background:var(--action)}.visually-hidden{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  .nav-separator{height:1px;margin:8px 6px;background:var(--hairline)}
  /* Setting-row contract: text zone flexes (rule above), action zone is protected
     and never compresses. Every row's action lives in one of these. */
  .row>button,.row>.row-actions,.settings-content>label>select,.update-head>.update-head-controls,.shortcut-row>.shortcut-actions,.shortcut-row>.key-list{flex:none}
  /* Action group: button + switch riding together in one row's action zone. */
  .row-actions{display:flex;align-items:center;gap:8px}
  /* Unified action-button sizing across every section (ghost per DESIGN.md). */
  .row>button,.row-actions>button,.update-check-btn{min-height:32px;padding:0 12px;white-space:nowrap}
  .switch{position:relative;flex:none;width:44px;height:44px;cursor:pointer}.switch input{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);clip-path:inset(50%);white-space:nowrap}.switch-track{position:absolute;left:4px;top:12px;width:36px;height:20px;border:1px solid color-mix(in srgb,var(--text-2) 42%,var(--bg-selected));border-radius:var(--radius-pill);background:var(--bg-selected);transition:background var(--dur-fast) ease-out,border-color var(--dur-fast) ease-out}.switch-track:after{content:"";position:absolute;left:1px;top:1px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.22);transition:transform var(--dur-fast) ease-out}.switch input:checked+.switch-track{border-color:var(--action);background:var(--action)}.switch input:checked+.switch-track:after{transform:translateX(16px);background:var(--action-on)}.switch input:focus-visible+.switch-track{outline:2px solid var(--text-2);outline-offset:3px}.switch:hover .switch-track{border-color:var(--text-2)}.switch:hover input:checked+.switch-track{border-color:var(--action)}.retention-warning{margin:10px 0;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.5}@media(prefers-reduced-motion:reduce){.switch-track,.switch-track:after{transition:none}}@media(forced-colors:active){.switch-track{border:1px solid ButtonText;background:Canvas}.switch-track:after{background:ButtonText}.switch input:checked+.switch-track{background:Highlight}.switch input:checked+.switch-track:after{background:HighlightText}}
  .settings-content.updates-content{overflow:hidden;padding-bottom:0}
  .updates-layout{height:100%;min-height:0;display:flex;flex-direction:column}
  .update-check{flex:none;min-height:56px;margin-top:0;padding-block:12px;border-bottom:1px solid var(--hairline)}
  .update-check .update-status{min-width:0}
  .update-check .update-actions{flex:none}
  .update-check .update-actions>button{min-height:32px;padding:0 12px;white-space:nowrap}
  .release-history{flex:1 1 auto;min-height:0;margin-top:16px;overflow:hidden;display:grid;grid-template-rows:minmax(0,1fr)}
  .release-browser{height:100%;min-height:0;overflow:hidden;display:grid;grid-template-columns:180px minmax(0,1fr)}
  .release-list{min-height:0;overflow-y:auto;padding:4px 8px 0 0;border-right:1px solid var(--hairline)}
  .release-option{display:flex;min-height:36px;padding:4px 8px;align-items:center;justify-content:space-between;gap:8px;border-radius:var(--radius-lg);cursor:pointer}
  .release-option strong,.release-option small{white-space:nowrap}
  .release-option small{font-variant-numeric:tabular-nums}
  .release-option.active{color:var(--text-1);background:var(--bg-selected)}
  .release-list:focus-visible{outline:none}
  .release-list:focus-visible .release-option.active{outline:2px solid var(--text-1);outline-offset:-2px}
  .release-loading .release-list,.release-loading .release-detail{gap:10px}
  .release-loading .release-detail{display:flex;flex-direction:column;padding-right:14px}
  .release-skeleton{display:block;border-radius:var(--radius-sm);background:var(--bg-raised)}
  .release-skeleton-row{height:36px;margin-bottom:4px;border-radius:var(--radius-lg)}
  .release-skeleton-title{width:30%;height:18px}
  .release-skeleton-line{width:88%;height:12px}.release-skeleton-line.short{width:54%}
  .release-load-failed{place-items:center}
  .release-load-message{display:flex;align-items:center;gap:12px}.release-load-message .inline-error{margin:0}.release-load-message button{min-height:32px;padding:0 12px;white-space:nowrap}
  .release-detail{min-width:0;min-height:0;display:grid;grid-template-rows:auto minmax(0,1fr);padding:10px 0 0 14px}
  .release-detail header{display:flex;align-items:center;justify-content:space-between;gap:12px}
  .release-detail header>span{display:flex;min-width:0;flex-direction:column;gap:2px}.release-detail-title{display:flex;align-items:center;gap:6px}.release-detail-title em{padding:1px 4px;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-hover);font-size:var(--fs-meta);font-style:normal;font-weight:400;line-height:1.3;white-space:nowrap}
  .release-body{min-height:0;margin-top:8px;padding:10px 12px;overflow-y:auto;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-shell);font-size:var(--fs-ui);line-height:1.55}.release-body.raw-release-body{white-space:pre-wrap}.release-body :global(h2),.release-body :global(h3){margin:0 0 8px;color:var(--text-1);font-size:var(--fs-body);line-height:1.35}.release-body :global(h2:not(:first-child)),.release-body :global(h3:not(:first-child)){margin-top:18px}.release-body :global(p),.release-body :global(ul),.release-body :global(blockquote){margin:0 0 12px}.release-body :global(ul){padding-left:20px}.release-body :global(li+li){margin-top:4px}.release-body :global(blockquote){padding:8px 10px;border-left:2px solid var(--hairline);border-radius:0 var(--radius-sm) var(--radius-sm) 0;background:var(--bg-raised)}.release-body :global(a){color:var(--action);text-decoration:underline;text-underline-offset:2px}
</style>
