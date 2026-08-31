<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { canPreviewClip, copyClip, getHistoryFacets, getPreviewCapability, hidePanel, openClipLink, pasteClip, previewClip, type PreviewCapability } from "$lib/history/api";
  import type { ContentType, HistorySourceOption } from "$lib/history/types";
  import { canExpand, filePaths } from "$lib/history/presentation";
  import { HistorySession } from "$lib/history/session.svelte";
  import { PreviewSession } from "$lib/history/preview-session.svelte";
  import { exitsSearch, routeWindowKey } from "$lib/history/keyboard";
  import HistoryList from "$lib/history/HistoryList.svelte";
  import ClipPreview from "$lib/history/ClipPreview.svelte";
  import AppTitleBar from "$lib/history/AppTitleBar.svelte";
  import HistoryActionBar from "$lib/history/HistoryActionBar.svelte";
  import { getSettings, quitApp } from "$lib/settings/api";
  import { currentPlatform } from "$lib/settings/shortcuts";
  import { effectiveLocale, localizedError, t } from "$lib/i18n/index.svelte";
  import SettingsView from "$lib/settings/SettingsView.svelte";
  import { updateStore } from "$lib/updater/store.svelte";
  import OnboardingView from "$lib/onboarding/OnboardingView.svelte";
  import { getOnboardingState, type OnboardingState } from "$lib/onboarding/api";

  type InteractionMode = "browse" | "search" | "menu" | "confirmation" | "file-tablist";

  const session = new HistorySession();
  const preview = new PreviewSession();
  const isMac = currentPlatform() === "macos";
  let mode = $state<InteractionMode>("browse");
  let previewExternal = false;
  let previewCapability = $state<PreviewCapability>({ provider: "unavailable", reason: "detection_failed" });
  let confirmationInvoker: HTMLElement | null = null;
  let fileIndex = $state(0);
  let trimWhitespace = $state(false);
  let restoreBrowsePosition = $state(false);
  let preserveSearchConditions = $state(false);
  let expandedId = $state<string | null>(null);
  let error = $state("");
  let copied = $state("");
  let copiedTimer: number | undefined;
  let menuOpen = $state(false);
  let appMenuOpen = $state(false);
  let view = $state<"loading" | "history" | "settings" | "onboarding">("loading");
  let settingsTab = $state<"general" | "updates" | "about">("general");
  let onboarding = $state<OnboardingState | null>(null);
  let onboardingMode = $state<"first_run" | "quick_start" | "auto_paste">("first_run");
  let deletePending = $state(false);
  let rowReorderMotion = $state(false);
  let reducedMotion = $state(false);
  let listbox = $state<HistoryList>();
  let pageNavigationPending = false;
  let searchTimer: number | undefined;
  let sourceSearchTimer: number | undefined;
  let sources = $state<HistorySourceOption[]>([]);
  let typeTotal = $state(0);
  let typeCounts = $state<Partial<Record<ContentType, number>>>({});
  let activeSourceQuery = "";
  let facetsVersion = 0;
  const deleteShortcut = isMac ? "Command+Backspace" : "Ctrl+Delete";
  const settingsShortcut = isMac ? "Command+," : "Ctrl+,";
  const quitShortcut = isMac ? "Command+Q" : "Ctrl+Q";
  const previousFileShortcut = isMac ? "Command+ArrowLeft" : "Ctrl+ArrowLeft";
  const nextFileShortcut = isMac ? "Command+ArrowRight" : "Ctrl+ArrowRight";
  const actionMenuShortcut = isMac ? "Command+K" : "Ctrl+K";
  type MainPanelRequest = { selectedId: string | null; settings: boolean };

  $effect(() => {
    effectiveLocale();
    error = "";
  });

  onMount(() => {
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateReducedMotion = () => { reducedMotion = motionQuery.matches; };
    updateReducedMotion();
    motionQuery.addEventListener("change", updateReducedMotion);
    void initializeView();
    const unlistenClips = listen("history_changed", () => { void refresh(session.page.page); void syncFacets(); });
    const unlistenPanel = listen<MainPanelRequest>("main_panel_shown", ({ payload }) => void onPanelShown(payload));
    return () => {
      if (sourceSearchTimer !== undefined) window.clearTimeout(sourceSearchTimer);
      motionQuery.removeEventListener("change", updateReducedMotion);
      unlistenClips.then((fn) => fn());
      unlistenPanel.then((fn) => fn());
    };
  });

  async function initializeView() {
    let initializationError = "";
    try {
      await Promise.all([syncSettings(), refreshPreviewCapability()]);
      onboarding = await getOnboardingState();
      if (onboarding.completed_revision === null) {
        onboardingMode = "first_run";
        view = "onboarding";
        return;
      }
    } catch (reason) {
      initializationError = localizedError(reason);
    }
    view = "history";
    await syncFacets();
    await refreshAndFocus(1);
    if (initializationError && !error) error = initializationError;
  }

  async function refreshPreviewCapability() {
    try { previewCapability = await getPreviewCapability(); }
    catch { previewCapability = { provider: "unavailable", reason: "detection_failed" }; }
  }

  function canPreviewSelected() {
    return canPreviewClip(previewCapability, session.detail?.content_type);
  }

  async function syncSettings() {
    try {
      const settings = await getSettings();
      restoreBrowsePosition = settings.restore_browse_position;
      preserveSearchConditions = settings.preserve_search_conditions;
      trimWhitespace = settings.trim_whitespace;
    } catch (reason) {
      error = localizedError(reason);
    }
  }

  async function syncFacets(sourceQuery = activeSourceQuery) {
    try {
      const version = ++facetsVersion;
      activeSourceQuery = sourceQuery;
      const selected = sources.find(({ id }) => id === session.filters.source_id);
      const facets = await getHistoryFacets(session.query, session.filters, sourceQuery);
      if (version !== facetsVersion) return;
      typeTotal = facets.type_total;
      typeCounts = facets.type_counts;
      sources = selected && !facets.sources.some(({ id }) => id === selected.id)
        ? [selected, ...facets.sources].slice(0, 20)
        : facets.sources;
    }
    catch { /* Search remains usable without source suggestions. */ }
  }

  function onSourceQuery(query: string) {
    if (sourceSearchTimer !== undefined) window.clearTimeout(sourceSearchTimer);
    sourceSearchTimer = window.setTimeout(() => {
      sourceSearchTimer = undefined;
      void syncFacets(query);
    }, 120);
  }

  async function refresh(targetPage = session.page.page, selectLatest = false) {
    error = "";
    const previousSelection = session.selectedId;
    preview.resetPage();
    const applied = await session.refresh(targetPage, selectLatest);
    if (!applied) return false;
    if (session.selectedId !== previousSelection) resetPreviewState();
    if (session.errorReason) {
      error = localizedError(session.errorReason);
    } else {
      void preview.loadPageThumbnails(session.page.items);
    }
    await applySelectedDetail(false);
    return true;
  }

  async function refreshAndFocus(targetPage: number) {
    if (await refresh(targetPage)) enterBrowse();
  }

  function enterBrowse(focus = true) {
    mode = "browse";
    if (focus) requestAnimationFrame(() => listbox?.focus());
  }

  function enterSearch() {
    mode = "search";
    listbox?.focusSearch();
  }

  function onSearchKeydown(event: KeyboardEvent) {
    if (!exitsSearch(event.key)) return;
    event.preventDefault();
    event.stopPropagation();
    enterBrowse();
  }

  function pasteMessage(outcome: string) {
    if (outcome === "copied_permission_required") return t("paste.permission");
    if (outcome === "copied_target_lost") return t("paste.targetLost");
    if (outcome === "copied_focus_failed") return t("paste.focusFailed");
    if (outcome === "copied_injection_failed") return t("paste.injectionFailed");
    if (outcome === "already_in_progress") return t("paste.inProgress");
    return t("paste.unsupported");
  }

  async function select(id: string | null, readSelectedFile = false) {
    const selectionChanged = session.selectedId !== id;
    if (selectionChanged) expandedId = null;
    resetPreviewState();
    await session.select(id);
    if (session.errorReason) error = localizedError(session.errorReason);
    await applySelectedDetail(readSelectedFile);
  }

  async function applySelectedDetail(readSelectedFile: boolean) {
    const next = session.detail;
    const id = session.selectedId;
    if (!next || !id) return;
    // Auto-selecting the first row must not touch its original file. Only a
    // user click/key selection or preview request opts into that read.
    const readOriginalFile = readSelectedFile && next.content_type === "file";
    try {
      await preview.loadSelection(id, next, readOriginalFile);
      if (next.content_type === "image") void preview.prefetchAdjacentImages(session.page.items, id);
    } catch (reason) {
      error = localizedError(reason);
    }
  }

  async function pasteSelected(plainText = false) {
    if (!session.selectedId) return;
    if (plainText && session.detail?.plain_text == null) return;
    try {
      const outcome = await pasteClip(session.selectedId, plainText);
      if (outcome !== "pasted") {
        window.clearTimeout(copiedTimer);
        copied = pasteMessage(outcome);
      }
    } catch (reason) { error = localizedError(reason); }
    menuOpen = false;
    enterBrowse();
  }

  async function pastePlainSelected() {
    await pasteSelected(true);
  }

  async function copyOnly(plainText = false) {
    if (!session.selectedId) return;
    if (plainText && session.detail?.plain_text == null) return;
    try {
      const moved = await copyClip(session.selectedId, plainText);
      window.clearTimeout(copiedTimer);
      copied = plainText ? t("history.copiedPlain") : t("history.copied");
      await refresh(moved ? 1 : session.page.page, moved);
      copiedTimer = window.setTimeout(() => {
        copied = "";
      }, 1800);
    } catch (reason) { error = localizedError(reason); }
    menuOpen = false;
    enterBrowse();
  }

  async function removeSelected() {
    if (!session.selectedId) return;
    const deletedId = session.selectedId;
    resetPreviewState();
    try {
      rowReorderMotion = true;
      await session.deleteSelected();
      evictClip(deletedId);
      await applySelectedDetail(false);
      await tick();
      enterBrowse();
    }
    catch (reason) {
      error = localizedError(reason);
      await applySelectedDetail(false);
      mode = "browse";
      requestAnimationFrame(focusConfirmationInvoker);
    }
    finally {
      requestAnimationFrame(() => rowReorderMotion = false);
    }
    menuOpen = false;
  }

  function requestDelete(invoker?: HTMLElement | null) {
    if (!session.selectedId) return;
    confirmationInvoker = invoker === undefined
      ? document.activeElement instanceof HTMLElement ? document.activeElement : null
      : invoker;
    menuOpen = false;
    deletePending = true;
    mode = "confirmation";
  }

  function cancelDelete() {
    deletePending = false;
    mode = "browse";
  }

  function focusConfirmationInvoker() {
    if (confirmationInvoker?.isConnected) confirmationInvoker.focus();
    else listbox?.focus();
  }

  function confirmDelete() {
    deletePending = false;
    void removeSelected();
  }

  async function viewSelectedClip() {
    if (!session.selectedId) return;
    try {
      const outcome = await previewClip(session.selectedId, fileIndex);
      previewExternal = outcome === "native_opened";
      if (!previewExternal) enterBrowse();
    }
    catch (reason) { error = localizedError(reason); }
    menuOpen = false;
  }

  async function openSelectedLink(originOnly = false) {
    if (!session.selectedId || session.detail?.content_type !== "link") return;
    try {
      await openClipLink(session.selectedId, originOnly);
    } catch (reason) {
      error = localizedError(reason);
    }
    menuOpen = false;
    enterBrowse();
  }

  function onSearch() {
    if (searchTimer !== undefined) window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => {
      searchTimer = undefined;
      void refresh(1);
      void syncFacets();
    }, 120);
  }

  function onFiltersChange() {
    void refresh(1);
    void syncFacets();
  }

  function clearSearchConditions() {
    session.query = "";
    session.clearFilters();
    void refresh(1);
    void syncFacets();
  }

  function openMenu() {
    if (menuOpen) {
      closeMenu();
      return;
    }
    appMenuOpen = false;
    menuOpen = true;
    mode = "menu";
  }

  function setActionMenuOpen(open: boolean) {
    menuOpen = open;
    if (open) appMenuOpen = false;
    syncOverlayMode();
  }

  function setAppMenuOpen(open: boolean) {
    appMenuOpen = open;
    if (open) menuOpen = false;
    syncOverlayMode();
  }

  function setDeletePending(open: boolean) {
    deletePending = open;
    syncOverlayMode();
  }

  function syncOverlayMode() {
    mode = deletePending ? "confirmation" : menuOpen || appMenuOpen ? "menu" : "browse";
  }

  function closeMenu() {
    menuOpen = false;
    enterBrowse();
  }

  function closeAppMenu() {
    appMenuOpen = false;
    mode = "browse";
  }

  function updateModeFromFocus(event: FocusEvent) {
    if (!(event.target instanceof Element)) return;
    if (mode === "search" && !event.target.closest("input")) mode = "browse";
    if (mode === "file-tablist" && !event.target.closest("[role='tablist']")) mode = "browse";
  }

  async function openSettingsView(tab: "general" | "updates" | "about" = "general") {
    appMenuOpen = false;
    settingsTab = tab;
    view = "settings";
  }

  function checkForUpdates() {
    void openSettingsView("updates");
    void updateStore.check();
  }

  function openQuickStart() {
    onboardingMode = "quick_start";
    onboarding = { completed_revision: 1, current_step: "overview", visited_steps: ["overview"], selected_example: "image" };
    view = "onboarding";
  }

  async function finishOnboarding(returnToSettings: boolean) {
    await syncSettings();
    view = returnToSettings ? "settings" : "history";
    if (!returnToSettings) await refreshAndFocus(1);
  }

  async function closeSettingsView() {
    await syncSettings();
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    view = "history";
    mode = "browse";
    await tick();
    listbox?.focus();
  }

  function listHasFocus() {
    return listbox?.hasFocus() ?? false;
  }

  function selectFromList(id: string) {
    listbox?.focus();
    const item = session.page.items.find((candidate) => candidate.id === id);
    if (session.selectedId === id && item && canExpand(item)) {
      if (session.detail?.content_type === "file") {
        void preview.loadFile(id, fileIndex);
      }
      expandedId = expandedId === id ? null : id;
      return;
    }
    void select(id, true);
  }

  // A hotkey summon fires panel_shown. Settings and onboarding are deliberate modes,
  // so a summon must not discard them — the history session underneath stays live via
  // the history_changed listener and is current when the user exits. Within history,
  // Browse position and search conditions are independent saved-session choices.
  async function onPanelShown(request: MainPanelRequest) {
    void refreshPreviewCapability();
    listbox?.closeFilters();
    if (request.settings) {
      await openSettingsView();
      return;
    }
    if (request.selectedId) {
      await focusHistoryItem(request.selectedId);
      return;
    }
    if (view !== "history") return;
    if (!preserveSearchConditions) {
      session.query = "";
      session.clearFilters();
      activeSourceQuery = "";
    }
    if (restoreBrowsePosition) await resumeBrowse();
    else await resetToLatest();
    await syncFacets();
  }

  function clearWindowFocus() {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
  }

  async function resetToLatest() {
    menuOpen = false;
    appMenuOpen = false;
    view = "history";
    mode = "browse";
    await tick();
    listbox?.focus();
    await refresh(1, true);
    enterBrowse();
  }

  async function focusHistoryItem(id: string) {
    view = "history";
    session.query = "";
    session.clearFilters();
    session.selectedId = id;
    await refreshAndFocus(1);
  }

  // Keep page, selection and search; just refresh the current page for freshness and
  // return keyboard focus to the list. session.refresh preserves the selected id when
  // it is still present on the page.
  async function resumeBrowse() {
    menuOpen = false;
    appMenuOpen = false;
    await tick();
    listbox?.focus();
    await refresh(session.page.page);
    enterBrowse();
  }

  function suppressContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function onListKeydown(event: KeyboardEvent) {
    if (onKeydown(event)) return;
    const index = session.page.items.findIndex((item) => item.id === session.selectedId);
    const selectIndex = (next: number) => void select(session.page.items[Math.max(0, Math.min(next, session.page.items.length - 1))]?.id ?? null, true);
    const selected = session.page.items.find((item) => item.id === session.selectedId);
    if ((event.metaKey || event.ctrlKey) && selected && canExpand(selected) && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
      event.preventDefault();
      void selectFile(fileIndex + (event.key === "ArrowLeft" ? -1 : 1));
    }
    else if (event.key === "ArrowDown") { event.preventDefault(); void moveSelection(1); }
    else if (event.key === "ArrowUp") { event.preventDefault(); void moveSelection(-1); }
    else if (event.key === "Home") { event.preventDefault(); selectIndex(0); }
    else if (event.key === "End") { event.preventDefault(); selectIndex(session.page.items.length - 1); }
    else if (event.key === "PageDown" && session.page.page < session.page.total_pages) { event.preventDefault(); listbox?.turnPage(1); }
    else if (event.key === "PageUp" && session.page.page > 1) { event.preventDefault(); listbox?.turnPage(-1); }
    else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (expandedId === session.selectedId) expandedId = null;
      else if (session.page.page > 1) listbox?.turnPage(-1);
    }
    else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (session.page.page < session.page.total_pages) listbox?.turnPage(1);
    }
    else if ((event.key === " " || event.code === "Space") && canPreviewSelected()) {
      event.preventDefault();
      void viewSelectedClip();
    }
    else if (event.key === "Enter") {
      event.preventDefault();
      void pasteSelected(event.shiftKey);
    }
    else if ((event.metaKey || event.ctrlKey) && ["Backspace", "Delete"].includes(event.key)) {
      event.preventDefault();
      void requestDelete();
    }
    else if (/^[0-9]$/.test(event.key)) {
      const target = event.key === "0" ? 9 : Number(event.key) - 1;
      if (session.page.items[target]) { event.preventDefault(); void select(session.page.items[target].id, true); }
    }
    else if (event.key === "Escape") {
      event.preventDefault();
      void hidePanel();
    }
  }

  async function moveSelection(direction: -1 | 1) {
    if (pageNavigationPending || session.page.items.length === 0) return;
    const index = session.page.items.findIndex((item) => item.id === session.selectedId);
    const nextIndex = index + direction;
    if (nextIndex >= 0 && nextIndex < session.page.items.length) {
      await select(session.page.items[nextIndex]?.id ?? null, true);
      return;
    }
    if (direction < 0 && session.page.page > 1) {
      pageNavigationPending = true;
      try {
        await refresh(session.page.page - 1);
        await select(session.page.items.at(-1)?.id ?? null, true);
      } finally {
        pageNavigationPending = false;
      }
    } else if (direction > 0 && session.page.page < session.page.total_pages) {
      pageNavigationPending = true;
      try {
        await refresh(session.page.page + 1);
        await select(session.page.items[0]?.id ?? null, true);
      } finally {
        pageNavigationPending = false;
      }
    }
  }

  function onFileNavigatorKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.preventDefault();
    enterBrowse();
  }

  function onKeydown(event: KeyboardEvent) {
    if (deletePending) return;
    if (view === "settings") {
      return false;
    }
    if (mode !== "browse") return;
    if ((event.metaKey || event.ctrlKey) && event.key === ",") {
      event.preventDefault();
      void openSettingsView();
      return true;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
      event.preventDefault(); enterSearch(); return true;
    }
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "c") {
      event.preventDefault(); void copyOnly(true); return true;
    }
    if (event.key === "/") {
      event.preventDefault(); enterSearch(); return true;
    }
    if (event.key === "Escape") {
      return false;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault(); openMenu(); return true;
    }
    if (event.shiftKey && event.key === "F10" && listHasFocus()) {
      event.preventDefault(); openMenu();
      return true;
    }
    return false;
  }

  function onWindowKeydown(event: KeyboardEvent) {
    const action = routeWindowKey(event, { view, mode, deletePending, menuOpen, appMenuOpen });
    if (!action) return;
    event.preventDefault();
    if (action === "cancel-delete") cancelDelete();
    else if (action === "close-menu") closeMenu();
    else if (action === "close-app-menu") closeAppMenu();
    else if (action === "return-to-browse") enterBrowse();
    else {
      deletePending = false;
      menuOpen = false;
      appMenuOpen = false;
      void hidePanel();
    }
  }

  function restoreAfterNativePreview() {
    if (!previewExternal || view !== "history") return;
    previewExternal = false;
    enterBrowse();
  }

  async function selectFile(index: number) {
    if (!session.selectedId || !session.detail || session.detail.content_type !== "file") return;
    const paths = filePaths(session.detail);
    if (index < 0 || index >= paths.length || index === fileIndex) return;
    fileIndex = index;
    void preview.loadFile(session.selectedId, index);
  }

  function resetPreviewState() {
    preview.resetSelection();
    fileIndex = 0;
  }

  function evictClip(id: string) {
    session.evict(id);
    preview.evict(id);
  }

  function clearContentCaches() {
    session.clearCaches();
    preview.clear();
  }

  function settingsClearedHistory() {
    clearContentCaches();
    void refresh(1);
  }
