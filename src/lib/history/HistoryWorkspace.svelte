<script lang="ts">
  import { onMount, tick } from "svelte";
  import { AlertDialog, DropdownMenu } from "bits-ui";
  import { listen } from "@tauri-apps/api/event";
  import { copyClip, getHistoryFacets, hidePanel, openClipLink, pasteClip, previewClip } from "$lib/history/api";
  import type { ContentType, HistorySourceOption } from "$lib/history/types";
  import { canExpand, filePaths } from "$lib/history/presentation";
  import { HistorySession } from "$lib/history/session.svelte";
  import { PreviewSession } from "$lib/history/preview-session.svelte";
  import { exitsSearch, routeWindowKey } from "$lib/history/keyboard";
  import HistoryList from "$lib/history/HistoryList.svelte";
  import ClipPreview from "$lib/history/ClipPreview.svelte";
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
  let appMenuButton = $state<HTMLButtonElement | null>(null);
  let menuButton = $state<HTMLButtonElement | null>(null);
  let confirmActionButton = $state<HTMLButtonElement | null>(null);
  let pageNavigationPending = false;
  let searchTimer: number | undefined;
  let sourceSearchTimer: number | undefined;
  let sources = $state<HistorySourceOption[]>([]);
  let typeTotal = $state(0);
  let typeCounts = $state<Partial<Record<ContentType, number>>>({});
  let activeSourceQuery = "";
  let facetsVersion = 0;
  const deleteShortcut = isMac ? "⌘⌫" : "Ctrl⌫";
  const settingsShortcut = isMac ? "⌘," : "Ctrl,";
  const quitShortcut = isMac ? "⌘Q" : "Ctrl+Q";
  const previousFileShortcut = isMac ? "⌘←" : "Ctrl←";
  const nextFileShortcut = isMac ? "⌘→" : "Ctrl→";
  const actionMenuShortcut = isMac ? "⌘K" : "Ctrl K";
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
      await syncSettings();
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

  function requestDelete() {
    if (!session.selectedId) return;
    confirmationInvoker = document.activeElement instanceof HTMLElement
      ? document.activeElement.closest("[role='menuitem']") ? menuButton ?? null : document.activeElement
      : null;
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
    view = "history";
    enterBrowse();
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
    else if (event.key === " " || event.code === "Space") {
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

<svelte:window onkeydown={onWindowKeydown} onfocusin={updateModeFromFocus} onfocus={restoreAfterNativePreview} oncontextmenu={suppressContextMenu} />

<main class="panel" aria-label={t("history.panel")}>
  {#if view === "onboarding" && onboarding}
    <OnboardingView initial={onboarding} mode={onboardingMode} onfinish={finishOnboarding} />
  {:else if view === "loading"}
    <div class="root-loading" role="status">{t("settings.loading")}</div>
  {:else}
  <header class="titlebar">
    {#if view === "history"}
      <div class="brand">
        <DropdownMenu.Root open={appMenuOpen} onOpenChange={(open) => { appMenuOpen = open; mode = open ? "menu" : "browse"; if (open) menuOpen = false; }}>
        <div class="app-menu-wrap">
          <DropdownMenu.Trigger bind:ref={appMenuButton} class="app-menu-trigger" aria-label={t("history.appMenu")}>
            <span class="brand-mark" aria-hidden="true"></span>
            <span>ClipClop</span>
          </DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu app-menu" aria-label={t("history.appMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); appMenuButton?.focus(); }}>
            <DropdownMenu.Item onclick={() => void openSettingsView()}>{t("history.settings")} <kbd>{settingsShortcut}</kbd></DropdownMenu.Item>
            <DropdownMenu.Item onclick={checkForUpdates}>{t("history.checkUpdates")}</DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => void openSettingsView("about")}>{t("history.about")}</DropdownMenu.Item>
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={() => void quitApp()}><span>{t("history.quit")}</span><kbd>{quitShortcut}</kbd></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
        </div>
        </DropdownMenu.Root>
      </div>
    {:else}
      <span class="settings-title">{t("settings.title")}</span>
    {/if}
    <div class="titlebar-drag" data-tauri-drag-region></div>
  </header>
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

  <AlertDialog.Root open={deletePending} onOpenChange={(open) => { deletePending = open; if (!open) mode = "browse"; }}>
  <footer class="actions">
    {#if deletePending}
      <AlertDialog.Content class="confirmation" aria-label={t("history.confirmDeleteLabel")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmActionButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); focusConfirmationInvoker(); }}>
        <span>{t("history.confirmDelete")}<small>{t("history.confirmDeleteHelp")}</small></span>
        <AlertDialog.Cancel class="ghost" onclick={cancelDelete}>{t("common.cancel")} <kbd>Esc</kbd></AlertDialog.Cancel>
        <AlertDialog.Action bind:ref={confirmActionButton} class="destructive" onclick={confirmDelete}>{t("history.delete")}</AlertDialog.Action>
      </AlertDialog.Content>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if copied}<span class="message">{copied}</span>{/if}
      <DropdownMenu.Root open={menuOpen} onOpenChange={(open) => { menuOpen = open; mode = open ? "menu" : "browse"; if (open) appMenuOpen = false; }}>
      <div class="menu-wrap">
        <DropdownMenu.Trigger bind:ref={menuButton} class={`ghost action-menu-trigger${menuOpen ? " expanded" : ""}`} disabled={!session.selectedId}><kbd>{actionMenuShortcut}</kbd> {t("history.actions")}</DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu action-menu" aria-label={t("history.actionMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); enterBrowse(); }}>
            <DropdownMenu.Item onclick={() => void viewSelectedClip()}><span>{t("history.viewSelected")}</span><kbd>Space</kbd></DropdownMenu.Item>
            {#if session.detail?.content_type === "link"}
              <DropdownMenu.Item onclick={() => void openSelectedLink()}><span>{t("history.openLink")}</span></DropdownMenu.Item>
            {/if}
            <DropdownMenu.Separator class="menu-separator" />
            {#if session.detail?.plain_text != null}
              <DropdownMenu.Item onclick={() => void pastePlainSelected()}><span>{t("history.pastePlain")}</span><kbd>⇧⏎</kbd></DropdownMenu.Item>
            {/if}
            <DropdownMenu.Item onclick={() => void copyOnly()}><span>{t("history.copy")}</span></DropdownMenu.Item>
            {#if session.detail?.plain_text != null}
              <DropdownMenu.Item onclick={() => void copyOnly(true)}><span>{t("history.copyPlain")}</span><kbd>{isMac ? "⌘⇧C" : "Ctrl⇧C"}</kbd></DropdownMenu.Item>
            {/if}
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={() => void requestDelete()}><span>{t("history.deleteFrom")}</span><kbd>{deleteShortcut}</kbd></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
      </div>
      </DropdownMenu.Root>
      <button class="copy" onclick={() => pasteSelected()} disabled={!session.selectedId}><kbd>⏎</kbd> {t("history.paste")}</button>
    {/if}
  </footer>
  </AlertDialog.Root>
  {:else}
    <SettingsView initialTab={settingsTab} onclose={closeSettingsView} oncleared={settingsClearedHistory} onquickstart={openQuickStart} />
  {/if}
  {/if}
</main>

<style>
  .panel { width:calc(100vw - 40px); height:calc(100vh - 40px); margin:20px; display:grid; grid-template-columns:300px 1fr; grid-template-rows:42px 1fr 48px; background:var(--bg-shell); border-radius:var(--radius-xl); box-shadow:var(--panel-shadow); overflow:hidden; }
  .root-loading{grid-column:1/-1;grid-row:1/-1;display:grid;place-items:center;color:var(--text-2);font-size:var(--fs-body)}
  .titlebar { grid-column:1 / -1; grid-row:1; display:flex; align-items:center; padding:0 14px; border-bottom:1px solid var(--hairline); user-select:none; }
  .titlebar-drag { flex:1; align-self:stretch; }
  .brand { display:flex; align-items:center; color:var(--text-2); }
  .app-menu-wrap { position:relative; }
  :global(.app-menu-trigger) { height:24px; display:flex; align-items:center; gap:4px; padding:0 4px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; font-size:var(--fs-ui); font-weight:600; letter-spacing:.01em; }
  :global(.app-menu-trigger:hover) { background:var(--bg-hover); }
  .brand-mark { width:14px; height:14px; flex:none; background:currentColor; mask:url("/clipclop-mark.svg") center/contain no-repeat; -webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat; }
  .settings-title { color:var(--text-1); font-size:var(--fs-emphasis); font-weight:600; }
  kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:var(--radius-sm); padding:1px 5px; white-space:nowrap; }
  .actions { grid-column:2; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, :global(.ghost), :global(.destructive) { display:flex; align-items:center; gap:6px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:650; }
  .copy:hover { background:var(--action-hover); }
  .copy:active { filter:brightness(.92); }
  .copy kbd { color:inherit; border-color:currentColor; opacity:.9; }
  :global(.ghost:hover), :global(.ghost.expanded) { color:var(--text-1); background:var(--bg-hover); }
  :global(.ghost:active), :global(.ghost.expanded:active) { background:var(--bg-selected); }
  :global(.action-menu-trigger.expanded) kbd { color:inherit; border-color:currentColor; }
  :global(.destructive) { color:var(--danger-on); background:var(--danger-fill); font-weight:600; }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  :global(.menu) { position:absolute; right:0; bottom:38px; width:210px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  :global(.action-menu) { width:260px; }
  :global(.app-menu) { top:30px; bottom:auto; left:0; right:auto; width:180px; }
  :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; }
  :global(.menu [role="menuitem"] > span) { min-width:0; }
  :global(.menu [role="menuitem"] > kbd) { flex:none; align-self:center; font-family:inherit; font-size:var(--fs-body); font-weight:500; line-height:1; }
  :global(.menu [role="menuitem"]:hover), :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  :global(.menu .danger) { color:var(--danger); }
  :global(.menu .danger kbd) { color:currentColor; border-color:currentColor; }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:var(--fs-meta); }
  .message.error { color:var(--danger); }
  :global(.confirmation) { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  :global(.confirmation button) { min-width:92px; min-height:32px; justify-content:center; padding:0 12px; }
  :global(.confirmation > span) { margin-right:auto; color:var(--text-1); font-size:var(--fs-ui); font-weight:600; }
  :global(.confirmation small) { display:block; margin-top:2px; color:var(--text-2); font-size:var(--fs-caption); font-weight:400; }
  @media (min-width:840px) { .panel { grid-template-columns:320px 1fr; } }
  @media (max-width:680px) { .panel { grid-template-columns:280px 1fr; } }
  @media (prefers-reduced-motion:no-preference) { .panel { animation:enter 120ms ease-out; } @keyframes enter { from { opacity:0; transform:scale(.98); } } }
</style>
