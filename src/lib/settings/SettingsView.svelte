<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { AlertDialog, Tabs } from "bits-ui";
  import { Check, LoaderCircle } from "@lucide/svelte";
  import Icon from "@iconify/svelte/dist/OfflineIcon.svelte";
  import githubIcon from "@iconify-icons/simple-icons/github";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/history/api";
  import { applyTheme, getSettings, openLogDir, performHaptic, previewTheme, updateSettings, type LanguagePreference, type Settings, type Theme } from "./api";
  import AppSelect from "$lib/components/AppSelect.svelte";
  import { currentPlatform, type ShortcutPlatform } from "./shortcuts";
  import { DEVELOPMENT_VERSION } from "$lib/updater/api";
  import { updateStore } from "$lib/updater/store.svelte";
  import { effectiveLocale, formatNumber, localizedError, setLanguagePreference, t } from "$lib/i18n/index.svelte";
  import GeneralSettings from "./GeneralSettings.svelte";
  import ShortcutSettings from "./ShortcutSettings.svelte";
  import UpdateSettings from "./UpdateSettings.svelte";

  type Tab = "general" | "history" | "appearance" | "shortcuts" | "updates" | "about";
  const tabs: Tab[] = ["general", "history", "appearance", "shortcuts", "updates", "about"];

  let { initialTab = "general", onclose, oncleared, onquickstart }: { initialTab?: Tab; onclose: () => void; oncleared: () => void; onquickstart: () => void } = $props();
  let settings = $state<Settings | null>(null);
  let tab = $state<Tab>("general");
  let status = $state("");
  let saving = $state(false);
  let saveSucceeded = $state(false);
  let saveFeedbackTimer: ReturnType<typeof setTimeout> | undefined;
  const appVersion = $derived(updateStore.appVersion);
  let confirmClear = $state(false);
  let navFocusRing = $state(false);
  let recording = $state(false);
  let savedSettings = $state<Settings | null>(null);
  let destroyed = false;
  let navButtons = $state<Array<HTMLButtonElement | null>>(Array(tabs.length).fill(null));
  let sectionHeading = $state<HTMLHeadingElement>();
  let clearTrigger = $state<HTMLButtonElement>();
  let confirmClearButton = $state<HTMLButtonElement | null>(null);
  let recorder = $state<HTMLButtonElement>();
  const platform: ShortcutPlatform = currentPlatform();
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


  onMount(() => {
    tab = initialTab;
    requestAnimationFrame(() => navButtons[tabs.indexOf(tab)]?.focus());
    void load();
  });
  onDestroy(() => {
    destroyed = true;
    clearTimeout(saveFeedbackTimer);
    if (!saving && savedSettings) {
      previewTheme(savedSettings.theme);
      setLanguagePreference(savedSettings.language);
    }
  });

  $effect(() => {
    effectiveLocale();
    untrack(() => {
      status = "";
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

  function selectTab(next: Tab) {
    tab = next;
    recording = false;
  }

  async function focusDetail() {
    await tick();
    const settingsContent = document.querySelector<HTMLElement>('.settings-content[data-state="active"]');
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
    clearTimeout(saveFeedbackTimer);
    saveSucceeded = false;
    saving = true;
    status = "";
    try {
      const saved = await updateSettings({ ...settings });
      settings = saved;
      savedSettings = { ...saved };
      setLanguagePreference(saved.language);
      applyTheme(saved.theme);
      recording = false;
      saveSucceeded = true;
      void performHaptic().catch(() => {});
      saveFeedbackTimer = setTimeout(() => saveSucceeded = false, 1600);
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

  async function removeAll() {
    try {
      await clearHistory(); confirmClear = false; status = t("settings.cleared"); oncleared();
    } catch (reason) { confirmClear = false; status = t("settings.clearFailed", { error: localizedError(reason) }); }
  }

  async function openLogs() {
    try { await openLogDir(); }
    catch (reason) { status = t("settings.openLogsFailed", { error: localizedError(reason) }); }
  }

  async function openGithub() {
    try { await openUrl("https://github.com/hiQianFan/ClipClop"); }
    catch (reason) { status = t("settings.openGithubFailed", { error: localizedError(reason) }); }
  }

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
      if (recording) { recording = false; status = t("settings.recordCancelled"); }
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
    <Tabs.Content value={panelTab} class={`settings-content${panelTab === "updates" ? " updates-content" : ""}`} tabindex={-1} onkeydown={onContentKeydown}>
      {#if settings}
        {#if panelTab === "general"}
          <GeneralSettings bind:settings {platform} {onquickstart} onerror={(message) => status = message} bind:heading={sectionHeading} />
        {:else if panelTab === "history"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.history")}</h1>
          <div class="row"><span><strong>{t("settings.retention")}</strong><small>{t("settings.retentionHelp")}</small></span><AppSelect value={settings.retention_days === null ? "none" : String(settings.retention_days)} items={retentionItems} ariaLabel={t("settings.retention")} onchange={(value) => settings!.retention_days = value === "none" ? null : Number(value) as Settings["retention_days"]} /></div>
          <div class="row"><span><strong>{t("settings.historyLimit")}</strong><small>{t("settings.historyLimitHelp")}</small></span><AppSelect value={settings.history_limit === null ? "none" : String(settings.history_limit)} items={historyLimitItems} ariaLabel={t("settings.historyLimit")} onchange={(value) => settings!.history_limit = value === "none" ? null : Number(value) as Settings["history_limit"]} /></div>
          <div class="row setting-row"><span><strong id="move-used-label">{t("settings.moveUsedToTop")}</strong><small id="move-used-help">{t("settings.moveUsedToTopHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="move-used-label" aria-describedby="move-used-help" bind:checked={settings.move_used_to_top} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="restore-pos-label">{t("settings.restoreBrowsePosition")}</strong><small id="restore-pos-help">{t("settings.restoreBrowsePositionHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="restore-pos-label" aria-describedby="restore-pos-help" bind:checked={settings.restore_browse_position} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="preserve-search-label">{t("settings.preserveSearchConditions")}</strong><small id="preserve-search-help">{t("settings.preserveSearchConditionsHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="preserve-search-label" aria-describedby="preserve-search-help" bind:checked={settings.preserve_search_conditions} /><span class="switch-track"></span></label></div>
          <div class="row setting-row"><span><strong id="trim-whitespace-label">{t("settings.trimWhitespace")}</strong><small id="trim-whitespace-help">{t("settings.trimWhitespaceHelp")}</small></span><label class="switch"><input type="checkbox" role="switch" aria-labelledby="trim-whitespace-label" aria-describedby="trim-whitespace-help" bind:checked={settings.trim_whitespace} /><span class="switch-track"></span></label></div>
          {#if settings.retention_days === null || settings.history_limit === null}<p class="retention-warning">{t("settings.retentionWarning")}</p>{/if}
          <div class="row"><span><strong>{t("settings.data")}</strong><small>{t("settings.dataHelp")}</small></span><button bind:this={clearTrigger} class="danger" onclick={requestClear}>{t("settings.clearAll")}</button></div>
        {:else if panelTab === "appearance"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">{t("settings.appearance")}</h1>
          <div class="row"><span><strong>{t("settings.theme")}</strong><small>{t("settings.appearanceHelp")}</small></span><AppSelect value={settings.theme} items={themeItems} ariaLabel={t("settings.theme")} onchange={changeTheme} /></div>
          <div class="row"><span><strong>{t("settings.language")}</strong><small>{t("settings.languageHelp")}</small></span><AppSelect value={settings.language} items={languageItems} ariaLabel={t("settings.language")} onchange={changeLanguage} /></div>
        {:else if panelTab === "shortcuts"}
          <ShortcutSettings bind:settings {platform} onstatus={(message) => status = message} bind:heading={sectionHeading} bind:recorder bind:recording />
        {:else if panelTab === "updates"}
          <UpdateSettings bind:settings onchecked={checkUpdates} onerror={(message) => status = message} bind:heading={sectionHeading} />
        {:else}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1" class="visually-hidden">{t("settings.about")}</h1>
          <div class="about"><img src="/app-icon-rounded.png" alt={t("settings.iconAlt")} /><h2>ClipClop</h2><p>{t("settings.tagline")}</p><small>{t("settings.version", { version: displayVersion(appVersion) })}</small><button class="github" aria-label={t("settings.github")} title={t("settings.github")} onclick={() => void openGithub()}><Icon icon={githubIcon} width={20} aria-hidden="true" /></button>
            <button class="log-door" title={t("settings.diagnosticsHelp")} onclick={() => void openLogs()}>{t("settings.diagnostics")}</button>
          </div>
        {/if}
      {:else}<div class="loading" role="status">{status || t("settings.loading")}</div>{/if}
    </Tabs.Content>
    {/each}
  </Tabs.Root>
  <AlertDialog.Root open={confirmClear} onOpenChange={(open) => confirmClear = open}>
  <footer>
    {#if confirmClear}<AlertDialog.Content class="clear-confirmation" aria-label={t("settings.clearConfirm")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmClearButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); clearTrigger?.focus(); }}><strong>{t("settings.clearConfirm")}</strong><AlertDialog.Cancel onclick={cancelClear}>{t("common.cancel")}</AlertDialog.Cancel><AlertDialog.Action bind:ref={confirmClearButton} class="danger" onclick={() => void removeAll()}>{t("settings.clear")}</AlertDialog.Action></AlertDialog.Content>
    {:else}<span aria-live="polite" aria-atomic="true">{status}</span><button onclick={onclose}>{t("common.back")}</button>{#if tab !== "about"}<button class="primary save-button" class:saved={saveSucceeded} onclick={() => void save()} disabled={!settings || saving} aria-busy={saving} aria-live="polite">{#if saving}<LoaderCircle size={14} class="save-spinner" />{t("settings.saving")}{:else if saveSucceeded}<Check size={14} />{t("settings.saved")}{:else}{t("common.save")}{/if}</button>{/if}{/if}
  </footer>
  </AlertDialog.Root>
</div>

<style>
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}button{padding:8px 10px;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui);line-height:1.4}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.row{min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}.row>span{flex:1 1 auto;min-width:0;display:flex;flex-direction:column;gap:3px}strong{font-size:var(--fs-body)}small{color:var(--text-3);font-size:var(--fs-ui);line-height:1.4}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about{position:relative}.about img{width:88px;height:88px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:var(--fs-ui)}.github{width:36px;height:36px;padding:0;display:grid;place-items:center}.log-door{position:absolute;bottom:8px;left:50%;transform:translateX(-50%);min-height:0;padding:4px 8px;color:var(--text-3);font-size:var(--fs-meta);font-weight:400;opacity:.7}.log-door:hover{color:var(--text-2);background:transparent;opacity:1}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}footer button{min-width:92px;min-height:32px;padding:0 12px}.primary{color:var(--action-on);background:var(--action)}.primary:hover:not(:disabled){color:var(--action-on);background:var(--action-hover)}.save-button{display:inline-flex;align-items:center;justify-content:center;gap:6px}.save-button.saved{color:var(--action);background:var(--bg-selected)}.save-button :global(.save-spinner){animation:save-spin .8s linear infinite}.danger{color:var(--danger)}.danger:hover:not(:disabled){color:var(--danger-on);background:var(--danger-fill)}button:disabled{opacity:.45;cursor:not-allowed}button:disabled:hover{background:transparent}.primary:disabled:hover{background:var(--action)}.visually-hidden{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}@keyframes save-spin{to{transform:rotate(360deg)}}
  .nav-separator{height:1px;margin:8px 6px;background:var(--hairline)}
  /* Setting-row contract: text zone flexes (rule above), action zone is protected
     and never compresses. Every row's action lives in one of these. */
  .row>button{flex:none}
  /* Unified action-button sizing across every section (ghost per DESIGN.md). */
  .row>button{min-height:32px;padding:0 12px;white-space:nowrap}
  .switch{position:relative;flex:none;width:44px;height:44px;cursor:pointer}.switch input{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);clip-path:inset(50%);white-space:nowrap}.switch-track{position:absolute;left:4px;top:12px;width:36px;height:20px;border:1px solid color-mix(in srgb,var(--text-2) 42%,var(--bg-selected));border-radius:var(--radius-pill);background:var(--bg-selected);transition:background var(--dur-fast) ease-out,border-color var(--dur-fast) ease-out}.switch-track:after{content:"";position:absolute;left:1px;top:1px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.22);transition:transform var(--dur-fast) ease-out}.switch input:checked+.switch-track{border-color:var(--action);background:var(--action)}.switch input:checked+.switch-track:after{transform:translateX(16px);background:var(--action-on)}.switch input:focus-visible+.switch-track{outline:2px solid var(--text-2);outline-offset:3px}.switch:hover .switch-track{border-color:var(--text-2)}.switch:hover input:checked+.switch-track{border-color:var(--action)}.retention-warning{margin:10px 0;padding:9px 11px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);line-height:1.5}@media(prefers-reduced-motion:reduce){.switch-track,.switch-track:after{transition:none}.save-button :global(.save-spinner){animation:none}}@media(forced-colors:active){.switch-track{border:1px solid ButtonText;background:Canvas}.switch-track:after{background:ButtonText}.switch input:checked+.switch-track{background:Highlight}.switch input:checked+.switch-track:after{background:HighlightText}}
  .settings-shell :global(.settings-body){min-height:0;display:grid;grid-template-columns:clamp(168px,22%,192px) minmax(0,1fr)}
  .settings-shell :global(.settings-nav){display:flex;flex-direction:column;gap:3px;padding:14px 12px;border-right:1px solid var(--hairline)}
  .settings-shell :global(.settings-nav button){min-height:40px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;text-align:left;font-size:var(--fs-body);font-weight:600;line-height:1.4}
  .settings-shell :global(.settings-nav button:hover),.settings-shell :global(.settings-nav button.active){color:var(--text-1);background:var(--bg-hover)}
  .settings-shell :global(.settings-nav button.active){background:var(--bg-selected)}
  .settings-shell :global(.settings-nav button:focus-visible){outline:none}
  .settings-shell :global(.settings-nav.tab-focus button:focus-visible){box-shadow:inset 0 0 0 2px var(--text-1)}
  .settings-shell :global(.settings-content){min-width:0;min-height:0;overflow:auto;padding:0 24px 20px}
  .settings-shell :global(.settings-content h1){margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}
  .settings-shell :global(.settings-content h1:focus){outline:none}
  .settings-shell :global(.settings-content>label){min-height:68px;padding-block:12px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}
  .settings-shell :global(.settings-content>label>select){flex:none}
  .settings-shell :global(.settings-content.updates-content){overflow:hidden;padding-bottom:0}
  .settings-shell :global(.clear-confirmation){width:100%;display:flex;align-items:center;justify-content:flex-end;gap:10px}
  .settings-shell :global(.clear-confirmation strong){min-width:0;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .settings-shell :global(.clear-confirmation button){min-width:92px;min-height:32px;padding:0 12px}
  .settings-shell :global(.clear-confirmation .danger){color:var(--danger-on);background:var(--danger-fill);font-weight:600}
</style>
