<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { copyClip, deleteClip, getClip, getClipAsset, getClipFileAsset, getClipThumbnail, getSourceAppIcon, hidePanel, listClips, openClip, openClipFile, pasteClip, toggleClipPreview } from "$lib/clips/api";
  import type { ClipDetail, ClipPage, ClipSummary } from "$lib/clips/types";
  import { cacheSet, canExpand, clipPreview, errorMessage, exactTime, fileName, filePaths, groupedFiles, metadataFacts, pasteFallbackMessage } from "$lib/clips/view";
  import { quitApp } from "$lib/settings/api";
  import { effectiveLocale, formatNumber, t } from "$lib/i18n/index.svelte";
  import SettingsView from "$lib/settings/SettingsView.svelte";
  import { ArrowLeft, ChevronLeft, ChevronRight, File, Image, Search } from "@lucide/svelte";

  let page = $state<ClipPage>({ items: [], page: 1, page_size: 10, total: 0, total_pages: 0 });
  let selectedId = $state<string | null>(null);
  let detail = $state<ClipDetail | null>(null);
  let assetUrl = $state<string | null>(null);
  let sourceIconUrl = $state<string | null>(null);
  let thumbnailUrls = $state<Record<string, string>>({});
  let fileThumbnailUrls = $state<Array<string | null>>([]);
  let fileIndex = $state(0);
  let previewPending = $state(false);
  let expandedId = $state<string | null>(null);
  let query = $state("");
  let loading = $state(true);
  let error = $state("");
  let copied = $state("");
  let menuOpen = $state(false);
  let appMenuOpen = $state(false);
  let view = $state<"history" | "settings">("history");
  let deletePending = $state(false);
  let rowReorderMotion = $state(false);
  let reducedMotion = $state(false);
  let searchInput = $state<HTMLInputElement>();
  let listbox = $state<HTMLDivElement>();
  let menuButton = $state<HTMLButtonElement>();
  let appMenuButton = $state<HTMLButtonElement>();
  let menuWrap = $state<HTMLDivElement>();
  let appMenuWrap = $state<HTMLDivElement>();
  let cancelActionButton = $state<HTMLButtonElement>();
  let confirmActionButton = $state<HTMLButtonElement>();
  let requestVersion = 0;
  let refreshRequestVersion = 0;
  let thumbnailRequestVersion = 0;
  let pageNavigationPending = false;
  let assetTimer: number | undefined;
  let searchTimer: number | undefined;
  const detailCache = new Map<string, ClipDetail>();
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
    const unlistenClips = listen("clips_changed", () => refresh(page.page));
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

  async function refresh(targetPage = page.page, selectLatest = false) {
    const refreshVersion = ++refreshRequestVersion;
    loading = true;
    error = "";
    const thumbnailVersion = ++thumbnailRequestVersion;
    try {
      const nextPage = await listClips(query, targetPage);
      if (refreshVersion !== refreshRequestVersion) return;
      page = nextPage;
      thumbnailUrls = Object.fromEntries(page.items.flatMap((item) => {
        const thumbnail = thumbnailCache.get(item.id);
        return thumbnail ? [[item.id, thumbnail]] : [];
      }));
      void loadThumbnails(page.items, thumbnailVersion);
      const next = !selectLatest && page.items.some((item) => item.id === selectedId)
        ? selectedId : page.items[0]?.id ?? null;
      await select(next);
    } catch (reason) {
      if (refreshVersion === refreshRequestVersion) error = errorMessage(reason);
      if (refreshVersion === refreshRequestVersion) {
        page = { items: [], page: targetPage, page_size: 10, total: 0, total_pages: 0 };
        await select(null);
      }
    } finally {
      if (refreshVersion === refreshRequestVersion) loading = false;
    }
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
    const selectionChanged = selectedId !== id;
    selectedId = id;
    if (selectionChanged) expandedId = null;
    if (assetTimer !== undefined) window.clearTimeout(assetTimer);
    detail = null;
    assetUrl = null;
    fileThumbnailUrls = [];
    fileIndex = 0;
    sourceIconUrl = null;
    previewPending = id !== null;
    if (!id) return;
    try {
      const next = detailCache.get(id) ?? await getClip(id);
      cacheSet(detailCache, id, next);
      if (version === requestVersion) {
        detail = next;
        previewPending = false;
        if (next.source_app) {
          const cachedIcon = sourceIconCache.get(next.source_app.id);
          if (cachedIcon !== undefined) sourceIconUrl = cachedIcon;
          else getSourceAppIcon(next.source_app.id).then((icon) => {
            cacheSet(sourceIconCache, next.source_app!.id, icon.data_url);
            if (version === requestVersion) sourceIconUrl = icon.data_url;
          }).catch(() => cacheSet(sourceIconCache, next.source_app!.id, null));
        }
        if (next.content_type === "image") scheduleAsset(id, null, version);
        // Auto-selecting the first row must not touch its original file. Only a
        // user click/key selection or preview request opts into that read.
        if (next.content_type === "file" && readSelectedFile) scheduleAsset(id, 0, version);
      }
    } catch (reason) {
      if (version === requestVersion) error = errorMessage(reason);
      if (version === requestVersion) previewPending = false;
    }
  }

  async function pasteSelected(plainText = false) {
    if (!selectedId) return;
    if (plainText && detail?.plain_text == null) return;
    try {
      const outcome = await pasteClip(selectedId, plainText);
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
    if (!selectedId) return;
    if (plainText && detail?.plain_text == null) return;
    try {
      await copyClip(selectedId, plainText);
      copied = plainText ? t("history.copiedPlain") : t("history.copied");
      setTimeout(() => copied = "", 1800);
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function removeSelected() {
    if (!selectedId) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    const nextId = page.items[index + 1]?.id ?? page.items[index - 1]?.id ?? null;
    const targetPage = page.items.length === 1 && page.page > 1 ? page.page - 1 : page.page;
    try {
      rowReorderMotion = true;
      await deleteClip(selectedId);
      evictClip(selectedId);
      selectedId = nextId;
      await refresh(targetPage);
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
    if (!selectedId) return;
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
    if (!selectedId) return;
    try {
      const openedSystemPreview = await toggleClipPreview(selectedId, fileIndex);
      if (!openedSystemPreview) {
        if (detail?.content_type === "file") await openClipFile(selectedId, fileIndex);
        else await openClip(selectedId);
      }
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
    return document.activeElement === listbox;
  }

  function selectFromList(id: string) {
    listbox?.focus();
    const item = page.items.find((candidate) => candidate.id === id);
    if (selectedId === id && item && canExpand(item)) {
      if (detail?.content_type === "file") scheduleAsset(id, fileIndex, requestVersion);
      expandedId = expandedId === id ? null : id;
      return;
    }
    void select(id, true);
  }

  async function resetToLatest() {
    menuOpen = false;
    appMenuOpen = false;
    view = "history";
    query = "";
    clearContentCaches();
    await tick();
    listbox?.focus();
    await refresh(1, true);
    if (document.activeElement === document.body || document.activeElement === listbox) {
      requestAnimationFrame(() => listbox?.focus());
    }
  }

  function suppressContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function onListKeydown(event: KeyboardEvent) {
    const index = page.items.findIndex((item) => item.id === selectedId);
    const selectIndex = (next: number) => void select(page.items[Math.max(0, Math.min(next, page.items.length - 1))]?.id ?? null, true);
    const selected = page.items.find((item) => item.id === selectedId);
    if ((event.metaKey || event.ctrlKey) && selected && canExpand(selected) && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
      event.preventDefault();
      void selectFile(fileIndex + (event.key === "ArrowLeft" ? -1 : 1));
    }
    else if (event.key === "ArrowDown") { event.preventDefault(); void moveSelection(1); }
    else if (event.key === "ArrowUp") { event.preventDefault(); void moveSelection(-1); }
    else if (event.key === "Home") { event.preventDefault(); selectIndex(0); }
    else if (event.key === "End") { event.preventDefault(); selectIndex(page.items.length - 1); }
    else if (event.key === "PageDown" && page.page < page.total_pages) { event.preventDefault(); void refresh(page.page + 1); }
    else if (event.key === "PageUp" && page.page > 1) { event.preventDefault(); void refresh(page.page - 1); }
    else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (expandedId === selectedId) expandedId = null;
      else if (page.page > 1) void refresh(page.page - 1);
    }
    else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (page.page < page.total_pages) void refresh(page.page + 1);
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
      if (page.items[target]) { event.preventDefault(); void select(page.items[target].id, true); }
    }
  }

  async function moveSelection(direction: -1 | 1) {
    if (pageNavigationPending || page.items.length === 0) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    const nextIndex = index + direction;
    if (nextIndex >= 0 && nextIndex < page.items.length) {
      await select(page.items[nextIndex]?.id ?? null, true);
      return;
    }
    if (direction < 0 && page.page > 1) {
      pageNavigationPending = true;
      try {
        await refresh(page.page - 1);
        await select(page.items.at(-1)?.id ?? null, true);
      } finally {
        pageNavigationPending = false;
      }
    } else if (direction > 0 && page.page < page.total_pages) {
      pageNavigationPending = true;
      try {
        await refresh(page.page + 1);
        await select(page.items[0]?.id ?? null, true);
      } finally {
        pageNavigationPending = false;
      }
    }
  }

  function onFileNavigatorKeydown(event: KeyboardEvent) {
    if (!detail || detail.content_type !== "file") return;
    const lastIndex = filePaths(detail).length - 1;
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
      event.preventDefault(); searchInput?.focus(); return;
    }
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "c") {
      event.preventDefault(); void copyOnly(true); return;
    }
    if (event.key === "/" && listHasFocus()) {
      event.preventDefault(); searchInput?.focus(); return;
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
    if (!selectedId || !detail || detail.content_type !== "file") return;
    const paths = filePaths(detail);
    if (index < 0 || index >= paths.length || index === fileIndex) return;
    fileIndex = index;
    assetUrl = null;
    scheduleAsset(selectedId, index, requestVersion);
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
    if (!detail || asset.byte_size === null) return;
    const sizes = [...(detail.metadata.file_sizes ?? [])];
    sizes[index] = asset.byte_size;
    detail.metadata.file_sizes = sizes;
  }

  function evictClip(id: string) {
    detailCache.delete(id);
    thumbnailCache.delete(id);
    for (const key of assetCache.keys()) if (key.startsWith(`${id}:`)) assetCache.delete(key);
  }

  function clearContentCaches() {
    detailCache.clear();
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
  <section class="left">
    <form class="search" onsubmit={(e) => { e.preventDefault(); onSearch(); }}>
      <span aria-hidden="true"><Search size={15} /></span>
      <input bind:this={searchInput} bind:value={query} oninput={onSearch} aria-label={t("history.searchLabel")} placeholder={t("history.searchPlaceholder")} />
      <kbd>/</kbd>
    </form>
    <div bind:this={listbox} class:full={page.items.length === page.page_size} class="list" role="listbox" aria-label={t("history.list")} aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined} onkeydown={onListKeydown}>
      {#if loading && page.items.length === 0}
        <div class="empty">{t("history.loading")}</div>
      {:else if error && page.items.length === 0}
        <button class="empty retry" onclick={() => refresh(1)}>{t("history.retry")}</button>
      {:else if page.items.length === 0}
        <div class="empty">{query ? t("history.noMatches") : t("history.empty")}</div>
      {:else}
        {#each page.items as item, index (item.id)}
          <div class:expanded={canExpand(item) && expandedId === item.id} class="clip-item" animate:flip={{ duration: reducedMotion || !rowReorderMotion ? 0 : 180, easing: cubicOut }} out:fade={{ duration: reducedMotion || !rowReorderMotion ? 0 : 90 }}>
            <div id={`clip-${item.id}`} class:selected={item.id === selectedId} class="row" role="option" tabindex="-1" aria-selected={item.id === selectedId} aria-posinset={index + 1} aria-setsize={page.items.length} ondblclick={() => pasteSelected()} onclick={() => selectFromList(item.id)} onkeydown={onListKeydown}>
              <span class="num">{formatNumber(index === 9 ? 0 : index + 1)}</span>
              <span class:swatch={item.content_type === "color"} class:media={item.content_type === "image" || item.content_type === "file"} class:file={item.content_type === "file"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>
                {#if thumbnailUrls[item.id]}<img src={thumbnailUrls[item.id]} alt="" />
                {:else if item.content_type === "image"}<span aria-hidden="true"><Image size={16} /></span>
                {:else if item.content_type === "file"}<File size={16} aria-hidden="true" />{/if}
              </span>
              <span class="snippet">{clipPreview(item)}</span>
              {#if canExpand(item)}<span class="disclosure" aria-hidden="true"><ChevronRight size={16} /></span>{/if}
            </div>
            {#if canExpand(item) && expandedId === item.id}
              <div class="row-details" role="group" in:fade={{ duration: reducedMotion ? 0 : 120 }} out:fade={{ duration: reducedMotion ? 0 : 90 }}>
                {#each groupedFiles(item) as path, filePosition}
                  <button tabindex="-1" class:selected={filePosition === fileIndex} class="row-child" onclick={(event) => { event.stopPropagation(); void selectFile(filePosition); }}>{fileName(path)}</button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class:pending={previewPending} class:file-preview={detail?.content_type === "file"} class="preview" aria-live="polite" aria-busy={previewPending}>
    {#if detail}
      <div class:text-preview={!['color', 'file', 'image'].includes(detail.content_type)} class="preview-body">
        {#if detail.content_type === "color"}
          <div class="color-preview"><span style:background={detail.preview}></span><code>{detail.preview}</code></div>
        {:else if detail.content_type === "file"}
          {#if assetUrl}<img class="asset" src={assetUrl} alt={t("history.fileThumbnail")} />
          {:else}<div class="file-preview-placeholder">{t("history.noPreview")}</div>{/if}
        {:else if detail.content_type === "image"}
          {#if assetUrl}<div class="asset-frame"><img class="asset" src={assetUrl} alt={t("history.imagePreview")} /></div>
          {:else}<div class="image-placeholder">{t("history.image")} · {typeof detail.metadata.width === "number" ? formatNumber(detail.metadata.width) : "?"}×{typeof detail.metadata.height === "number" ? formatNumber(detail.metadata.height) : "?"}</div>{/if}
        {:else}
          <pre>{detail.plain_text ?? detail.preview}</pre>
        {/if}
      </div>
      {#if detail.content_type === "file" && filePaths(detail).length > 1}
        <nav class="file-nav" aria-label={t("history.fileNavigation")}>
          <button tabindex="-1" class="file-nav-arrow" aria-label={t("history.previousFile", { shortcut: previousFileShortcut })} title={t("history.previousFileTitle", { shortcut: previousFileShortcut })} disabled={fileIndex === 0} onclick={() => void selectFile(fileIndex - 1)}><kbd>{previousFileShortcut}</kbd></button>
          <div class="file-strip" role="tablist" aria-label={t("history.fileCount", { count: formatNumber(filePaths(detail).length) })}>
            {#each filePaths(detail) as path, index}
              <button data-file-index={index} tabindex={index === fileIndex ? 0 : -1} role="tab" class:selected={index === fileIndex} class="file-thumb" aria-selected={index === fileIndex} aria-label={t("history.viewFile", { index: formatNumber(index + 1), name: fileName(path) })} title={fileName(path)} onclick={() => void selectFile(index)} onkeydown={onFileNavigatorKeydown}>
                {#if fileThumbnailUrls[index]}<img src={fileThumbnailUrls[index] ?? undefined} alt="" />
                {:else}<File size={16} aria-hidden="true" />{/if}
              </button>
            {/each}
          </div>
          <button tabindex="-1" class="file-nav-arrow" aria-label={t("history.nextFile", { shortcut: nextFileShortcut })} title={t("history.nextFileTitle", { shortcut: nextFileShortcut })} disabled={fileIndex === filePaths(detail).length - 1} onclick={() => void selectFile(fileIndex + 1)}><kbd>{nextFileShortcut}</kbd></button>
          <span class="file-nav-count" aria-live="polite">{formatNumber(fileIndex + 1)}/{formatNumber(filePaths(detail).length)}</span>
        </nav>
      {/if}
      <div class="preview-meta">
        {#if detail.content_type === "file"}
          <div class="meta-file">
            <span title={filePaths(detail)[fileIndex] ?? detail.preview}>{fileName(filePaths(detail)[fileIndex] ?? detail.preview)}</span>
            {#if filePaths(detail)[fileIndex]}<code title={filePaths(detail)[fileIndex]}>{filePaths(detail)[fileIndex]}</code>{/if}
          </div>
        {/if}
        <div class="meta-summary">
          <div class="meta-source">
            {#if detail.source_app}
              {#if sourceIconUrl}
                <img class="app-icon" src={sourceIconUrl} alt="" />
              {:else}
                <span class="app-fallback" aria-hidden="true">{detail.source_app.name.slice(0, 1)}</span>
              {/if}
              <div class="source-details"><span>{detail.source_app.name}</span><time>{exactTime(detail.created_at)}</time></div>
            {:else}
              <div class="source-details"><time>{exactTime(detail.created_at)}</time></div>
            {/if}
          </div>
          <dl class="meta-facts">
            {#each metadataFacts(detail, fileIndex) as fact}
              <div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>
            {/each}
          </dl>
        </div>
      </div>
    {:else if selectedId}
      <div class="preview-loading"><span>{t("history.previewLoading")}</span><pre>{page.items.find((item) => item.id === selectedId) ? clipPreview(page.items.find((item) => item.id === selectedId)!) : ""}</pre></div>
    {:else}
      <div class="empty">{t("history.select")}</div>
    {/if}
  </section>

  <footer class="pager">
    <button disabled={page.page <= 1} onclick={() => refresh(page.page - 1)} aria-label={t("history.previousPage")}><ChevronLeft size={16} aria-hidden="true" /></button>
    <span>{formatNumber(page.total_pages === 0 ? 0 : page.page)}/{formatNumber(page.total_pages)}</span>
    <button disabled={page.page >= page.total_pages} onclick={() => refresh(page.page + 1)} aria-label={t("history.nextPage")}><ChevronRight size={16} aria-hidden="true" /></button>
  </footer>
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
            <button data-menu-item role="menuitem" onclick={() => void viewSelectedClip()} disabled={!selectedId}><span>{t("history.viewSelected")}</span><kbd>Space</kbd></button>
            <div class="menu-separator" role="separator"></div>
            <button data-menu-item role="menuitem" onclick={() => void pastePlainSelected()} disabled={detail?.plain_text == null}><span>{t("history.pastePlain")}</span><kbd>⇧⏎</kbd></button>
            <button data-menu-item role="menuitem" onclick={() => void copyOnly()} disabled={!selectedId}><span>{t("history.copy")}</span></button>
            <button data-menu-item role="menuitem" onclick={() => void copyOnly(true)} disabled={detail?.plain_text == null}><span>{t("history.copyPlain")}</span><kbd>{isMac ? "⌘⇧C" : "Ctrl⇧C"}</kbd></button>
            <div class="menu-separator" role="separator"></div>
            <button data-menu-item role="menuitem" class="danger" onclick={() => void requestDelete()} disabled={!selectedId}><span>{t("history.deleteFrom")}</span><kbd>{deleteShortcut}</kbd></button>
          </div>
        {/if}
      </div>
      <button class="copy" onclick={() => pasteSelected()} disabled={!selectedId}><kbd>⏎</kbd> {t("history.paste")}</button>
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
  .left { grid-column:1; grid-row:2; min-height:0; display:flex; flex-direction:column; border-right:1px solid var(--hairline); }
  .search { height:42px; flex:none; display:flex; align-items:center; gap:8px; padding:0 14px; color:var(--text-3); border-bottom:1px solid var(--hairline); }
  .search input { min-width:0; flex:1; border:0; outline:0; padding:0; color:var(--text-1); background:transparent; font-size:13px; }
  .search input::placeholder { color:var(--text-2); }
  kbd { font:10px/1.4 var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:4px; padding:1px 5px; white-space:nowrap; }
  .list { flex:1; min-height:0; display:flex; flex-direction:column; gap:1px; padding:6px; overflow-y:auto; }
  .list:focus-visible { outline:none; }
  .clip-item { width:100%; }
  .list.full .clip-item:not(.expanded) { flex:1 0 44px; }
  .list.full .clip-item:not(.expanded) .row { height:100%; }
  .row { width:100%; min-height:44px; display:flex; align-items:center; gap:8px; padding:7px 8px; border-radius:8px; color:var(--text-1); background:transparent; text-align:left; cursor:default; }
  .row:hover { background:var(--bg-hover); }
  .list:focus .row.selected { background:var(--bg-selected); }
  .num { width:16px; flex:none; color:var(--text-3); font-family:-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif; font-size:12px; font-weight:650; line-height:1; font-variant-numeric:tabular-nums lining-nums; font-feature-settings:"tnum" 1, "lnum" 1, "zero" 0; letter-spacing:-.01em; text-align:center; }
  .list:focus .row.selected .num { color:var(--text-2); }
  .lead { width:28px; height:28px; flex:none; display:flex; align-items:center; justify-content:center; border-radius:4px; color:var(--text-2); font:7px var(--mono); }
  .lead.swatch { color:transparent; border:1px solid var(--hairline); }
  .lead.media { overflow:hidden; background:var(--bg-raised); font:15px/1 system-ui; }
  .lead.media img { width:100%; height:100%; object-fit:cover; }
  .snippet { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font:13px/1.5 var(--mono); }
  .disclosure { width:16px; flex:none; display:flex; align-items:center; justify-content:center; color:var(--text-3); transform:rotate(0deg); transition:transform 160ms cubic-bezier(.16, 1, .3, 1), color 120ms ease-out; }
  .clip-item.expanded .disclosure { color:var(--text-2); transform:rotate(90deg); }
  .row-details { margin:0 8px 4px 50px; padding:3px 8px 7px; }
  .row-child { width:100%; display:block; overflow:hidden; padding:4px 6px; border-radius:4px; color:var(--text-2); background:transparent; font:11px/1.45 var(--mono); text-align:left; text-overflow:ellipsis; white-space:nowrap; }
  .row-child:hover, .row-child.selected { background:var(--bg-hover); color:var(--text-1); }
  .row-child:focus-visible { outline:2px solid var(--text-2); outline-offset:1px; }
  .preview { grid-column:2; grid-row:2; min-width:0; min-height:0; display:flex; flex-direction:column; }
  .preview.pending { contain:content; }
  .preview-body { flex:1; min-height:0; overflow:hidden; display:flex; align-items:center; justify-content:center; padding:20px; }
  .preview-body.text-preview { align-items:flex-start; justify-content:flex-start; }
  .preview-body.text-preview pre { width:100%; }
  pre { max-width:100%; max-height:100%; margin:0; overflow:hidden; color:var(--text-1); font:13px/1.65 var(--mono); white-space:pre-wrap; overflow-wrap:anywhere; }
  .preview-meta { height:64px; flex:none; display:flex; align-items:center; padding:8px 20px; border-top:1px solid var(--hairline); }
  .preview.file-preview .preview-meta { height:96px; display:grid; grid-template-rows:minmax(0, 1fr) auto; gap:7px; padding-block:10px; }
  .meta-summary { min-width:0; width:100%; display:flex; align-items:center; justify-content:space-between; gap:20px; }
  .meta-source { min-width:0; display:flex; align-items:center; gap:8px; }
  .source-details { min-width:0; display:flex; flex-direction:column; gap:2px; color:var(--text-2); font:12px/1.2 var(--mono); }
  .source-details span { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .source-details time { color:var(--text-3); font-size:10px; }
  .meta-file { min-width:0; width:100%; display:flex; flex-direction:column; gap:3px; color:var(--text-2); font:11px/1.3 var(--mono); }
  .meta-file > span, .meta-file code { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .meta-file > span { color:var(--text-1); }
  .meta-file code { color:var(--text-3); font:10px/1.3 var(--mono); }
  .meta-facts { display:flex; align-items:center; justify-content:flex-end; gap:16px; margin:0; }
  .meta-facts div { display:flex; flex-direction:column; align-items:flex-end; gap:2px; white-space:nowrap; }
  .meta-facts dt { color:var(--text-3); font:10px/1 var(--mono); }
  .meta-facts dd { margin:0; color:var(--text-2); font:11px/1.2 var(--mono); }
  .app-icon, .app-fallback { width:22px; height:22px; flex:none; border-radius:4px; }
  .app-icon { object-fit:contain; }
  .app-fallback { display:grid; place-items:center; color:var(--bg-shell); background:var(--text-2); font:600 11px var(--mono); }
  .color-preview { display:flex; align-items:center; gap:14px; }
  .color-preview span { width:72px; height:72px; border:1px solid var(--hairline); border-radius:8px; }
  .color-preview code { color:var(--text-2); font:12px/1.6 var(--mono); white-space:pre-wrap; }
  .image-placeholder { min-height:180px; display:grid; place-items:center; color:var(--text-3); border:1px solid var(--hairline); border-radius:8px; }
  .asset-frame { width:100%; height:100%; min-height:180px; display:flex; align-items:center; justify-content:center; }
  .asset { display:block; max-width:100%; max-height:100%; border-radius:8px; object-fit:contain; }
  .file-preview-placeholder { color:var(--text-3); font-size:12px; }
  .file-nav { height:58px; flex:none; display:flex; align-items:center; gap:8px; padding:6px 20px; border-top:1px solid var(--hairline); }
  .file-strip { min-width:0; flex:1; display:flex; align-items:center; gap:6px; overflow-x:auto; }
  .file-thumb { width:38px; height:38px; flex:none; display:grid; place-items:center; padding:3px; border-radius:6px; background:transparent; }
  .file-thumb:hover, .file-thumb.selected { background:var(--bg-selected); }
  .file-thumb:focus-visible { outline:0; background:var(--bg-hover); box-shadow:inset 0 0 0 1px var(--text-2); }
  .file-thumb img { width:100%; height:100%; border-radius:4px; object-fit:cover; }
  .file-nav-arrow { min-width:38px; height:28px; flex:none; display:grid; place-items:center; padding:0 3px; border-radius:4px; color:var(--text-2); background:transparent; }
  .file-nav-arrow kbd { color:inherit; border-color:currentColor; }
  .file-nav-arrow:hover:not(:disabled) { background:var(--bg-hover); }
  .file-nav-count { flex:none; color:var(--text-3); font:10px var(--mono); }
  .empty { flex:1; display:grid; place-items:center; padding:24px; color:var(--text-3); font-size:13px; text-align:center; background:transparent; }
  .preview-loading { flex:1; min-height:0; padding:20px; color:var(--text-2); }
  .preview-loading span { display:block; margin-bottom:8px; color:var(--text-3); font-size:11px; }
  .preview-loading pre { color:var(--text-2); }
  .retry { width:100%; cursor:pointer; }
  .pager { grid-column:1; grid-row:3; display:grid; grid-template-columns:36px minmax(32px, auto) 36px; align-items:center; justify-content:start; gap:8px; padding:0 14px; border-top:1px solid var(--hairline); border-right:1px solid var(--hairline); color:var(--text-2); font:12px var(--mono); }
  .pager span { min-width:32px; text-align:center; font-variant-numeric:tabular-nums; }
  .pager button { width:36px; height:30px; display:grid; place-items:center; padding:0; border:1px solid var(--hairline); border-radius:4px; color:var(--text-2); background:transparent; font-size:14px; line-height:1; transition:background 100ms ease-out, color 100ms ease-out; }
  .pager button:hover:not(:disabled) { color:var(--text-1); background:var(--bg-hover); }
  .pager button:active:not(:disabled) { background:var(--bg-selected); }
  .pager button:disabled { opacity:.35; }
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
  @media (prefers-reduced-motion:reduce) { .disclosure { transition:none; } }
</style>
