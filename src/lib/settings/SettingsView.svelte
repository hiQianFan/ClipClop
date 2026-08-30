<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { AlertDialog, Progress, Tabs } from "bits-ui";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/history/api";
  import { openAutoPasteSettings } from "$lib/onboarding/api";
  import { applyTheme, getSettings, openFilePreviewSettings, openLogDir, previewTheme, updateSettings, type LanguagePreference, type Settings, type Theme, type TrayClickAction } from "./api";
  import AppSelect from "$lib/components/AppSelect.svelte";
  import { currentPlatform, defaultShortcut, shortcutFromKeyboardEvent, shortcutKeycaps, shortcutSpokenLabel, type ShortcutPlatform } from "./shortcuts";
  import { DEVELOPMENT_VERSION, listReleaseNotes, openLatestRelease, type ReleaseNote } from "$lib/updater/api";
  import { updateStore } from "$lib/updater/store.svelte";
  import { effectiveLocale, formatDateTime, formatNumber, localizedError, setLanguagePreference, t, type StaticMessageKey } from "$lib/i18n/index.svelte";
  import { RefreshCw } from "@lucide/svelte";

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
  const progressLabel = $derived(updateProgress === null ? t("settings.downloading") : `${formatNumber(updateProgress)}%`);
  const lastChecked = $derived(settings?.last_update_check ? t("settings.lastChecked", { time: formatDateTime(settings.last_update_check) }) : t("settings.notChecked"));
  // Derived so the message re-localizes on language change instead of freezing
  // at the locale that was active when the task ran.
  const updateMessage = $derived.by(() => {
    effectiveLocale();
    switch (updateState) {
      case "checking":
        return updateStore.displayStatus === "current"
          ? t("settings.updateCurrent")
          : updateStore.displayStatus === "available" && update
            ? t("settings.newVersion", { version: update.version })
            : updateStore.displayStatus === "skipped"
              ? t("settings.skippedVersion", { version: updateStore.skippedVersion ?? "" })
              : t("settings.checkingLong");
      case "current": return t("settings.updateCurrent");
      case "skipped": return t("settings.skippedVersion", { version: updateStore.skippedVersion ?? "" });
      case "downloading": return t("settings.downloadingVersion", { version: update?.version ?? "" });
      case "downloaded": return t("settings.downloadedVersion", { version: update?.version ?? "" });
      case "installing": return t("settings.installing");
      case "error":
        return updateStore.errorSource === "unsupported"
          ? t("settings.devUpdate")
          : updateStore.errorSource === "install"
            ? t("settings.installFailedTitle")
            : updateStore.errorSource === "relaunch"
              ? t("settings.restartFailedTitle")
            : updateStore.errorSource === "download"
              ? t("settings.downloadFailedTitle")
            : t("settings.checkFailedTitle");
      default: return update ? t("settings.newVersion", { version: update.version }) : t("settings.notChecked");
    }
  });
  const updateDetail = $derived.by(() => {
    if (updateState === "checking" && updateStore.displayStatus) return t("settings.checkingLong");
    if (updateState === "current") return lastChecked;
    if (updateState !== "error") return "";
    if (updateStore.errorSource === "download" || updateStore.errorSource === "check") return t("settings.checkConnection");
    if (updateStore.errorSource === "install") return t("settings.packageRetained");
    if (updateStore.errorSource === "relaunch") return t("settings.updateInstalled");
    return "";
  });
  let confirmClear = $state(false);
  let navFocusRing = $state(false);
  let recording = $state(false);
  let shortcutError = $state("");
  let savedSettings = $state<Settings | null>(null);
  let destroyed = false;
  let navButtons = $state<Array<HTMLButtonElement | null>>(Array(tabs.length).fill(null));
  let settingsContent = $state<HTMLElement | null>(null);
  let sectionHeading = $state<HTMLHeadingElement>();
  let clearTrigger = $state<HTMLButtonElement>();
  let confirmClearButton = $state<HTMLButtonElement | null>(null);
  let recorder = $state<HTMLButtonElement>();
  let releases = $state<ReleaseNote[]>([]);
  let selectedRelease = $state<ReleaseNote | null>(null);
  let releasesLoading = $state(false);
  let releasesError = $state("");
  let releaseList = $state<HTMLDivElement>();
  const platform: ShortcutPlatform = currentPlatform();
  const trayItems = $derived([
    { value: "recent", label: t("settings.trayClickRecent") },
    { value: "history", label: t("settings.trayClickHistory") },
  ]);
  const retentionItems = $derived([
    ...[1, 7, 30, 90].map((count) => ({ value: String(count), label: t("settings.days", { count: formatNumber(count) }) })),
    { value: "365", label: t("settings.year") },
    { value: "none", label: t("settings.forever") },
  ]);
  const historyLimitItems = $derived([
    ...[100, 500, 1000, 5000].map((count) => ({ value: String(count), label: t("settings.items", { count: formatNumber(count) }) })),
    { value: "none", label: t("settings.unlimited") },
  ]);
  const themeItems = $derived([
    { value: "system", label: t("settings.followSystem") },
    { value: "light", label: t("settings.light") },
    { value: "dark", label: t("settings.dark") },
  ]);
  const languageItems = $derived([
    { value: "system", label: t("settings.languageSystem") },
    { value: "zh-CN", label: t("settings.languageChinese") },
    { value: "en", label: t("settings.languageEnglish") },
  ]);

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
    if (!saving && savedSettings) {
      previewTheme(savedSettings.theme);
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
    if (event.key === "ArrowRight" || (event.key === "Tab" && !event.shiftKey)) {
      event.preventDefault();
      await focusDetail();
    }
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
        previewTheme(savedSettings.theme);
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

  function requestClear() {
    confirmClear = true;
  }

  function cancelClear() {
    confirmClear = false;
  }

  // The store owns the async lifecycle; these just trigger it. State updates flow
  // back through the derived reads above, so closing/reopening the view mid-task
  // never loses progress.
  async function checkUpdates() {
    await updateStore.check();
    if (!destroyed && settings) {
      try { settings.last_update_check = (await getSettings()).last_update_check; }
      catch { /* The update result remains useful when refreshing this timestamp fails. */ }
    }
  }

  function installUpdate() { void updateStore.install(); }
  function downloadUpdate(autoInstall = false) { void updateStore.download(autoInstall); }
  function cancelDownload() { void updateStore.cancel(); }
  function retryUpdate() { void updateStore.retry(); }

  function skipUpdate() { void updateStore.skip(); }

  async function removeAll() {
    try {
      await clearHistory(); confirmClear = false; status = t("settings.cleared"); oncleared();
    } catch (reason) { confirmClear = false; status = t("settings.clearFailed", { error: localizedError(reason) }); }
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
  function changeTheme(value: string) {
    if (!settings) return;
    settings.theme = value as Theme;
    previewTheme(settings.theme);
  }
  function changeLanguage(value: string) {
    if (!settings) return;
    settings.language = value as LanguagePreference;
    previewLanguage();
  }
  function displayVersion(version: string) {
    if (version === DEVELOPMENT_VERSION) return t("update.devVersion");
    return version === "__clipclop_unknown__" ? t("common.unknown") : version;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Tab") navFocusRing = true;
    else if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) navFocusRing = false;
    if (event.defaultPrevented) return;
    if (event.key === "Escape") {
      event.preventDefault();
      if (recording) { recording = false; shortcutError = ""; status = t("settings.recordCancelled"); }
      else if (confirmClear) cancelClear();
      else onclose();
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault(); void save();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} onpointerdown={() => navFocusRing = false} />

<div class="settings-shell">
  <Tabs.Root class="settings-body" value={tab} onValueChange={(value) => selectTab(value as Tab)} orientation="vertical" activationMode="automatic" loop={true}>
    <Tabs.List class={`settings-nav${navFocusRing ? " tab-focus" : ""}`} aria-label={t("settings.categories")}>
      <Tabs.Trigger bind:ref={navButtons[0]} value="general" class={tab === "general" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.general")}</Tabs.Trigger>
      <Tabs.Trigger bind:ref={navButtons[1]} value="history" class={tab === "history" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.history")}</Tabs.Trigger>
      <Tabs.Trigger bind:ref={navButtons[2]} value="appearance" class={tab === "appearance" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.appearance")}</Tabs.Trigger>
      <Tabs.Trigger bind:ref={navButtons[3]} value="shortcuts" class={tab === "shortcuts" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.shortcuts")}</Tabs.Trigger>
      <span class="nav-separator" aria-hidden="true"></span>
      <Tabs.Trigger bind:ref={navButtons[4]} value="updates" class={tab === "updates" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.updates")}</Tabs.Trigger>
      <Tabs.Trigger bind:ref={navButtons[5]} value="about" class={tab === "about" ? "active" : ""} onkeydown={onNavKeydown}>{t("settings.about")}</Tabs.Trigger>
    </Tabs.List>
    {#each tabs as panelTab}
    {#if panelTab === tab}
    <Tabs.Content bind:ref={settingsContent} value={panelTab} class={`settings-content${tab === "updates" ? " updates-content" : ""}`} tabindex={-1} onkeydown={onContentKeydown}>
      {#if settings}
        {#if tab === "general"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.general")}</h1>
          <div class="row setting-row"><span><strong id="launch-label">{t("settings.launch")}</strong><small id="launch-help">{t("settings.launchHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="launch-label" aria-describedby="launch-help" bind:checked={settings.launch_at_login} /><span class="switch-track"></span></label></div>
          {#if platform === "macos"}<div class="row"><span><strong>{t("settings.trayClick")}</strong><small>{t("settings.trayClickHelp")}</small></span><AppSelect value={settings.tray_click_action} items={trayItems} ariaLabel={t("settings.trayClick")} onchange={(value) => settings!.tray_click_action = value as TrayClickAction} /></div>{/if}
          <div class="row"><span><strong>{t("settings.quickStart")}</strong><small>{t("settings.quickStartHelp")}</small></span><button onclick={onquickstart}>{t("settings.quickStart")}</button></div>
          {#if platform === "macos"}<div class="row"><span><strong>{t("settings.autoPaste")}</strong><small>{t("settings.autoPasteHelp")}</small></span><button onclick={() => void openAutoPasteSystemSettings()}>{t("settings.manage")}</button></div>{/if}
          {#if platform === "macos"}<div class="row"><span><strong>{t("settings.filePreview")}</strong><small>{t("settings.filePreviewHelp")}</small></span><button onclick={() => void openFilePreviewSystemSettings()}>{t("settings.manage")}</button></div>{/if}
        {:else if tab === "history"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.history")}</h1>
          <div class="row"><span><strong>{t("settings.retention")}</strong><small>{t("settings.retentionHelp")}</small></span><AppSelect value={settings.retention_days === null ? "none" : String(settings.retention_days)} items={retentionItems} ariaLabel={t("settings.retention")} onchange={(value) => settings!.retention_days = value === "none" ? null : Number(value) as Settings["retention_days"]} /></div>
          <div class="row"><span><strong>{t("settings.historyLimit")}</strong><small>{t("settings.historyLimitHelp")}</small></span><AppSelect value={settings.history_limit === null ? "none" : String(settings.history_limit)} items={historyLimitItems} ariaLabel={t("settings.historyLimit")} onchange={(value) => settings!.history_limit = value === "none" ? null : Number(value) as Settings["history_limit"]} /></div>
          <div class="row setting-row"><span><strong id="move-used-label">{t("settings.moveUsedToTop")}</strong><small id="move-used-help">{t("settings.moveUsedToTopHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="move-used-label" aria-describedby="move-used-help" bind:checked={settings.move_used_to_top} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="restore-pos-label">{t("settings.restoreBrowsePosition")}</strong><small id="restore-pos-help">{t("settings.restoreBrowsePositionHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="restore-pos-label" aria-describedby="restore-pos-help" bind:checked={settings.restore_browse_position} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="preserve-search-label">{t("settings.preserveSearchConditions")}</strong><small id="preserve-search-help">{t("settings.preserveSearchConditionsHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="preserve-search-label" aria-describedby="preserve-search-help" bind:checked={settings.preserve_search_conditions} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="trim-whitespace-label">{t("settings.trimWhitespace")}</strong><small id="trim-whitespace-help">{t("settings.trimWhitespaceHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="trim-whitespace-label" aria-describedby="trim-whitespace-help" bind:checked={settings.trim_whitespace} /><span class="switch-track"></span></label></div>
          {#if settings.retention_days === null || settings.history_limit === null}<p class="retention-warning">{t("settings.retentionWarning")}</p>{/if}
          <div class="row"><span><strong>{t("settings.data")}</strong><small>{t("settings.dataHelp")}</small></span><button bind:this={clearTrigger} class="danger" onclick={requestClear}>{t("settings.clearAll")}</button></div>
        {:else if tab === "appearance"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.appearance")}</h1>
          <div class="row"><span><strong>{t("settings.theme")}</strong><small>{t("settings.appearanceHelp")}</small></span><AppSelect value={settings.theme} items={themeItems} ariaLabel={t("settings.theme")} onchange={changeTheme} /></div>
          <div class="row"><span><strong>{t("settings.language")}</strong><small>{t("settings.languageHelp")}</small></span><AppSelect value={settings.language} items={languageItems} ariaLabel={t("settings.language")} onchange={changeLanguage} /></div>
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
          <section class="update-section" aria-label={t("settings.updateSettingsAndStatus")}>
            <div class="update-head"><span><strong>{t("settings.autoCheckUpdates")}</strong><small>{t("settings.currentVersion", { version: displayVersion(appVersion) })}</small></span><div class="update-head-controls"><button class="update-refresh" disabled={updateState === "checking" || updateBusy} aria-label={t("settings.check")} title={t("settings.check")} aria-busy={updateState === "checking"} onclick={checkUpdates}><RefreshCw size={15} class={updateState === "checking" ? "spin" : ""} /></button><label class="switch compact-switch"><input type="checkbox" role="switch" aria-label={t("settings.autoCheckUpdates")} bind:checked={settings.check_updates} /><span class="switch-track"></span></label></div></div>
            <div class="update-rail">
              <span class="visually-hidden" role={updateState === "error" ? "alert" : "status"} aria-live="polite">{updateMessage}{updateDetail ? ` ${updateDetail}` : ""}</span>
              <div class="update-rail-main">
                <strong class:error={updateState === "error" && updateStore.errorSource !== "unsupported"}>{updateMessage}</strong>
                {#if updateState === "downloading"}
                  <div class="update-progress">
                    <Progress.Root class="progress-track" value={updateProgress} max={100} aria-label={progressLabel} aria-valuetext={progressLabel}><div class="progress-fill" class:indeterminate={updateProgress === null} style={updateProgress === null ? "" : `width:${updateProgress}%`}></div></Progress.Root>
                    {#if updateProgress !== null}<span class="progress-label">{formatNumber(updateProgress)}%</span>{/if}
                  </div>
                {:else if updateDetail}<span class="update-detail">{updateDetail}</span>{/if}
              </div>
              <div class="update-actions">
                {#if updateState === "downloading"}
                  <button class="update-primary" onclick={cancelDownload}>{t("common.cancel")}</button>
                {:else if updateState === "downloaded"}
                  <button class="secondary" onclick={skipUpdate}>{t("settings.skipVersion")}</button><button class="update-primary" onclick={installUpdate}>{t("settings.installRestart")}</button>
                {:else if updateState === "error" && updateStore.errorSource === "relaunch"}
                  <button class="update-primary" onclick={retryUpdate}>{t("settings.restart")}</button>
                {:else if updateState === "error" && (updateStore.errorSource === "download" || updateStore.errorSource === "install")}
                  <button class="secondary" onclick={skipUpdate}>{t("settings.skipVersion")}</button><button class="update-primary" onclick={retryUpdate}>{t("settings.retry")}</button>
                {:else if updateState === "error" && updateStore.errorSource === "check"}
                  <button class="update-primary" onclick={retryUpdate}>{t("settings.retry")}</button>
                {:else if update && updateState !== "installing" && updateState !== "checking"}
                  <button class="secondary" onclick={skipUpdate}>{t("settings.skipVersion")}</button><button class="update-primary" onclick={() => downloadUpdate()}>{t("settings.download")}</button>
                {/if}
              </div>
            </div>
          </section>
          <section class="release-history" aria-label={t("settings.releaseNotes")}>
            {#if releasesLoading}
              <div class="release-browser release-loading" aria-busy="true" aria-label={t("settings.loading")}>
                <div class="release-list" aria-hidden="true">
                  {#each Array(7) as _, index}<span class:active={index === 0} class="release-skeleton-row"><i></i><i></i></span>{/each}
                </div>
                <div class="release-detail" aria-hidden="true">
                  <header><span><i class="release-skeleton title"></i><i class="release-skeleton date"></i></span></header>
                  <div class="release-skeleton-body"><i class="release-skeleton heading"></i><i class="release-skeleton line"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line"></i><i class="release-skeleton heading second"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line short"></i></div>
                </div>
              </div>
            {:else if releasesError}
              <div class="release-browser release-load-failed"><div class="release-load-message"><p class="inline-error" role="alert">{releasesError}</p><button onclick={() => void loadReleases()}>{t("settings.refreshReleaseNotes")}</button></div></div>
            {:else if releases.length}
              <div class="release-browser">
                <div bind:this={releaseList} class="release-list" role="listbox" aria-label={t("settings.releaseNotes")} aria-activedescendant={selectedRelease ? `release-option-${selectedRelease.version}` : undefined} tabindex="0" onclick={onReleaseListClick} onkeydown={onReleaseListKeydown}>
                  {#each releases as release, index}<div id={`release-option-${release.version}`} class="release-option" class:active={selectedRelease?.version === release.version} data-release-index={index} role="option" aria-selected={selectedRelease?.version === release.version}><strong>{release.version}</strong><small>{formatDateTime(release.publishedAt)}</small></div>{/each}
                </div>
                {#if selectedRelease}<article class="release-detail"><header><span><span class="release-detail-title"><strong>{selectedRelease.version}</strong>{#if selectedRelease.isLatest}<em>{t("settings.latestRelease")}</em>{/if}</span><small>{formatDateTime(selectedRelease.publishedAt)}</small></span>{#if selectedRelease.isLatest}<button class="release-page" onclick={() => void openReleasePage()}>{t("settings.releasePage")} ↗</button>{/if}</header><div class="release-body" class:raw-release-body={!selectedRelease.notesHtml}>{#if selectedRelease.notesHtml}{@html selectedRelease.notesHtml}{:else}{selectedRelease.notes}{/if}</div></article>{/if}
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
    </Tabs.Content>
    {:else}
      <Tabs.Content value={panelTab} class="settings-content" tabindex={-1} />
    {/if}
    {/each}
  </Tabs.Root>
  <AlertDialog.Root open={confirmClear} onOpenChange={(open) => confirmClear = open}>
  <footer>
    {#if confirmClear}<AlertDialog.Content class="clear-confirmation" aria-label={t("settings.clearConfirm")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmClearButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); clearTrigger?.focus(); }}><strong>{t("settings.clearConfirm")}</strong><AlertDialog.Cancel onclick={cancelClear}>{t("common.cancel")}</AlertDialog.Cancel><AlertDialog.Action bind:ref={confirmClearButton} class="danger" onclick={() => void removeAll()}>{t("settings.clear")}</AlertDialog.Action></AlertDialog.Content>
    {:else}<span aria-live="polite" aria-atomic="true">{status}</span><button onclick={onclose}>{t("common.back")}</button>{#if tab !== "about"}<button class="primary" onclick={() => void save()} disabled={!settings || saving} aria-busy={saving}>{t("common.save")}</button>{/if}{/if}
  </footer>
  </AlertDialog.Root>
</div>

<!-- svelte-ignore css_unused_selector -->
<style>
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}button{padding:8px 10px;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);line-height:1.4}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.section-intro{margin:0 0 8px;color:var(--text-2);font-size:var(--fs-ui);line-height:1.5}.shortcut-help{max-width:72ch;margin:0 0 18px;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.55}.shortcut-help strong{color:var(--text-1)}.row,.update-head{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}label>span,.row>span,.update-head>span,.shortcut-row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}strong{font-size:var(--fs-body)}small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}select{min-width:116px;padding:7px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-1);background:var(--bg-raised);font-size:var(--fs-ui)}input{width:18px;height:18px}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:var(--fs-ui);color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-actions,.key-list{display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.key-combination{display:flex;align-items:center;gap:4px;border:0;background:transparent}.shortcut-actions .key-combination{min-width:92px;justify-content:center}.keycap{padding:3px 6px;border:1px solid var(--hairline);border-radius:var(--radius-sm);color:var(--text-1);background:var(--bg-raised);font:var(--fs-ui)/var(--lh-snug) ui-monospace,monospace;white-space:nowrap}.key-plus,.alternative{color:var(--text-3);font-size:var(--fs-meta);line-height:1.3}.alternative{margin:0 2px}.recording{color:var(--text-1);background:var(--bg-selected)}.inline-error{margin:8px 0 0;color:var(--danger);font-size:var(--fs-ui)}.update-head label{display:flex;align-items:center;gap:8px}.update-head-controls{display:flex;align-items:center;gap:16px}.update-head-controls .update-check-btn{flex:none;border-radius:var(--radius-md)}.update-card{display:flex;flex-direction:column;gap:12px;margin-top:16px;padding:16px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised)}.update-card-head{display:flex;align-items:center;gap:12px}.update-badge{flex:none;display:grid;place-items:center;width:34px;height:34px;border-radius:var(--radius-lg);color:var(--action-on);background:var(--action)}.update-card-title{display:flex;flex-direction:column;gap:2px}.update-card-title strong{font-size:var(--fs-emphasis)}.update-progress{display:flex;flex-direction:column;gap:7px}.progress-fill{height:100%;border-radius:var(--radius-pill);background:var(--action);transition:width var(--dur-slow) ease}.progress-fill.indeterminate{width:35%;animation:progress-slide 1.1s ease-in-out infinite}.progress-label{color:var(--text-2);font-size:var(--fs-ui);font-variant-numeric:tabular-nums}.update-status{display:flex;align-items:center;gap:6px;font-size:var(--fs-ui);line-height:1.4}.update-status :global(svg){flex:none}.update-card .update-status{margin:0}.update-actions,.update-check{display:flex;align-items:center;gap:8px}.update-actions{justify-content:flex-end}.update-check{justify-content:space-between;margin-top:16px}.update-check-btn{display:inline-flex;align-items:center;gap:6px}.update-check-btn :global(svg.spin){animation:spin 1s linear infinite}@keyframes progress-slide{0%{transform:translateX(-120%)}100%{transform:translateX(340%)}}@keyframes spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.progress-fill.indeterminate,.update-check-btn :global(svg.spin){animation:none}}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about{position:relative}.about img{width:56px;height:56px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:var(--fs-ui)}.log-door{position:absolute;bottom:8px;left:50%;transform:translateX(-50%);min-height:0;padding:4px 8px;color:var(--text-3);font-size:var(--fs-meta);font-weight:400;opacity:.7}.log-door:hover{color:var(--text-2);background:transparent;opacity:1}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}footer button{min-width:92px;min-height:32px;padding:0 12px}.primary{color:var(--action-on);background:var(--action)}.primary:hover:not(:disabled){color:var(--action-on);background:var(--action-hover)}.danger,.error{color:var(--danger)}footer .danger{color:var(--danger-on);background:var(--danger-fill);font-weight:600}.danger:hover:not(:disabled){color:var(--danger-on);background:var(--danger-fill)}button:disabled{opacity:.45;cursor:not-allowed}button:disabled:hover{background:transparent}.primary:disabled:hover{background:var(--action)}.visually-hidden{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  .nav-separator{height:1px;margin:8px 6px;background:var(--hairline)}
  /* Setting-row contract: text zone flexes (rule above), action zone is protected
     and never compresses. Every row's action lives in one of these. */
  .row>button,.update-head>.update-head-controls,.shortcut-row>.shortcut-actions,.shortcut-row>.key-list{flex:none}
  /* Unified action-button sizing across every section (ghost per DESIGN.md). */
  .row>button,.update-check-btn{min-height:32px;padding:0 12px;white-space:nowrap}
  .switch{position:relative;flex:none;width:44px;height:44px;cursor:pointer}.switch input{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);clip-path:inset(50%);white-space:nowrap}.switch-track{position:absolute;left:4px;top:12px;width:36px;height:20px;border:1px solid color-mix(in srgb,var(--text-2) 42%,var(--bg-selected));border-radius:var(--radius-pill);background:var(--bg-selected);transition:background var(--dur-fast) ease-out,border-color var(--dur-fast) ease-out}.switch-track:after{content:"";position:absolute;left:1px;top:1px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.22);transition:transform var(--dur-fast) ease-out}.switch input:checked+.switch-track{border-color:var(--action);background:var(--action)}.switch input:checked+.switch-track:after{transform:translateX(16px);background:var(--action-on)}.switch input:focus-visible+.switch-track{outline:2px solid var(--text-2);outline-offset:3px}.switch:hover .switch-track{border-color:var(--text-2)}.switch:hover input:checked+.switch-track{border-color:var(--action)}.retention-warning{margin:10px 0;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.5}@media(prefers-reduced-motion:reduce){.switch-track,.switch-track:after{transition:none}}@media(forced-colors:active){.switch-track{border:1px solid ButtonText;background:Canvas}.switch-track:after{background:ButtonText}.switch input:checked+.switch-track{background:Highlight}.switch input:checked+.switch-track:after{background:HighlightText}}
  .updates-layout{height:100%;min-height:0;display:flex;flex-direction:column}
  .update-section{flex:none}
  .update-head-controls{gap:8px}
  .update-refresh{position:relative;width:32px;height:32px;padding:0;display:grid;place-items:center}
  .update-refresh :global(svg.spin){animation:spin 1s linear infinite}
  .update-rail{height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}
  .update-rail-main{min-width:0;flex:1;display:flex;align-items:center;gap:12px}
  .update-rail-main>strong{min-width:0;max-width:210px;flex:0 1 auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .update-detail{min-width:0;overflow:hidden;color:var(--text-2);font-size:var(--fs-ui);text-overflow:ellipsis;white-space:nowrap}
  .update-progress{min-width:80px;flex:1;display:flex;flex-direction:row;align-items:center;gap:8px}
  .update-progress :global(.progress-track){min-width:80px;flex:1;height:4px}
  .progress-label{width:38px;flex:none;text-align:right}
  .update-rail .update-actions{flex:none;display:flex;align-items:center;gap:8px}
  .update-rail .update-actions>button{min-height:32px;padding:0 12px;white-space:nowrap}
  .update-rail .update-actions>.secondary{color:var(--text-3)}
  .update-rail .update-actions>.secondary:hover{color:var(--text-1)}
  .update-rail .update-actions>.update-primary{min-width:88px;color:var(--action-on);background:var(--action);font-weight:600}
  .update-rail .update-actions>.update-primary:hover{color:var(--action-on);background:var(--action-hover)}
  @media(prefers-reduced-motion:reduce){.update-refresh :global(svg.spin){animation:none}}
  .release-history{flex:1 1 auto;min-height:0;margin-top:16px;overflow:hidden;display:grid;grid-template-rows:minmax(0,1fr)}
  .release-browser{height:100%;min-height:0;overflow:hidden;display:grid;grid-template-columns:180px minmax(0,1fr)}
  .release-list{min-height:0;overflow-y:auto;padding:4px 8px 0 0;border-right:1px solid var(--hairline)}
  .release-list,.release-body{scrollbar-width:thin;scrollbar-color:color-mix(in srgb,var(--text-3) 52%,transparent) transparent}
  .release-list::-webkit-scrollbar,.release-body::-webkit-scrollbar{width:9px}
  .release-list::-webkit-scrollbar-track,.release-body::-webkit-scrollbar-track{background:transparent}
  .release-list::-webkit-scrollbar-thumb,.release-body::-webkit-scrollbar-thumb{border:3px solid transparent;border-radius:var(--radius-pill);background:color-mix(in srgb,var(--text-3) 52%,transparent);background-clip:padding-box}
  .release-list::-webkit-scrollbar-thumb:hover,.release-body::-webkit-scrollbar-thumb:hover{background-color:var(--text-3)}
  .release-option{display:flex;min-height:36px;padding:4px 8px;align-items:center;justify-content:space-between;gap:8px;border-radius:var(--radius-lg);cursor:pointer}
  .release-option strong,.release-option small{white-space:nowrap}
  .release-option small{font-variant-numeric:tabular-nums}
  .release-option.active{color:var(--text-1);background:var(--bg-selected)}
  .release-list:focus-visible{outline:none}
  .release-list:focus-visible .release-option.active{outline:2px solid var(--text-1);outline-offset:-2px}
  .release-loading .release-detail{padding-right:14px}
  .release-skeleton,.release-skeleton-row i{display:block;border-radius:var(--radius-sm);background:color-mix(in srgb,var(--text-3) 14%,transparent)}
  .release-skeleton-row{min-height:36px;margin-bottom:4px;padding:7px 8px;display:flex;align-items:center;justify-content:space-between;gap:12px;border-radius:var(--radius-lg)}
  .release-skeleton-row.active{background:color-mix(in srgb,var(--bg-selected) 55%,transparent)}
  .release-skeleton-row i:first-child{width:52px;height:14px}.release-skeleton-row i:last-child{width:70px;height:12px}
  .release-loading .release-detail header{min-height:54px;padding-top:10px}.release-loading .release-detail header>span{gap:6px}
  .release-skeleton.title{width:72px;height:16px}.release-skeleton.date{width:92px;height:11px}
  .release-skeleton-body{display:flex;flex-direction:column;gap:10px;padding:12px;border-radius:var(--radius-md);background:color-mix(in srgb,var(--bg-raised) 55%,transparent)}
  .release-skeleton.heading{width:64px;height:15px;margin-bottom:3px}.release-skeleton.heading.second{margin-top:12px}
  .release-skeleton.line{width:76%;height:10px}.release-skeleton.line.wide{width:91%}.release-skeleton.line.short{width:58%}
  .release-load-failed{place-items:center}
  .release-load-message{display:flex;align-items:center;gap:12px}.release-load-message .inline-error{margin:0}.release-load-message button{min-height:32px;padding:0 12px;white-space:nowrap}
  .release-detail{min-width:0;min-height:0;display:grid;grid-template-rows:auto minmax(0,1fr);padding:10px 0 0 14px}
  .release-detail header{display:flex;align-items:center;justify-content:space-between;gap:12px}
  .release-page{padding:4px 6px;color:var(--text-3);white-space:nowrap}
  .release-detail header>span{display:flex;min-width:0;flex-direction:column;gap:2px}.release-detail-title{display:flex;align-items:center;gap:6px}.release-detail-title em{padding:1px 4px;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-hover);font-size:var(--fs-meta);font-style:normal;font-weight:400;line-height:1.3;white-space:nowrap}
  .release-body{min-height:0;margin-top:8px;padding:10px 12px;overflow-y:auto;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-shell);font-size:var(--fs-ui);line-height:1.55}.release-body.raw-release-body{white-space:pre-wrap}.release-body :global(h2),.release-body :global(h3){margin:0 0 8px;color:var(--text-1);font-size:var(--fs-body);line-height:1.35}.release-body :global(h2:not(:first-child)),.release-body :global(h3:not(:first-child)){margin-top:18px}.release-body :global(p),.release-body :global(ul),.release-body :global(blockquote){margin:0 0 12px}.release-body :global(ul){padding-left:20px}.release-body :global(li+li){margin-top:4px}.release-body :global(blockquote){padding:8px 10px;border-left:2px solid var(--hairline);border-radius:0 var(--radius-sm) var(--radius-sm) 0;background:var(--bg-raised)}.release-body :global(a){color:var(--action);text-decoration:underline;text-underline-offset:2px}
  :global(.settings-body){min-height:0;display:grid;grid-template-columns:clamp(168px,22%,192px) minmax(0,1fr)}
  :global(.settings-nav){display:flex;flex-direction:column;gap:3px;padding:14px 12px;border-right:1px solid var(--hairline)}
  :global(.settings-nav button){min-height:40px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;text-align:left;font-size:var(--fs-body);font-weight:600;line-height:1.4}
  :global(.settings-nav button:hover),:global(.settings-nav button.active){color:var(--text-1);background:var(--bg-hover)}
  :global(.settings-nav button.active){background:var(--bg-selected)}
  :global(.settings-nav button:focus-visible){outline:none}
  :global(.settings-nav.tab-focus button:focus-visible){box-shadow:inset 0 0 0 2px var(--text-1)}
  :global(.settings-content){min-width:0;min-height:0;overflow:auto;padding:0 24px 20px}
  :global(.settings-content h1){margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}
  :global(.settings-content h1:focus){outline:none}
  :global(.settings-content>label){min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}
  :global(.settings-content>label>select){flex:none}
  :global(.settings-content.updates-content){overflow:hidden;padding-bottom:0}
  :global(.progress-track){height:6px;overflow:hidden;border-radius:var(--radius-pill);background:var(--bg-selected)}
  :global(.clear-confirmation){width:100%;display:flex;align-items:center;justify-content:flex-end;gap:10px}
  :global(.clear-confirmation strong){min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  :global(.clear-confirmation button){min-width:92px;min-height:32px;padding:0 12px}
  :global(.clear-confirmation .danger){color:var(--danger-on);background:var(--danger-fill);font-weight:600}
</style>
