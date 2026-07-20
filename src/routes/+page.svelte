<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { clearHistory, copyClip, deleteClip, getClip, getClipAsset, getClipFileAsset, getClipThumbnail, getSourceAppIcon, hidePanel, listClips, openClip, openClipFile, pasteClip, toggleClipPreview, type PasteOutcome } from "$lib/clips/api";
  import type { AppError, ClipDetail, ClipPage, ClipSummary } from "$lib/clips/types";
  import { applyTheme, getSettings, ignoreSource, quitApp, updateSettings, type Settings } from "$lib/settings/api";
  import { cachedUpdate, checkForUpdate, currentVersion, downloadAndInstall, openLatestRelease, type AvailableUpdate } from "$lib/updater/api";
  import { openUrl } from "@tauri-apps/plugin-opener";
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
  let settings = $state<Settings | null>(null);
  let settingsStatus = $state("");
  let activeSettingsTab = $state<"general" | "updates" | "about">("general");
  let appVersion = $state("…");
  let update = $state<AvailableUpdate | null>(null);
  let updateStatus = $state<"idle" | "checking" | "current" | "downloading" | "installing" | "error">("idle");
  let updateMessage = $state("");
  let updateProgress = $state<number | null>(null);
  let pendingAction = $state<"delete" | "clear" | null>(null);
  let rowReorderMotion = $state(false);
  let reducedMotion = $state(false);
  let searchInput = $state<HTMLInputElement>();
  let listbox = $state<HTMLDivElement>();
  let menuButton = $state<HTMLButtonElement>();
  let appMenuButton = $state<HTMLButtonElement>();
  let settingsFirstControl = $state<HTMLInputElement>();
  let settingsTabEls = $state<HTMLButtonElement[]>([]);
  let cancelActionButton = $state<HTMLButtonElement>();
  let confirmActionButton = $state<HTMLButtonElement>();
  let requestVersion = 0;
  let refreshRequestVersion = 0;
  let thumbnailRequestVersion = 0;
  let pageNavigationPending = false;
  let assetTimer: number | undefined;
  const detailCache = new Map<string, ClipDetail>();
  const assetCache = new Map<string, string | null>();
  const thumbnailCache = new Map<string, string>();
  const sourceIconCache = new Map<string, string | null>();
  const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);
  const deleteShortcut = isMac ? "⌘⌫" : "Ctrl⌫";
  const settingsShortcut = isMac ? "⌘," : "Ctrl,";
  const previousFileShortcut = isMac ? "⌘←" : "Ctrl←";
  const nextFileShortcut = isMac ? "⌘→" : "Ctrl→";

  onMount(() => {
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateReducedMotion = () => { reducedMotion = motionQuery.matches; };
    updateReducedMotion();
    motionQuery.addEventListener("change", updateReducedMotion);
    const captureEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" || event.key === "Esc" || event.code === "Escape") {
        event.preventDefault();
        event.stopImmediatePropagation();
        if (pendingAction) {
          cancelPendingAction();
        } else if (menuOpen) {
          closeMenu();
        } else if (appMenuOpen) {
          closeAppMenu();
        } else if (view === "settings") {
          closeSettingsView();
        } else {
          void hidePanel();
        }
      }
    };
    document.addEventListener("keydown", captureEscape, true);
    getSettings().then((settings) => applyTheme(settings.theme)).catch(() => {});
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

  async function select(id: string | null) {
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
      detailCache.set(id, next);
      if (version === requestVersion) {
        detail = next;
        previewPending = false;
        if (next.source_app) {
          const cachedIcon = sourceIconCache.get(next.source_app.id);
          if (cachedIcon !== undefined) sourceIconUrl = cachedIcon;
          else getSourceAppIcon(next.source_app.id).then((icon) => {
            sourceIconCache.set(next.source_app!.id, icon.data_url);
            if (version === requestVersion) sourceIconUrl = icon.data_url;
          }).catch(() => sourceIconCache.set(next.source_app!.id, null));
        }
        if (next.content_type === "image") scheduleAsset(id, null, version);
      }
    } catch (reason) {
      if (version === requestVersion) error = errorMessage(reason);
      if (version === requestVersion) previewPending = false;
    }
  }

  async function pasteSelected() {
    if (!selectedId) return;
    try {
      const outcome = await pasteClip(selectedId);
      if (outcome !== "pasted") {
        copied = pasteFallbackMessage(outcome);
        setTimeout(() => copied = "", 3200);
      }
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function copyOnly() {
    if (!selectedId) return;
    try {
      await copyClip(selectedId);
      copied = "已复制，可手动粘贴";
      setTimeout(() => copied = "", 1800);
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  function pasteFallbackMessage(outcome: PasteOutcome) {
    if (outcome === "copied_permission_required") return "已复制；请允许辅助功能权限后自动粘贴";
    if (outcome === "copied_target_lost") return "已复制；原窗口已关闭，请手动粘贴";
    if (outcome === "copied_focus_failed") return "已复制；无法恢复原窗口，请手动粘贴";
    if (outcome === "copied_injection_failed") return "已复制；系统拦截了自动粘贴，请手动粘贴";
    if (outcome === "copied_already_in_progress") return "已复制；正在处理上一次粘贴";
    return "已复制；当前平台暂不支持自动粘贴";
  }

  async function removeSelected() {
    if (!selectedId) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    const nextId = page.items[index + 1]?.id ?? page.items[index - 1]?.id ?? null;
    try {
      rowReorderMotion = true;
      await deleteClip(selectedId);
      selectedId = nextId;
      await refresh(page.page);
      await tick();
      listbox?.focus();
    }
    catch (reason) { error = errorMessage(reason); }
    finally {
      requestAnimationFrame(() => rowReorderMotion = false);
    }
    menuOpen = false;
  }

  async function removeAll() {
    try {
      await clearHistory();
      await refresh(1);
      if (view === "settings") settingsStatus = "历史已清空";
      else { await tick(); listbox?.focus(); }
    }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function requestPendingAction(action: "delete" | "clear") {
    if ((action === "delete" && !selectedId) || (action === "clear" && page.total === 0)) return;
    menuOpen = false;
    pendingAction = action;
    await tick();
    confirmActionButton?.focus();
  }

  function cancelPendingAction() {
    pendingAction = null;
    requestAnimationFrame(() => view === "settings" ? settingsFirstControl?.focus() : listbox?.focus());
  }

  function confirmPendingAction() {
    const action = pendingAction;
    pendingAction = null;
    if (action === "delete") void removeSelected();
    if (action === "clear") void removeAll();
  }

  async function ignoreSelectedSource() {
    const source = detail?.source_app;
    if (!source) return;
    if (!confirm(`以后不再记录来自“${source.name}”的内容？`)) return;
    try { await ignoreSource(source.id); copied = `已忽略 ${source.name}`; }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function openSelectedClip() {
    if (!selectedId) return;
    try {
      if (detail?.content_type === "file") await openClipFile(selectedId, fileIndex);
      else await openClip(selectedId);
    }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function previewSelectedClip() {
    if (!selectedId) return;
    try {
      const openedSystemPreview = await toggleClipPreview(selectedId, fileIndex);
      if (!openedSystemPreview) {
        const selected = page.items.find((item) => item.id === selectedId);
        if (selected && canExpand(selected)) expandedId = expandedId === selectedId ? null : selectedId;
      }
    }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  function openActionLabel() {
    const type = page.items.find((item) => item.id === selectedId)?.content_type;
    if (type === "file") return "在默认应用中打开文件";
    if (type === "image") return "在默认应用中查看图片";
    if (type === "link") return "在默认浏览器打开链接";
    if (type === "color") return "在默认应用中查看色值";
    if (type === "code") return "在默认应用中查看代码";
    return "在默认应用中查看文本";
  }


  function onSearch() { void refresh(1); }

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

  async function openAppMenu() {
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

  async function openSettingsView() {
    appMenuOpen = false;
    settingsStatus = "";
    activeSettingsTab = "general";
    updateStatus = "idle";
    updateMessage = "";
    settings = null;
    appVersion = await currentVersion();
    update = cachedUpdate();
    view = "settings";
    try {
      settings = await getSettings();
      await tick();
      settingsFirstControl?.focus();
    } catch (reason) {
      settingsStatus = errorMessage(reason);
    }
  }

  function closeSettingsView() {
    view = "history";
    settingsStatus = "";
    requestAnimationFrame(() => listbox?.focus());
  }

  const settingsTabs = ["general", "updates", "about"] as const;

  function selectSettingsTab(tab: (typeof settingsTabs)[number]) {
    activeSettingsTab = tab;
  }

  function onSettingsTabKeydown(event: KeyboardEvent) {
    const index = settingsTabs.indexOf(activeSettingsTab);
    let next = index;
    if (event.key === "ArrowDown") next = (index + 1) % settingsTabs.length;
    else if (event.key === "ArrowUp") next = (index - 1 + settingsTabs.length) % settingsTabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = settingsTabs.length - 1;
    else return;
    event.preventDefault();
    activeSettingsTab = settingsTabs[next];
    requestAnimationFrame(() => settingsTabEls[next]?.focus());
  }

  async function checkUpdates() {
    updateStatus = "checking";
    updateMessage = "";
    try {
      const result = await checkForUpdate();
      if (result.kind === "available") {
        appVersion = result.update.currentVersion;
        update = result.update;
        updateStatus = "idle";
      } else if (result.kind === "current") {
        appVersion = result.currentVersion;
        update = null;
        updateStatus = "current";
        updateMessage = "当前已是最新版本";
      } else {
        appVersion = result.currentVersion;
        updateStatus = "error";
        updateMessage = "开发环境不执行自动更新";
      }
    } catch (reason) {
      updateStatus = "error";
      updateMessage = `检查失败：${errorMessage(reason)}`;
    }
  }

  async function installUpdate() {
    if (!update) return;
    updateStatus = "downloading";
    updateProgress = null;
    updateMessage = "正在下载更新…";
    try {
      await downloadAndInstall(update.version, (progress) => {
        updateProgress = progress;
        updateMessage = progress === null ? "正在下载更新…" : `正在下载更新 ${progress}%`;
      });
      updateStatus = "installing";
      updateMessage = "正在安装并重新启动…";
    } catch (reason) {
      updateStatus = "error";
      updateMessage = `安装失败：${errorMessage(reason)}`;
    }
  }

  async function saveSettings() {
    if (!settings) return;
    settingsStatus = "";
    try {
      settings = await updateSettings(settings);
      applyTheme(settings.theme);
      settingsStatus = "已保存";
    } catch (reason) {
      settingsStatus = errorMessage(reason);
    }
  }

  function removeIgnoredApp(appId: string) {
    if (!settings) return;
    settings.ignored_apps = settings.ignored_apps.filter((item) => item !== appId);
  }

  function appLabel(appId: string) {
    return appId.split(/[\\/]/).pop()?.replace(/\.exe$/i, "") || appId;
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
      expandedId = expandedId === id ? null : id;
      return;
    }
    void select(id);
  }

  async function resetToLatest() {
    menuOpen = false;
    appMenuOpen = false;
    view = "history";
    query = "";
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
    const selectIndex = (next: number) => void select(page.items[Math.max(0, Math.min(next, page.items.length - 1))]?.id ?? null);
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
      void previewSelectedClip();
    }
    else if (event.key === "Enter") { event.preventDefault(); void pasteSelected(); }
    else if ((event.metaKey || event.ctrlKey) && ["Backspace", "Delete"].includes(event.key)) {
      event.preventDefault();
      void requestPendingAction("delete");
    }
    else if (/^[0-9]$/.test(event.key)) {
      const target = event.key === "0" ? 9 : Number(event.key) - 1;
      if (page.items[target]) { event.preventDefault(); void select(page.items[target].id); }
    }
  }

  async function moveSelection(direction: -1 | 1) {
    if (pageNavigationPending || page.items.length === 0) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    const nextIndex = index + direction;
    if (nextIndex >= 0 && nextIndex < page.items.length) {
      await select(page.items[nextIndex]?.id ?? null);
      return;
    }
    if (direction < 0 && page.page > 1) {
      pageNavigationPending = true;
      try {
        await refresh(page.page - 1);
        await select(page.items.at(-1)?.id ?? null);
      } finally {
        pageNavigationPending = false;
      }
    } else if (direction > 0 && page.page < page.total_pages) {
      pageNavigationPending = true;
      try {
        await refresh(page.page + 1);
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

  function cycleTabFocus(event: KeyboardEvent) {
    if (event.key !== "Tab" || event.metaKey || event.ctrlKey || event.altKey) return false;
    const scope = pendingAction
      ? document.querySelector<HTMLElement>(".confirmation")
      : menuOpen
        ? document.querySelector<HTMLElement>(".action-menu")
        : appMenuOpen
          ? document.querySelector<HTMLElement>(".app-menu")
          : document.querySelector<HTMLElement>(".panel");
    if (!scope) return false;
    const elements = Array.from(scope.querySelectorAll<HTMLElement>(
      'button:not([disabled]):not([tabindex="-1"]), input:not([disabled]):not([tabindex="-1"]), select:not([disabled]):not([tabindex="-1"]), [tabindex]:not([tabindex="-1"])'
    )).filter((element) => element.getClientRects().length > 0);
    if (elements.length === 0) return false;
    event.preventDefault();
    const current = elements.indexOf(document.activeElement as HTMLElement);
    const next = event.shiftKey
      ? (current <= 0 ? elements.length - 1 : current - 1)
      : (current < 0 || current === elements.length - 1 ? 0 : current + 1);
    elements[next]?.focus();
    return true;
  }

  function onKeydown(event: KeyboardEvent) {
    if (cycleTabFocus(event)) return;
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "w") {
      event.preventDefault();
      pendingAction = null;
      menuOpen = false;
      appMenuOpen = false;
      void hidePanel();
      return;
    }
    if (pendingAction) return;
    if (view === "settings") {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        void saveSettings();
      }
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

  function errorMessage(reason: unknown) {
    if (typeof reason === "object" && reason && "message" in reason) return String((reason as AppError).message);
    return String(reason ?? "未知错误");
  }

  function exactTime(value: string) {
    const date = new Date(value);
    const pad = (number: number) => String(number).padStart(2, "0");
    const year = date.getFullYear() === new Date().getFullYear() ? "" : `${date.getFullYear()}-`;
    return `${year}${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0)} MB`;
  }

  function countValue(detail: ClipDetail, key: string) {
    const value = detail.metadata[key];
    return typeof value === "number" ? value : null;
  }

  function metadataFacts(detail: ClipDetail) {
    const facts: Array<{ label: string; value: string }> = [];
    if (detail.content_type === "image") {
      const width = countValue(detail, "width");
      const height = countValue(detail, "height");
      if (width && height) facts.push({ label: "尺寸", value: `${width} × ${height}` });
      facts.push({ label: "大小", value: formatBytes(detail.byte_size) });
      return facts;
    }
    if (detail.content_type === "file") {
      const files = filePaths(detail);
      facts.push({ label: "文件", value: `${fileIndex + 1}/${files.length || 1}` });
      const sizes = Array.isArray(detail.metadata.file_sizes)
        ? detail.metadata.file_sizes.filter((size): size is number => typeof size === "number") : [];
      const size = sizes[fileIndex];
      if (size !== undefined) facts.push({ label: "大小", value: formatBytes(size) });
      return facts;
    }
    const charCount = countValue(detail, "char_count") ?? detail.plain_text?.length ?? 0;
    if (charCount) facts.push({ label: "字符", value: charCount.toLocaleString() });
    facts.push({ label: "大小", value: formatBytes(detail.byte_size) });
    return facts;
  }

  function groupedFiles(item: ClipSummary) {
    return item.content_type === "file" && Array.isArray(item.metadata.files)
      ? item.metadata.files.filter((path): path is string => typeof path === "string")
      : [];
  }

  function canExpand(item: ClipSummary) {
    return groupedFiles(item).length > 1;
  }

  function filePaths(record: ClipDetail) {
    return Array.isArray(record.metadata.files)
      ? record.metadata.files.filter((path): path is string => typeof path === "string")
      : [];
  }

  function fileName(path: string) {
    const normalized = path.replace(/^file:\/\//, "");
    return normalized.split(/[\\/]/).pop() || normalized;
  }

  async function selectFile(index: number) {
    if (!selectedId || !detail || detail.content_type !== "file") return;
    const paths = filePaths(detail);
    if (index < 0 || index >= paths.length || index === fileIndex) return;
    fileIndex = index;
    const key = assetKey(selectedId, index);
    if (assetCache.has(key)) {
      assetUrl = assetCache.get(key) ?? null;
      return;
    }
    assetUrl = null;
    const version = requestVersion;
    try {
      const asset = await getClipFileAsset(selectedId, index);
      assetCache.set(key, asset.data_url);
      if (version === requestVersion) assetUrl = asset.data_url;
    } catch (reason) {
      if (version === requestVersion) error = errorMessage(reason);
    }
  }

  function assetKey(id: string, index: number | null) {
    return `${id}:${index ?? "image"}`;
  }

  function scheduleAsset(id: string, index: number | null, version: number) {
    const key = assetKey(id, index);
    if (assetCache.has(key)) {
      assetUrl = assetCache.get(key) ?? null;
      return;
    }
    assetTimer = window.setTimeout(() => {
      assetTimer = undefined;
      const request = index === null ? getClipAsset(id) : getClipFileAsset(id, index);
      request.then((asset) => {
        assetCache.set(key, asset.data_url);
        if (version === requestVersion) assetUrl = asset.data_url;
      }).catch((reason) => {
        if (version === requestVersion) error = errorMessage(reason);
      });
    }, 80);
  }
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={suppressContextMenu} />

<main class="panel" aria-label="ClipClop 剪贴板历史">
  <header class="titlebar">
    {#if view === "history"}
      <div class="brand">
        <div class="app-menu-wrap">
          <button bind:this={appMenuButton} class="app-menu-trigger" aria-label="ClipClop 应用菜单" aria-haspopup="menu" aria-expanded={appMenuOpen} onclick={() => void openAppMenu()}>ClipClop</button>
          {#if appMenuOpen}
            <div class="menu app-menu" role="menu" tabindex="-1" aria-label="ClipClop 应用菜单" onkeydown={onMenuKeydown}>
              <button data-menu-item role="menuitem" onclick={() => { appMenuOpen = false; void openSettingsView(); }}>设置… <kbd>{settingsShortcut}</kbd></button>
              <button data-menu-item role="menuitem" class="danger" onclick={() => void quitApp()}>退出 ClipClop</button>
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <button class="back" aria-label="返回历史记录" onclick={closeSettingsView}><ArrowLeft size={16} aria-hidden="true" /></button>
      <span class="settings-title">设置</span>
    {/if}
    <div class="titlebar-drag" data-tauri-drag-region></div>
  </header>
  {#if view === "history"}
  <section class="left">
    <form class="search" onsubmit={(e) => { e.preventDefault(); onSearch(); }}>
      <span aria-hidden="true"><Search size={15} /></span>
      <input bind:this={searchInput} bind:value={query} oninput={onSearch} aria-label="搜索剪贴板历史" placeholder="搜索剪贴板…" />
      <kbd>/</kbd>
    </form>
    <div bind:this={listbox} class="list" role="tree" aria-label="剪贴板历史" aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined} onkeydown={onListKeydown}>
      {#if loading && page.items.length === 0}
        <div class="empty">正在读取历史…</div>
      {:else if error && page.items.length === 0}
        <button class="empty retry" onclick={() => refresh(1)}>读取失败，点击重试</button>
      {:else if page.items.length === 0}
        <div class="empty">{query ? "没有匹配结果" : "复制一点内容，然后再回来听见哒哒声。"}</div>
      {:else}
        {#each page.items as item, index (item.id)}
          <div class:expanded={canExpand(item) && expandedId === item.id} class="clip-item" animate:flip={{ duration: reducedMotion || !rowReorderMotion ? 0 : 180, easing: cubicOut }} out:fade={{ duration: reducedMotion || !rowReorderMotion ? 0 : 90 }}>
            <div id={`clip-${item.id}`} class:selected={item.id === selectedId} class="row" role="treeitem" tabindex="-1" aria-selected={item.id === selectedId} aria-expanded={canExpand(item) ? expandedId === item.id : undefined} aria-posinset={index + 1} aria-setsize={page.items.length} ondblclick={() => pasteSelected()} onclick={() => selectFromList(item.id)} onkeydown={onListKeydown}>
              <span class="num">{index === 9 ? 0 : index + 1}</span>
              <span class:swatch={item.content_type === "color"} class:media={item.content_type === "image" || item.content_type === "file"} class:file={item.content_type === "file"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>
                {#if thumbnailUrls[item.id]}<img src={thumbnailUrls[item.id]} alt="" />
                {:else if item.content_type === "image"}<span aria-hidden="true"><Image size={16} /></span>
                {:else if item.content_type === "file"}<File size={16} aria-hidden="true" />{/if}
              </span>
              <span class="snippet">{item.preview}</span>
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
          {#if assetUrl}<img class="asset" src={assetUrl} alt="文件缩略图" />
          {:else}<div class="file-preview-placeholder">无可用预览</div>{/if}
        {:else if detail.content_type === "image"}
          {#if assetUrl}<div class="asset-frame"><img class="asset" src={assetUrl} alt="剪贴板图片预览" /></div>
          {:else}<div class="image-placeholder">图片 · {String(detail.metadata.width ?? "?")}×{String(detail.metadata.height ?? "?")}</div>{/if}
        {:else}
          <pre>{detail.plain_text ?? detail.preview}</pre>
        {/if}
      </div>
      {#if detail.content_type === "file" && filePaths(detail).length > 1}
        <nav class="file-nav" aria-label="已复制文件导航；使用左右方向键切换文件">
          <button tabindex="-1" class="file-nav-arrow" aria-label={`上一个文件，列表中快捷键${previousFileShortcut}，导航内使用左方向键`} title={`上一个文件（${previousFileShortcut}；导航内 ←）`} disabled={fileIndex === 0} onclick={() => void selectFile(fileIndex - 1)}><kbd>{previousFileShortcut}</kbd></button>
          <div class="file-strip" role="tablist" aria-label={`${filePaths(detail).length} 个已复制文件`}>
            {#each filePaths(detail) as path, index}
              <button data-file-index={index} tabindex={index === fileIndex ? 0 : -1} role="tab" class:selected={index === fileIndex} class="file-thumb" aria-selected={index === fileIndex} aria-label={`查看文件 ${index + 1}：${fileName(path)}`} title={fileName(path)} onclick={() => void selectFile(index)} onkeydown={onFileNavigatorKeydown}>
                {#if fileThumbnailUrls[index]}<img src={fileThumbnailUrls[index] ?? undefined} alt="" />
                {:else}<File size={16} aria-hidden="true" />{/if}
              </button>
            {/each}
          </div>
          <button tabindex="-1" class="file-nav-arrow" aria-label={`下一个文件，列表中快捷键${nextFileShortcut}，导航内使用右方向键`} title={`下一个文件（${nextFileShortcut}；导航内 →）`} disabled={fileIndex === filePaths(detail).length - 1} onclick={() => void selectFile(fileIndex + 1)}><kbd>{nextFileShortcut}</kbd></button>
          <span class="file-nav-count" aria-live="polite">{fileIndex + 1}/{filePaths(detail).length}</span>
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
            {#each metadataFacts(detail) as fact}
              <div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>
            {/each}
          </dl>
        </div>
      </div>
    {:else if selectedId}
      <div class="preview-loading"><span>正在读取</span><pre>{page.items.find((item) => item.id === selectedId)?.preview ?? ""}</pre></div>
    {:else}
      <div class="empty">选择一条记录查看内容</div>
    {/if}
  </section>

  <footer class="pager">
    <button disabled={page.page <= 1} onclick={() => refresh(page.page - 1)} aria-label="上一页"><ChevronLeft size={16} aria-hidden="true" /></button>
    <span>{page.total_pages === 0 ? 0 : page.page}/{page.total_pages}</span>
    <button disabled={page.page >= page.total_pages} onclick={() => refresh(page.page + 1)} aria-label="下一页"><ChevronRight size={16} aria-hidden="true" /></button>
  </footer>
  <footer class="actions">
    {#if pendingAction}
      <div class="confirmation" role="alertdialog" aria-label={pendingAction === "delete" ? "确认删除记录" : "确认清空历史"}>
        <span>{pendingAction === "delete" ? "删除此记录？" : "清空全部历史？"}<small>仅从 ClipClop 移除，不影响原始文件或系统剪贴板。</small></span>
        <button bind:this={cancelActionButton} class="ghost" onclick={cancelPendingAction}>取消 <kbd>Esc</kbd></button>
        <button bind:this={confirmActionButton} class="destructive" onclick={confirmPendingAction}>{pendingAction === "delete" ? "删除" : "清空"}</button>
      </div>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if copied}<span class="message">{copied}</span>{/if}
      <div class="menu-wrap">
        <button bind:this={menuButton} class:expanded={menuOpen} class="ghost action-menu-trigger" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => void openMenu()}><kbd>⌘K</kbd> 操作</button>
        {#if menuOpen}
          <div class="menu action-menu" role="menu" tabindex="-1" aria-label="操作菜单" onkeydown={onMenuKeydown}>
            <button data-menu-item role="menuitem" onclick={() => void previewSelectedClip()} disabled={!selectedId}><span>快速预览</span><kbd>Space</kbd></button>
            <button data-menu-item role="menuitem" onclick={() => void openSelectedClip()} disabled={!selectedId}><span>{openActionLabel()}</span></button>
            <button data-menu-item role="menuitem" onclick={() => void copyOnly()} disabled={!selectedId}><span>仅复制到剪贴板</span></button>
            <button data-menu-item role="menuitem" onclick={ignoreSelectedSource} disabled={!detail?.source_app}><span>忽略此来源应用</span></button>
            <button data-menu-item role="menuitem" class="danger" onclick={() => void requestPendingAction("delete")} disabled={!selectedId}><span>从 ClipClop 删除</span><kbd>{deleteShortcut}</kbd></button>
          </div>
        {/if}
      </div>
      <button class="copy" onclick={() => pasteSelected()} disabled={!selectedId}><kbd>⏎</kbd> 粘贴</button>
    {/if}
  </footer>
  {:else}
    <div class="settings-body">
      <div class="settings-nav" role="tablist" aria-orientation="vertical" aria-label="设置分类">
        <button bind:this={settingsTabEls[0]} role="tab" id="settings-tab-general" aria-controls="settings-panel" aria-selected={activeSettingsTab === "general"} tabindex={activeSettingsTab === "general" ? 0 : -1} class:active={activeSettingsTab === "general"} onclick={() => selectSettingsTab("general")} onkeydown={onSettingsTabKeydown}>常规</button>
        <button bind:this={settingsTabEls[1]} role="tab" id="settings-tab-updates" aria-controls="settings-panel" aria-selected={activeSettingsTab === "updates"} tabindex={activeSettingsTab === "updates" ? 0 : -1} class:active={activeSettingsTab === "updates"} onclick={() => selectSettingsTab("updates")} onkeydown={onSettingsTabKeydown}>软件更新</button>
        <button bind:this={settingsTabEls[2]} role="tab" id="settings-tab-about" aria-controls="settings-panel" aria-selected={activeSettingsTab === "about"} tabindex={activeSettingsTab === "about" ? 0 : -1} class:active={activeSettingsTab === "about"} onclick={() => selectSettingsTab("about")} onkeydown={onSettingsTabKeydown}>关于</button>
      </div>
      <div id="settings-panel" class="settings-content" role="tabpanel" aria-labelledby={`settings-tab-${activeSettingsTab}`} tabindex="0">
        {#if settings}
          {#if activeSettingsTab === "general"}
            <label><span><strong>开机启动</strong><small>登录系统后在后台启动 ClipClop。</small></span><input bind:this={settingsFirstControl} type="checkbox" bind:checked={settings.launch_at_login} /></label>
            <label><span><strong>保留期限</strong><small>超出期限的历史会在后续捕获时清理。</small></span><select bind:value={settings.retention_days}><option value={7}>7 天</option><option value={30}>30 天</option><option value={90}>90 天</option></select></label>
            <label><span><strong>外观</strong><small>跟随系统，或固定使用 Light/Dark。</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">跟随系统</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
            <div class="preference-row"><span><strong>全局快捷键</strong><small>当前版本使用平台默认值；暂不支持自定义。</small></span><kbd>{settings.hotkey}</kbd></div>
            <div class="preference-row"><span><strong>数据管理</strong><small>清除 ClipClop 保存的全部历史，不影响原始文件或系统剪贴板。</small></span><button class="danger-action" onclick={() => void requestPendingAction("clear")} disabled={page.total === 0}>清空全部历史</button></div>
            {#if settings.ignored_apps.length > 0}
              <div class="ignored-apps"><strong>已忽略的应用</strong>{#each settings.ignored_apps as appId}<div><code title={appId}>{appLabel(appId)}</code><button onclick={() => removeIgnoredApp(appId)}>移除</button></div>{/each}</div>
            {/if}
          {:else if activeSettingsTab === "updates"}
            <div class="update-head"><span><strong>保持 ClipClop 为最新版本</strong><small>当前版本 {appVersion}；最多每天自动检查一次。</small></span><label class="update-toggle"><span>自动检查</span><input type="checkbox" bind:checked={settings.check_updates} /></label></div>
            {#if update}
              <div class="update-card">
                <div class="update-card-head"><strong>ClipClop {update.version} 可用</strong>{#if update.date}<small>{new Date(update.date).toLocaleDateString()}</small>{/if}</div>
                {#if update.notes}<p>{update.notes}</p>{/if}
                {#if updateStatus === "downloading" && updateProgress !== null}<progress max="100" value={updateProgress}></progress>{/if}
                <div class="update-actions"><button class="ghost" onclick={() => void openLatestRelease()}>查看发布页</button><button class="copy" disabled={updateStatus === "downloading" || updateStatus === "installing"} onclick={installUpdate}>下载并安装</button></div>
              </div>
            {:else}
              <div class="update-check"><span class:error={updateStatus === "error"} aria-live="polite">{updateMessage}</span><button class="ghost" disabled={updateStatus === "checking"} onclick={checkUpdates}>{updateStatus === "checking" ? "正在检查…" : "检查更新"}</button></div>
            {/if}
            {#if update && updateMessage}<small class="update-note" class:error={updateStatus === "error"} aria-live="polite">{updateMessage}</small>{/if}
          {:else}
            <div class="about">
              <img class="about-mark" src="/app-icon.png" alt="ClipClop 图标" />
              <h2>ClipClop</h2>
              <p>轻量、离线优先的跨平台剪贴板历史工具。</p>
              <small>版本 {appVersion} · MIT License</small>
              <div class="about-links"><button class="github" aria-label="在 GitHub 查看 ClipClop 项目" title="GitHub" onclick={() => void openUrl("https://github.com/hiQianFan/ClipClop")}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .7a11.5 11.5 0 0 0-3.64 22.4c.58.1.79-.25.79-.56v-2.2c-3.23.7-3.91-1.37-3.91-1.37-.53-1.34-1.29-1.7-1.29-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.71 1.26 3.37.96.1-.75.4-1.26.73-1.55-2.58-.29-5.29-1.29-5.29-5.69 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.47.11-3.06 0 0 .97-.31 3.16 1.18A10.9 10.9 0 0 1 12 6.12c.98 0 1.95.13 2.86.38 2.2-1.49 3.16-1.18 3.16-1.18.63 1.59.23 2.77.11 3.06.74.81 1.19 1.84 1.19 3.1 0 4.42-2.72 5.39-5.3 5.68.42.36.79 1.07.79 2.16v3.21c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .7Z"/></svg></button></div>
            </div>
          {/if}
        {:else}
          <div class="settings-loading">{settingsStatus || "正在读取设置…"}</div>
        {/if}
      </div>
    </div>
    <footer class="settings-actions">
      {#if pendingAction === "clear"}
        <div class="confirmation" role="alertdialog" aria-label="确认清空历史">
          <span>清空全部历史？<small>仅从 ClipClop 移除，不影响原始文件或系统剪贴板。</small></span>
          <button bind:this={cancelActionButton} class="ghost" onclick={cancelPendingAction}>取消 <kbd>Esc</kbd></button>
          <button bind:this={confirmActionButton} class="destructive" onclick={confirmPendingAction}>清空</button>
        </div>
      {:else}
        <span aria-live="polite" class:error={settingsStatus !== "" && !["已保存", "历史已清空"].includes(settingsStatus)}>{settingsStatus}</span>
        <button class="ghost" onclick={closeSettingsView}>返回 <kbd>Esc</kbd></button>
        {#if activeSettingsTab !== "about"}<button class="copy" onclick={() => void saveSettings()} disabled={!settings}>保存</button>{/if}
      {/if}
    </footer>
  {/if}
</main>

<style>
  .panel { width:calc(100vw - 40px); height:calc(100vh - 40px); margin:20px; display:grid; grid-template-columns:300px 1fr; grid-template-rows:42px 1fr 48px; background:var(--bg-shell); border-radius:14px; box-shadow:var(--panel-shadow); overflow:hidden; }
  .titlebar { grid-column:1 / -1; grid-row:1; display:flex; align-items:center; padding:0 14px; border-bottom:1px solid var(--hairline); user-select:none; }
  .titlebar-drag { flex:1; align-self:stretch; }
  .brand { display:flex; align-items:center; color:var(--text-2); }
  .app-menu-wrap { position:relative; }
  .app-menu-trigger { height:24px; padding:0 4px; border-radius:5px; color:var(--text-2); background:transparent; font-size:12px; font-weight:600; letter-spacing:.01em; }
  .app-menu-trigger:hover { background:var(--bg-hover); }
  .back { width:24px; height:24px; padding:0; border-radius:5px; color:var(--text-2); background:transparent; font-size:16px; }
  .back:hover { background:var(--bg-hover); }
  .settings-title { margin-left:7px; color:var(--text-2); font-size:12px; font-weight:600; }
  .left { grid-column:1; grid-row:2; min-height:0; display:flex; flex-direction:column; border-right:1px solid var(--hairline); }
  .search { height:42px; flex:none; display:flex; align-items:center; gap:8px; padding:0 14px; color:var(--text-3); border-bottom:1px solid var(--hairline); }
  .search input { min-width:0; flex:1; border:0; outline:0; padding:0; color:var(--text-1); background:transparent; font-size:13px; }
  .search input::placeholder { color:var(--text-2); }
  kbd { font:10px/1.4 var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:4px; padding:1px 5px; white-space:nowrap; }
  .list { flex:1; min-height:0; display:flex; flex-direction:column; gap:1px; padding:6px; }
  .list:focus-visible { outline:none; }
  .clip-item { width:100%; }
  .row { width:100%; min-height:44px; display:flex; align-items:center; gap:8px; padding:7px 8px; border-radius:8px; color:var(--text-1); background:transparent; text-align:left; cursor:default; }
  .row:hover { background:var(--bg-hover); }
  .row.selected { background:var(--bg-selected); }
  .num { width:16px; flex:none; color:var(--text-3); font-family:-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif; font-size:12px; font-weight:650; line-height:1; font-variant-numeric:tabular-nums lining-nums; font-feature-settings:"tnum" 1, "lnum" 1, "zero" 0; letter-spacing:-.01em; text-align:center; }
  .row.selected .num { color:var(--text-2); }
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
  .menu .danger { color:var(--danger); }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:11px; }
  .message.error { color:var(--danger); }
  .confirmation { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  .confirmation > span { margin-right:auto; color:var(--text-1); font-size:12px; font-weight:600; }
  .confirmation small { display:block; margin-top:2px; color:var(--text-2); font-size:10px; font-weight:400; }
  .settings-body { grid-column:1 / -1; grid-row:2; min-height:0; display:grid; grid-template-columns:148px 1fr; }
  .settings-nav { min-height:0; overflow-y:auto; display:flex; flex-direction:column; gap:2px; padding:12px 10px; border-right:1px solid var(--hairline); }
  .settings-nav button { text-align:left; padding:8px 10px; border-radius:6px; color:var(--text-2); background:transparent; font-size:12px; font-weight:600; }
  .settings-nav button:hover { background:var(--bg-hover); color:var(--text-1); }
  .settings-nav button.active { color:var(--text-1); background:var(--bg-selected); }
  .settings-content { min-height:0; overflow-y:auto; padding:0 20px; }
  .settings-content:focus-visible { outline:none; }
  .settings-content > label, .preference-row { min-height:68px; display:flex; align-items:center; justify-content:space-between; gap:24px; border-bottom:1px solid var(--hairline); }
  .settings-content > label > span, .preference-row > span { display:flex; flex-direction:column; gap:3px; }
  .settings-content strong { color:var(--text-1); font-size:13px; font-weight:600; }
  .settings-content small { color:var(--text-3); font-size:11px; }
  .settings-content select { min-width:116px; padding:7px 28px 7px 9px; border:1px solid var(--hairline); border-radius:6px; color:var(--text-1); background:var(--bg-raised); }
  .settings-content input { width:18px; height:18px; accent-color:var(--text-1); }
  .update-head { display:flex; align-items:center; justify-content:space-between; gap:18px; padding:16px 0; border-bottom:1px solid var(--hairline); }
  .update-head > span { display:flex; flex-direction:column; gap:3px; }
  .update-toggle { display:flex; align-items:center; gap:8px; color:var(--text-2); font-size:12px; }
  .update-toggle span { display:block; }
  .update-card { display:flex; flex-direction:column; gap:10px; margin-top:16px; padding:14px; border-radius:8px; background:var(--bg-raised); }
  .update-card-head { display:flex; justify-content:space-between; gap:12px; }
  .update-card p { max-height:120px; overflow:auto; white-space:pre-wrap; color:var(--text-2); font-size:12px; line-height:1.5; }
  .update-card progress { width:100%; accent-color:var(--action); }
  .update-actions { display:flex; justify-content:flex-end; gap:8px; }
  .update-check { display:flex; align-items:center; justify-content:space-between; gap:12px; margin-top:16px; }
  .update-check span, .update-note { color:var(--text-2); font-size:12px; }
  .update-check span.error, .update-note.error { color:var(--danger); }
  .update-note { display:block; margin-top:10px; }
  .about { height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:8px; text-align:center; }
  .about-mark { width:56px; height:56px; margin-bottom:6px; border-radius:12px; object-fit:contain; }
  .about h2 { font-size:16px; }
  .about p { max-width:280px; color:var(--text-2); font-size:12px; line-height:1.5; }
  .about-links { display:flex; gap:8px; margin-top:12px; }
  .about-links .github { width:34px; height:34px; display:grid; place-items:center; padding:0; border-radius:8px; color:var(--text-2); background:var(--bg-hover); }
  .about-links .github:hover { color:var(--text-1); background:var(--bg-selected); }
  .about-links .github svg { width:18px; height:18px; fill:currentColor; }
  .danger-action { padding:7px 10px; border:1px solid color-mix(in srgb, var(--danger) 45%, transparent); border-radius:6px; color:var(--danger); background:transparent; }
  .danger-action:hover:not(:disabled) { background:color-mix(in srgb, var(--danger) 8%, transparent); }
  .ignored-apps { padding:16px 0; display:flex; flex-direction:column; gap:8px; }
  .ignored-apps > strong { margin-bottom:2px; }
  .ignored-apps div { display:flex; align-items:center; justify-content:space-between; gap:16px; padding:7px 9px; border-radius:6px; background:var(--bg-raised); }
  .ignored-apps code { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:var(--text-2); font:11px var(--mono); }
  .ignored-apps button { color:var(--text-2); background:transparent; font-size:12px; }
  .settings-loading { height:100%; display:grid; place-items:center; color:var(--text-3); font-size:12px; }
  .settings-actions { grid-column:1 / -1; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .settings-actions > span { margin-right:auto; color:var(--text-2); font-size:11px; }
  .settings-actions > span.error { color:var(--danger); }
  @media (min-width:840px) { .panel { grid-template-columns:320px 1fr; } }
  @media (max-width:680px) { .panel { grid-template-columns:280px 1fr; } }
  @media (prefers-reduced-motion:no-preference) { .panel { animation:enter 120ms ease-out; } @keyframes enter { from { opacity:0; transform:scale(.98); } } }
  @media (prefers-reduced-motion:reduce) { .disclosure { transition:none; } }
</style>
