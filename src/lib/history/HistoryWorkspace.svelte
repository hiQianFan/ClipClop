<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { copyClip, getClipAsset, getClipFileAsset, getClipThumbnail, getSourceAppIcon, hidePanel, pasteClip, previewClip } from "$lib/history/api";
  import type { ClipSummary } from "$lib/history/types";
  import { cacheSet, canExpand, errorMessage, filePaths, pasteFallbackMessage } from "$lib/history/presentation";
  import { HistorySession } from "$lib/history/session.svelte";
  import HistoryList from "$lib/history/HistoryList.svelte";
  import ClipPreview from "$lib/history/ClipPreview.svelte";
  import { quitApp } from "$lib/settings/api";
  import { effectiveLocale, t } from "$lib/i18n/index.svelte";
  import SettingsView from "$lib/settings/SettingsView.svelte";
  import { ArrowLeft } from "@lucide/svelte";

  const session = new HistorySession();
  let assetUrl = $state<string | null>(null);
  let sourceIconUrl = $state<string | null>(null);
  let thumbnailUrls = $state<Record<string, string>>({});
  let fileThumbnailUrls = $state<Array<string | null>>([]);
  let fileIndex = $state(0);
  let expandedId = $state<string | null>(null);
  let error = $state("");
  let copied = $state("");
  let menuOpen = $state(false);
  let appMenuOpen = $state(false);
  let view = $state<"history" | "settings">("history");
  let deletePending = $state(false);
  let rowReorderMotion = $state(false);
  let reducedMotion = $state(false);
  let listbox = $state<HistoryList>();
  let menuButton = $state<HTMLButtonElement>();
  let appMenuButton = $state<HTMLButtonElement>();
  let menuWrap = $state<HTMLDivElement>();
  let appMenuWrap = $state<HTMLDivElement>();
  let cancelActionButton = $state<HTMLButtonElement>();
  let confirmActionButton = $state<HTMLButtonElement>();
  let requestVersion = 0;
  let thumbnailRequestVersion = 0;
  let pageNavigationPending = false;
  let assetTimer: number | undefined;
  let searchTimer: number | undefined;
  const assetCache = new Map<string, { data_url: string | null; byte_size: number | null }>();
  const thumbnailCache = new Map<string, string>();
  const sourceIconCache = new Map<string, string | null>();
  const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);
  const deleteShortcut = isMac ? "⌘⌫" : "Ctrl⌫";
  const settingsShortcut = isMac ? "⌘," : "Ctrl,";
  const previousFileShortcut = isMac ? "⌘←" : "Ctrl←";
  const nextFileShortcut = isMac ? "⌘→" : "Ctrl→";
  const actionMenuShortcut = isMac ? "⌘K" : "Ctrl K";

  $effect(() => {
    effectiveLocale();
    copied = "";
    error = "";
  });

  onMount(() => {
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateReducedMotion = () => { reducedMotion = motionQuery.matches; };
    updateReducedMotion();
    motionQuery.addEventListener("change", updateReducedMotion);
    const captureEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" || event.key === "Esc" || event.code === "Escape") {
        // Settings owns its nested Escape hierarchy (recording, confirmation,
        // then return). Let its window handler receive the event.
        if (view === "settings") return;
        event.preventDefault();
        event.stopImmediatePropagation();
        if (deletePending) {
          cancelDelete();
        } else if (menuOpen) {
          closeMenu();
        } else if (appMenuOpen) {
          closeAppMenu();
        } else {
          void hidePanel();
        }
      }
    };
    document.addEventListener("keydown", captureEscape, true);
    void refresh(1);
    const unlistenClips = listen("history_changed", () => refresh(session.page.page));
    // Only an explicit panel show is a new browsing session. Quick Look also
    // returns focus to this window, but must preserve the current selection.
    const unlistenPanel = listen("panel_shown", () => void resetToLatest());
    return () => {
      document.removeEventListener("keydown", captureEscape, true);
      motionQuery.removeEventListener("change", updateReducedMotion);
      unlistenClips.then((fn) => fn());
      unlistenPanel.then((fn) => fn());
    };
  });

  async function refresh(targetPage = session.page.page, selectLatest = false) {
    error = "";
    const thumbnailVersion = ++thumbnailRequestVersion;
    await session.refresh(targetPage, selectLatest);
    if (session.errorReason) {
      error = errorMessage(session.errorReason);
    } else {
      thumbnailUrls = Object.fromEntries(session.page.items.flatMap((item) => {
        const thumbnail = thumbnailCache.get(item.id);
        return thumbnail ? [[item.id, thumbnail]] : [];
      }));
      void loadThumbnails(session.page.items, thumbnailVersion);
    }
    await applySelectedDetail(false);
  }

  async function loadThumbnails(items: ClipSummary[], version: number) {
    // File thumbnails require reading the original path. Do not touch protected
    // folders (Downloads, Desktop, Documents) merely by opening the panel.
    const mediaItems = items.filter((item) => item.content_type === "image" && !thumbnailCache.has(item.id));
    for (const item of mediaItems) {
      if (version !== thumbnailRequestVersion) return;
      try {
        const thumbnail = await getClipThumbnail(item.id);
        if (thumbnail.data_url) thumbnailCache.set(item.id, thumbnail.data_url);
      } catch { /* A neutral file icon is an intentional fallback. */ }
      if (version === thumbnailRequestVersion) {
        thumbnailUrls = Object.fromEntries(items.flatMap((current) => {
          const thumbnail = thumbnailCache.get(current.id);
          return thumbnail ? [[current.id, thumbnail]] : [];
        }));
      }
    }
  }

  async function select(id: string | null, readSelectedFile = false) {
    const version = ++requestVersion;
    const selectionChanged = session.selectedId !== id;
    if (selectionChanged) expandedId = null;
    if (assetTimer !== undefined) window.clearTimeout(assetTimer);
    assetUrl = null;
    fileThumbnailUrls = [];
    fileIndex = 0;
    sourceIconUrl = null;
    await session.select(id);
    if (session.errorReason) error = errorMessage(session.errorReason);
    if (version === requestVersion) await applySelectedDetail(readSelectedFile, version);
  }

  async function applySelectedDetail(readSelectedFile: boolean, version = ++requestVersion) {
    const next = session.detail;
    const id = session.selectedId;
    if (!next || !id) return;
    if (next.source_app) {
          const cachedIcon = sourceIconCache.get(next.source_app.id);
          if (cachedIcon !== undefined) sourceIconUrl = cachedIcon;
          else getSourceAppIcon(id).then((icon) => {
            cacheSet(sourceIconCache, next.source_app!.id, icon.data_url);
            if (version === requestVersion) sourceIconUrl = icon.data_url;
          }).catch(() => cacheSet(sourceIconCache, next.source_app!.id, null));
    }
    if (next.content_type === "image") scheduleAsset(id, null, version);
    // Auto-selecting the first row must not touch its original file. Only a
    // user click/key selection or preview request opts into that read.
    if (next.content_type === "file" && readSelectedFile) scheduleAsset(id, 0, version);
  }

  async function pasteSelected(plainText = false) {
    if (!session.selectedId) return;
    if (plainText && session.detail?.plain_text == null) return;
    try {
      const outcome = await pasteClip(session.selectedId, plainText);
      if (outcome !== "pasted") {
        copied = pasteFallbackMessage(outcome);
        setTimeout(() => copied = "", 3200);
      }
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function pastePlainSelected() {
    await pasteSelected(true);
  }

  async function copyOnly(plainText = false) {
    if (!session.selectedId) return;
    if (plainText && session.detail?.plain_text == null) return;
    try {
      await copyClip(session.selectedId, plainText);
      copied = plainText ? t("history.copiedPlain") : t("history.copied");
      setTimeout(() => copied = "", 1800);
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function removeSelected() {
    if (!session.selectedId) return;
    const deletedId = session.selectedId;
    try {
      rowReorderMotion = true;
      await session.deleteSelected();
      evictClip(deletedId);
      await applySelectedDetail(false);
      await tick();
      listbox?.focus();
    }
    catch (reason) { error = errorMessage(reason); }
    finally {
      requestAnimationFrame(() => rowReorderMotion = false);
    }
    menuOpen = false;
  }

  async function requestDelete() {
    if (!session.selectedId) return;
    menuOpen = false;
    deletePending = true;
    await tick();
    confirmActionButton?.focus();
  }

  function cancelDelete() {
    deletePending = false;
    requestAnimationFrame(() => listbox?.focus());
  }

  function confirmDelete() {
    deletePending = false;
    void removeSelected();
  }

  async function viewSelectedClip() {
    if (!session.selectedId) return;
    try {
      await previewClip(session.selectedId, fileIndex);
    }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  function onSearch() {
    if (searchTimer !== undefined) window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => {
      searchTimer = undefined;
      void refresh(1);
    }, 120);
  }

  async function openMenu() {
    if (menuOpen) {
      closeMenu();
      return;
    }
    appMenuOpen = false;
    menuOpen = true;
    await tick();
    menuItemElements().find((item) => !item.disabled)?.focus();
  }

  function closeMenu() {
    menuOpen = false;
    requestAnimationFrame(() => menuButton?.focus());
  }

  async function toggleAppMenu() {
    if (appMenuOpen) {
      closeAppMenu();
      return;
    }
    menuOpen = false;
    appMenuOpen = true;
    await tick();
    menuItemElements().find((item) => !item.disabled)?.focus();
  }

  function closeAppMenu() {
    appMenuOpen = false;
    requestAnimationFrame(() => appMenuButton?.focus());
  }

  function dismissMenusFromOutsidePointer(event: PointerEvent) {
    if (!(event.target instanceof Node)) return;
    if (menuOpen && !menuWrap?.contains(event.target)) menuOpen = false;
    if (appMenuOpen && !appMenuWrap?.contains(event.target)) appMenuOpen = false;
  }

  function dismissMenusFromOutsideFocus(event: FocusEvent) {
    if (!(event.target instanceof Node)) return;
    if (menuOpen && !menuWrap?.contains(event.target)) menuOpen = false;
    if (appMenuOpen && !appMenuWrap?.contains(event.target)) appMenuOpen = false;
  }

  async function openSettingsView() {
    appMenuOpen = false;
    view = "settings";
  }

  function closeSettingsView() {
    view = "history";
    requestAnimationFrame(() => listbox?.focus());
  }

  function menuItemElements() {
    return Array.from(document.querySelectorAll<HTMLButtonElement>("[data-menu-item]"));
  }

  function onMenuKeydown(event: KeyboardEvent) {
    const items = menuItemElements().filter((item) => !item.disabled);
    const index = items.indexOf(document.activeElement as HTMLButtonElement);
    if (event.key === "ArrowDown" || event.key === "ArrowUp" || event.key === "Home" || event.key === "End") {
      event.preventDefault();
      const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1
        : (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      items[next]?.focus();
    }
  }

  function listHasFocus() {
    return listbox?.hasFocus() ?? false;
  }

  function selectFromList(id: string) {
    listbox?.focus();
    const item = session.page.items.find((candidate) => candidate.id === id);
    if (session.selectedId === id && item && canExpand(item)) {
      if (session.detail?.content_type === "file") scheduleAsset(id, fileIndex, requestVersion);
      expandedId = expandedId === id ? null : id;
      return;
    }
    void select(id, true);
  }

  async function resetToLatest() {
    menuOpen = false;
    appMenuOpen = false;
    view = "history";
    session.query = "";
    clearContentCaches();
    await tick();
    listbox?.focus();
    await refresh(1, true);
    if (document.activeElement === document.body || listbox?.hasFocus()) {
      requestAnimationFrame(() => listbox?.focus());
    }
  }

  function suppressContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function onListKeydown(event: KeyboardEvent) {
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
    else if (event.key === "PageDown" && session.page.page < session.page.total_pages) { event.preventDefault(); void refresh(session.page.page + 1); }
    else if (event.key === "PageUp" && session.page.page > 1) { event.preventDefault(); void refresh(session.page.page - 1); }
    else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (expandedId === session.selectedId) expandedId = null;
      else if (session.page.page > 1) void refresh(session.page.page - 1);
    }
    else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (session.page.page < session.page.total_pages) void refresh(session.page.page + 1);
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
    if (!session.detail || session.detail.content_type !== "file") return;
    const lastIndex = filePaths(session.detail).length - 1;
    const next = event.key === "ArrowLeft" ? fileIndex - 1
      : event.key === "ArrowRight" ? fileIndex + 1
      : event.key === "Home" ? 0
      : event.key === "End" ? lastIndex
      : null;
    if (next === null || next < 0 || next > lastIndex) return;
    event.preventDefault();
    void selectFile(next);
    requestAnimationFrame(() => document.querySelector<HTMLButtonElement>(`[data-file-index="${next}"]`)?.focus());
  }

  function onKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "w") {
      event.preventDefault();
      deletePending = false;
      menuOpen = false;
      appMenuOpen = false;
      void hidePanel();
      return;
    }
    if (deletePending) return;
    if (view === "settings") {
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key === ",") {
      event.preventDefault();
      void openSettingsView();
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
      event.preventDefault(); listbox?.focusSearch(); return;
    }
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "c") {
      event.preventDefault(); void copyOnly(true); return;
    }
    if (event.key === "/" && listHasFocus()) {
      event.preventDefault(); listbox?.focusSearch(); return;
    }
    if (event.key === "Escape") {
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault(); void openMenu(); return;
    }
    if (event.shiftKey && event.key === "F10" && listHasFocus()) {
      event.preventDefault(); void openMenu();
    }
  }

  async function selectFile(index: number) {
    if (!session.selectedId || !session.detail || session.detail.content_type !== "file") return;
    const paths = filePaths(session.detail);
    if (index < 0 || index >= paths.length || index === fileIndex) return;
    fileIndex = index;
    assetUrl = null;
    scheduleAsset(session.selectedId, index, requestVersion);
  }

  function assetKey(id: string, index: number | null) {
    return `${id}:${index ?? "image"}`;
  }

  function scheduleAsset(id: string, index: number | null, version: number) {
    const key = assetKey(id, index);
    const cached = assetCache.get(key);
    if (cached) {
      assetUrl = cached.data_url;
      if (index !== null) applyFileAsset(index, cached);
      return;
    }
    assetTimer = window.setTimeout(() => {
      assetTimer = undefined;
      const request = index === null ? getClipAsset(id) : getClipFileAsset(id, index);
      request.then((asset) => {
        cacheSet(assetCache, key, asset);
        if (version === requestVersion && (index === null || index === fileIndex)) {
          assetUrl = asset.data_url;
          if (index !== null) applyFileAsset(index, asset);
        }
      }).catch((reason) => {
        if (version === requestVersion) error = errorMessage(reason);
      });
    }, 80);
  }

  function applyFileAsset(index: number, asset: { data_url: string | null; byte_size: number | null }) {
    fileThumbnailUrls[index] = asset.data_url;
    if (!session.detail || asset.byte_size === null) return;
    const sizes = [...(session.detail.metadata.file_sizes ?? [])];
    sizes[index] = asset.byte_size;
    session.detail.metadata.file_sizes = sizes;
  }

  function evictClip(id: string) {
    session.clearCaches();
    thumbnailCache.delete(id);
    for (const key of assetCache.keys()) if (key.startsWith(`${id}:`)) assetCache.delete(key);
  }

  function clearContentCaches() {
    session.clearCaches();
    assetCache.clear();
    thumbnailCache.clear();
    sourceIconCache.clear();
  }

  function settingsClearedHistory() {
    clearContentCaches();
    void refresh(1);
  }
</script>

<svelte:window onkeydown={onKeydown} onpointerdown={dismissMenusFromOutsidePointer} onfocusin={dismissMenusFromOutsideFocus} oncontextmenu={suppressContextMenu} />

<main class="panel" aria-label={t("history.panel")}>
  <header class="titlebar">
    {#if view === "history"}
      <div class="brand">
        <div bind:this={appMenuWrap} class="app-menu-wrap">
          <button bind:this={appMenuButton} class="app-menu-trigger" aria-label={t("history.appMenu")} aria-haspopup="menu" aria-expanded={appMenuOpen} onclick={() => void toggleAppMenu()}>
            <span class="brand-mark" aria-hidden="true"></span>
            <span>ClipClop</span>
          </button>
          {#if appMenuOpen}
            <div class="menu app-menu" role="menu" tabindex="-1" aria-label={t("history.appMenu")} onkeydown={onMenuKeydown}>
              <button data-menu-item role="menuitem" onclick={() => { appMenuOpen = false; void openSettingsView(); }}>{t("history.settings")} <kbd>{settingsShortcut}</kbd></button>
              <button data-menu-item role="menuitem" class="danger" onclick={() => void quitApp()}>{t("history.quit")}</button>
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <button class="back" aria-label={t("history.back")} onclick={closeSettingsView}><ArrowLeft size={16} aria-hidden="true" /></button>
      <span class="settings-title">{t("settings.title")}</span>
    {/if}
    <div class="titlebar-drag" data-tauri-drag-region></div>
  </header>
  {#if view === "history"}
  <HistoryList
    bind:this={listbox}
    bind:query={session.query}
    page={session.page}
    selectedId={session.selectedId}
    {expandedId}
    {fileIndex}
    loading={session.loading}
    {error}
    {thumbnailUrls}
    {reducedMotion}
    {rowReorderMotion}
    onsearch={onSearch}
    onselect={selectFromList}
    onpaste={() => void pasteSelected()}
    onfile={(index) => void selectFile(index)}
    onkeydown={onListKeydown}
    onpage={(page) => void refresh(page)}
  />

  <ClipPreview
    detail={session.detail}
    selectedId={session.selectedId}
    page={session.page}
    pending={session.detailPending}
    {assetUrl}
    {sourceIconUrl}
    {fileThumbnailUrls}
    {fileIndex}
    {previousFileShortcut}
    {nextFileShortcut}
    onfile={(index) => void selectFile(index)}
    onfilekeydown={onFileNavigatorKeydown}
  />

  <footer class="actions">
    {#if deletePending}
      <div class="confirmation" role="alertdialog" aria-label={t("history.confirmDeleteLabel")}>
        <span>{t("history.confirmDelete")}<small>{t("history.confirmDeleteHelp")}</small></span>
        <button bind:this={cancelActionButton} class="ghost" onclick={cancelDelete}>{t("common.cancel")} <kbd>Esc</kbd></button>
        <button bind:this={confirmActionButton} class="destructive" onclick={confirmDelete}>{t("history.delete")}</button>
      </div>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if copied}<span class="message">{copied}</span>{/if}
      <div bind:this={menuWrap} class="menu-wrap">
        <button bind:this={menuButton} class:expanded={menuOpen} class="ghost action-menu-trigger" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => void openMenu()}><kbd>{actionMenuShortcut}</kbd> {t("history.actions")}</button>
        {#if menuOpen}
          <div class="menu action-menu" role="menu" tabindex="-1" aria-label={t("history.actionMenu")} onkeydown={onMenuKeydown}>
            <button data-menu-item role="menuitem" onclick={() => void viewSelectedClip()} disabled={!session.selectedId}><span>{t("history.viewSelected")}</span><kbd>Space</kbd></button>
            <div class="menu-separator" role="separator"></div>
            <button data-menu-item role="menuitem" onclick={() => void pastePlainSelected()} disabled={session.detail?.plain_text == null}><span>{t("history.pastePlain")}</span><kbd>⇧⏎</kbd></button>
            <button data-menu-item role="menuitem" onclick={() => void copyOnly()} disabled={!session.selectedId}><span>{t("history.copy")}</span></button>
            <button data-menu-item role="menuitem" onclick={() => void copyOnly(true)} disabled={session.detail?.plain_text == null}><span>{t("history.copyPlain")}</span><kbd>{isMac ? "⌘⇧C" : "Ctrl⇧C"}</kbd></button>
            <div class="menu-separator" role="separator"></div>
            <button data-menu-item role="menuitem" class="danger" onclick={() => void requestDelete()} disabled={!session.selectedId}><span>{t("history.deleteFrom")}</span><kbd>{deleteShortcut}</kbd></button>
          </div>
        {/if}
      </div>
      <button class="copy" onclick={() => pasteSelected()} disabled={!session.selectedId}><kbd>⏎</kbd> {t("history.paste")}</button>
    {/if}
  </footer>
  {:else}
    <SettingsView onclose={closeSettingsView} oncleared={settingsClearedHistory} />
  {/if}
</main>

<style>
  .panel { width:calc(100vw - 40px); height:calc(100vh - 40px); margin:20px; display:grid; grid-template-columns:300px 1fr; grid-template-rows:42px 1fr 48px; background:var(--bg-shell); border-radius:14px; box-shadow:var(--panel-shadow); overflow:hidden; }
  .titlebar { grid-column:1 / -1; grid-row:1; display:flex; align-items:center; padding:0 14px; border-bottom:1px solid var(--hairline); user-select:none; }
  .titlebar-drag { flex:1; align-self:stretch; }
  .brand { display:flex; align-items:center; color:var(--text-2); }
  .app-menu-wrap { position:relative; }
  .app-menu-trigger { height:24px; display:flex; align-items:center; gap:4px; padding:0 4px; border-radius:5px; color:var(--text-2); background:transparent; font-size:12px; font-weight:600; letter-spacing:.01em; }
  .app-menu-trigger:hover { background:var(--bg-hover); }
  .brand-mark { width:14px; height:14px; flex:none; background:currentColor; mask:url("/clipclop-mark.svg") center/contain no-repeat; -webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat; }
  .back { width:24px; height:24px; padding:0; border-radius:5px; color:var(--text-2); background:transparent; font-size:16px; }
  .back:hover { background:var(--bg-hover); }
  .settings-title { margin-left:7px; color:var(--text-2); font-size:12px; font-weight:600; }
  kbd { font:10px/1.4 var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:4px; padding:1px 5px; white-space:nowrap; }
  .actions { grid-column:2; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, .ghost, .destructive { display:flex; align-items:center; gap:6px; border-radius:6px; color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:650; }
  .copy:hover { background:var(--action-hover); }
  .copy:active { filter:brightness(.92); }
  .copy kbd { color:inherit; border-color:currentColor; opacity:.9; }
  .ghost:hover, .ghost.expanded { color:var(--text-1); background:var(--bg-hover); }
  .ghost:active, .ghost.expanded:active { background:var(--bg-selected); }
  .action-menu-trigger.expanded kbd { color:inherit; border-color:currentColor; }
  .destructive { color:var(--danger-on); background:var(--danger-fill); font-weight:600; }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu { position:absolute; right:0; bottom:38px; width:210px; padding:6px; border:1px solid var(--hairline); border-radius:8px; background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .action-menu { width:260px; }
  .app-menu { top:30px; bottom:auto; left:0; right:auto; width:180px; }
  .menu button { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:6px; color:var(--text-1); background:transparent; line-height:1.4; text-align:left; }
  .menu button > span { min-width:0; }
  .menu button > kbd { flex:none; align-self:center; }
  .menu button:hover { background:var(--bg-hover); }
  .menu-separator { height:1px; margin:5px 6px; background:var(--hairline); }
  .menu .danger { color:var(--danger); }
  .menu .danger kbd { color:currentColor; border-color:currentColor; }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:11px; }
  .message.error { color:var(--danger); }
  .confirmation { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  .confirmation > span { margin-right:auto; color:var(--text-1); font-size:12px; font-weight:600; }
  .confirmation small { display:block; margin-top:2px; color:var(--text-2); font-size:10px; font-weight:400; }
  @media (min-width:840px) { .panel { grid-template-columns:320px 1fr; } }
  @media (max-width:680px) { .panel { grid-template-columns:280px 1fr; } }
  @media (prefers-reduced-motion:no-preference) { .panel { animation:enter 120ms ease-out; } @keyframes enter { from { opacity:0; transform:scale(.98); } } }
</style>