</script>

<svelte:window onkeydown={onWindowKeydown} onfocusin={updateModeFromFocus} onfocus={restoreAfterNativePreview} onblur={clearWindowFocus} oncontextmenu={suppressContextMenu} />

<main class="panel" aria-label={t("history.panel")}>
  {#if view === "onboarding" && onboarding}
    <OnboardingView initial={onboarding} mode={onboardingMode} onfinish={finishOnboarding} />
  {:else if view === "loading"}
    <div class="root-loading" role="status">{t("settings.loading")}</div>
  {:else}
  <AppTitleBar
    history={view === "history"}
    open={appMenuOpen}
    {settingsShortcut}
    {quitShortcut}
    onopenchange={setAppMenuOpen}
    onsettings={() => void openSettingsView()}
    onupdates={checkForUpdates}
    onabout={() => void openSettingsView("about")}
    onquit={() => void quitApp()}
  />
  {#if view === "history"}
  <HistoryList
    bind:this={listbox}
    bind:query={session.query}
    filters={session.filters}
    {sources}
    {typeTotal}
    {typeCounts}
    page={session.page}
    selectedId={session.selectedId}
    {expandedId}
    {fileIndex}
    loading={session.loading}
    {error}
    thumbnailUrls={preview.thumbnailUrls}
    {reducedMotion}
    {rowReorderMotion}
    onsearch={onSearch}
    onsearchfocus={() => mode = "search"}
    onsearchkeydown={onSearchKeydown}
    onfilterschange={onFiltersChange}
    onsourcequery={onSourceQuery}
    onclearsearch={clearSearchConditions}
    onlistfocus={() => mode = "browse"}
    onselect={selectFromList}
    onpaste={() => void pasteSelected()}
    onfile={(index) => { mode = "file-tablist"; void selectFile(index); }}
    onkeydown={onListKeydown}
    onpage={(page) => void refreshAndFocus(page)}
  />

  <ClipPreview
    detail={session.detail}
    selectedId={session.selectedId}
    page={session.page}
    noMatches={Boolean(session.query || session.filters.content_type || session.filters.source_id || session.filters.time_range !== "any")}
    pending={session.detailPending}
    assetUrl={preview.assetUrl}
    thumbnailUrl={session.selectedId ? preview.thumbnailUrls[session.selectedId] ?? null : null}
    fileAccessDenied={preview.fileAccessDenied}
    sourceIconUrl={preview.sourceIconUrl}
    fileThumbnailUrls={preview.fileThumbnailUrls}
    fileByteSizes={preview.fileByteSizes}
    {fileIndex}
    {trimWhitespace}
    {previousFileShortcut}
    {nextFileShortcut}
    onfile={(index) => void selectFile(index)}
    onfilekeydown={onFileNavigatorKeydown}
    onfilefocus={() => mode = "file-tablist"}
    onopenorigin={() => void openSelectedLink(true)}
    oninert={() => enterBrowse()}
  />

  <HistoryActionBar
    selected={Boolean(session.selectedId)}
    canPreview={canPreviewSelected()}
    isLink={session.detail?.content_type === "link"}
    hasPlainText={session.detail?.plain_text != null}
    {isMac}
    {error}
    {copied}
    {menuOpen}
    {deletePending}
    {actionMenuShortcut}
    {deleteShortcut}
    onmenuopenchange={setActionMenuOpen}
    ondeleteopenchange={setDeletePending}
    onbrowse={() => enterBrowse()}
    onpreview={() => void viewSelectedClip()}
    onopenlink={() => void openSelectedLink()}
    onpasteplain={() => void pastePlainSelected()}
    oncopy={() => void copyOnly()}
    oncopyplain={() => void copyOnly(true)}
    onrequestdelete={requestDelete}
    oncanceldelete={cancelDelete}
    onconfirmdelete={confirmDelete}
    onpaste={() => void pasteSelected()}
    onrestorefocus={focusConfirmationInvoker}
  />
  {:else}
    <SettingsView initialTab={settingsTab} onclose={closeSettingsView} oncleared={settingsClearedHistory} onquickstart={openQuickStart} />
  {/if}
  {/if}
</main>

<style>
  .panel { width:calc(100vw - 40px); height:calc(100vh - 40px); margin:20px; display:grid; grid-template-columns:300px 1fr; grid-template-rows:42px 1fr 48px; background:var(--bg-shell); border-radius:var(--radius-xl); box-shadow:var(--panel-shadow); overflow:hidden; }
  .root-loading{grid-column:1/-1;grid-row:1/-1;display:grid;place-items:center;color:var(--text-2);font-size:var(--fs-body)}
  @media (min-width:840px) { .panel { grid-template-columns:320px 1fr; } }
  @media (max-width:680px) { .panel { grid-template-columns:280px 1fr; } }
  @media (prefers-reduced-motion:no-preference) { .panel { animation:enter 120ms ease-out; } @keyframes enter { from { opacity:0; transform:scale(.98); } } }
</style>
