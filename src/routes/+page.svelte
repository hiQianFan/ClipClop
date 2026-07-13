<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { clearHistory, copyClip, deleteClip, getClip, getClipAsset, getClipThumbnail, getSourceAppIcon, hidePanel, listClips, openClip } from "$lib/clips/api";
  import type { AppError, ClipDetail, ClipPage, ClipSummary } from "$lib/clips/types";
  import { applyTheme, getSettings, ignoreSource, quitApp, updateSettings, type Settings } from "$lib/settings/api";

  let page = $state<ClipPage>({ items: [], page: 1, page_size: 10, total: 0, total_pages: 0 });
  let selectedId = $state<string | null>(null);
  let detail = $state<ClipDetail | null>(null);
  let assetUrl = $state<string | null>(null);
  let sourceIconUrl = $state<string | null>(null);
  let thumbnailUrls = $state<Record<string, string>>({});
  let query = $state("");
  let loading = $state(true);
  let error = $state("");
  let copied = $state("");
  let menuOpen = $state(false);
  let appMenuOpen = $state(false);
  let view = $state<"history" | "settings">("history");
  let settings = $state<Settings | null>(null);
  let settingsStatus = $state("");
  let pendingAction = $state<"delete" | "clear" | null>(null);
  let reducedMotion = $state(false);
  let searchInput = $state<HTMLInputElement>();
  let listbox = $state<HTMLDivElement>();
  let menuButton = $state<HTMLButtonElement>();
  let appMenuButton = $state<HTMLButtonElement>();
  let settingsFirstControl = $state<HTMLInputElement>();
  let cancelActionButton = $state<HTMLButtonElement>();
  let confirmActionButton = $state<HTMLButtonElement>();
  let requestVersion = 0;
  let thumbnailRequestVersion = 0;
  const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);
  const deleteShortcut = isMac ? "⌘⌫" : "Ctrl⌫";
  const settingsShortcut = isMac ? "⌘," : "Ctrl,";

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
    const unlisten = listen("clips_changed", () => refresh(page.page));
    return () => {
      document.removeEventListener("keydown", captureEscape, true);
      motionQuery.removeEventListener("change", updateReducedMotion);
      unlisten.then((fn) => fn());
    };
  });

  async function refresh(targetPage = page.page) {
    loading = true;
    error = "";
    const thumbnailVersion = ++thumbnailRequestVersion;
    try {
      page = await listClips(query, targetPage);
      thumbnailUrls = {};
      void loadThumbnails(page.items, thumbnailVersion);
      const next = page.items.some((item) => item.id === selectedId)
        ? selectedId : page.items[0]?.id ?? null;
      await select(next);
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      loading = false;
    }
  }

  async function loadThumbnails(items: ClipSummary[], version: number) {
    const mediaItems = items.filter((item) => item.content_type === "image" || item.content_type === "file");
    const entries = await Promise.all(mediaItems.map(async (item) => {
      try {
        const thumbnail = await getClipThumbnail(item.id);
        return thumbnail.data_url ? [item.id, thumbnail.data_url] as const : null;
      } catch { return null; }
    }));
    if (version === thumbnailRequestVersion) {
      thumbnailUrls = Object.fromEntries(entries.filter((entry): entry is readonly [string, string] => entry !== null));
    }
  }

  async function select(id: string | null) {
    selectedId = id;
    detail = null;
    assetUrl = null;
    sourceIconUrl = null;
    if (!id) return;
    const version = ++requestVersion;
    try {
      const next = await getClip(id);
      if (version === requestVersion) {
        detail = next;
        if (next.source_app) {
          getSourceAppIcon(next.source_app.id).then((icon) => {
            if (version === requestVersion) sourceIconUrl = icon.data_url;
          }).catch(() => {});
        }
        if (next.content_type === "image" || next.content_type === "file") {
          const asset = await getClipAsset(id);
          if (version === requestVersion) assetUrl = asset.data_url;
        }
      }
    } catch (reason) {
      if (version === requestVersion) error = errorMessage(reason);
    }
  }

  async function copy(mode: "rich" | "plain") {
    if (!selectedId) return;
    try {
      await copyClip(selectedId, mode);
      copied = mode === "rich" ? "已复制" : "已复制为纯文本";
      setTimeout(() => copied = "", 1400);
      await getCurrentWindow().hide();
    } catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function removeSelected() {
    if (!selectedId) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    const nextId = page.items[index + 1]?.id ?? page.items[index - 1]?.id ?? null;
    try {
      await deleteClip(selectedId);
      selectedId = nextId;
      await refresh(page.page);
      await tick();
      listbox?.focus();
    }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function removeAll() {
    try { await clearHistory(); await refresh(1); await tick(); listbox?.focus(); }
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
    requestAnimationFrame(() => listbox?.focus());
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
    try { await openClip(selectedId); }
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
    if (type === "formatted_text") return "在默认应用中查看富文本";
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
    settings = null;
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
    void select(id);
  }

  function restoreListFocus() {
    menuOpen = false;
    appMenuOpen = false;
    if (view === "history") requestAnimationFrame(() => listbox?.focus());
  }

  function onListKeydown(event: KeyboardEvent) {
    const index = page.items.findIndex((item) => item.id === selectedId);
    const selectIndex = (next: number) => void select(page.items[Math.max(0, Math.min(next, page.items.length - 1))]?.id ?? null);
    if (event.key === "ArrowDown") { event.preventDefault(); selectIndex(index + 1); }
    else if (event.key === "ArrowUp") { event.preventDefault(); selectIndex(index - 1); }
    else if (event.key === "Home") { event.preventDefault(); selectIndex(0); }
    else if (event.key === "End") { event.preventDefault(); selectIndex(page.items.length - 1); }
    else if (event.key === "PageDown" && page.page < page.total_pages) { event.preventDefault(); void refresh(page.page + 1); }
    else if (event.key === "PageUp" && page.page > 1) { event.preventDefault(); void refresh(page.page - 1); }
    else if (event.key === "ArrowLeft" && page.page > 1) { event.preventDefault(); void refresh(page.page - 1); }
    else if (event.key === "ArrowRight" && page.page < page.total_pages) { event.preventDefault(); void refresh(page.page + 1); }
    else if (event.key === " " || event.code === "Space") {
      event.preventDefault();
      void openSelectedClip();
    }
    else if (event.key === "Enter") { event.preventDefault(); void copy(event.shiftKey ? "plain" : "rich"); }
    else if ((event.metaKey || event.ctrlKey) && ["Backspace", "Delete"].includes(event.key)) {
      event.preventDefault();
      void requestPendingAction("delete");
    }
    else if (/^[0-9]$/.test(event.key)) {
      const target = event.key === "0" ? 9 : Number(event.key) - 1;
      if (page.items[target]) { event.preventDefault(); void select(page.items[target].id); }
    }
  }

  function onKeydown(event: KeyboardEvent) {
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
      const files = Array.isArray(detail.metadata.files) ? detail.metadata.files : [];
      facts.push({ label: "文件", value: `${files.length || 1} 项` });
      const sizes = Array.isArray(detail.metadata.file_sizes)
        ? detail.metadata.file_sizes.filter((size): size is number => typeof size === "number") : [];
      if (sizes.length) facts.push({ label: "大小", value: formatBytes(sizes.reduce((sum, size) => sum + size, 0)) });
      return facts;
    }
    const charCount = countValue(detail, "char_count") ?? detail.plain_text?.length ?? 0;
    if (charCount) facts.push({ label: "字符", value: charCount.toLocaleString() });
    const formats = detail.flavors
      .filter((flavor) => flavor.format === "text/html" || flavor.format === "text/rtf")
      .map((flavor) => flavor.format === "text/html" ? "HTML" : "RTF");
    if (formats.length) facts.push({ label: "格式", value: formats.join(" + ") });
    facts.push({ label: "大小", value: formatBytes(detail.byte_size) });
    return facts;
  }
</script>

<svelte:window onkeydown={onKeydown} onfocus={restoreListFocus} />

<main class="panel" aria-label="ClipClop 剪贴板历史">
  <header class="titlebar">
    {#if view === "history"}
      <div class="brand">
        <div class="app-menu-wrap">
          <button bind:this={appMenuButton} class="app-menu-trigger" aria-label="ClipClop 应用菜单" aria-haspopup="menu" aria-expanded={appMenuOpen} onclick={() => void openAppMenu()}><img src="/favicon.png" alt="" /></button>
          {#if appMenuOpen}
            <div class="menu app-menu" role="menu" tabindex="-1" aria-label="ClipClop 应用菜单" onkeydown={onMenuKeydown}>
              <button data-menu-item role="menuitem" onclick={() => void openSettingsView()}>设置… <kbd>{settingsShortcut}</kbd></button>
              <button data-menu-item role="menuitem" class="danger" onclick={() => void quitApp()}>退出 ClipClop</button>
            </div>
          {/if}
        </div>
        <span>ClipClop</span>
      </div>
    {:else}
      <button class="back" aria-label="返回历史记录" onclick={closeSettingsView}>←</button>
      <span class="settings-title">设置</span>
    {/if}
    <div class="titlebar-drag" data-tauri-drag-region></div>
  </header>
  {#if view === "history"}
  <section class="left">
    <form class="search" onsubmit={(e) => { e.preventDefault(); onSearch(); }}>
      <span aria-hidden="true">⌕</span>
      <input bind:this={searchInput} bind:value={query} oninput={onSearch} aria-label="搜索剪贴板历史" placeholder="搜索剪贴板…" />
      <kbd>/</kbd>
    </form>
    <div bind:this={listbox} class="list" role="listbox" aria-label="剪贴板历史" aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined} onkeydown={onListKeydown}>
      {#if loading && page.items.length === 0}
        <div class="empty">正在读取历史…</div>
      {:else if error && page.items.length === 0}
        <button class="empty retry" onclick={() => refresh(1)}>读取失败，点击重试</button>
      {:else if page.items.length === 0}
        <div class="empty">{query ? "没有匹配结果" : "复制一点内容，然后再回来听见哒哒声。"}</div>
      {:else}
        {#each page.items as item, index (item.id)}
          <div id={`clip-${item.id}`} class:selected={item.id === selectedId} class="row" role="option" tabindex="-1" aria-selected={item.id === selectedId} aria-posinset={index + 1} aria-setsize={page.items.length} ondblclick={() => copy("rich")} onclick={() => selectFromList(item.id)} onkeydown={onListKeydown} animate:flip={{ duration: reducedMotion ? 0 : 180, easing: cubicOut }} out:fade={{ duration: reducedMotion ? 0 : 90 }}>
            <span class="num">{index === 9 ? 0 : index + 1}</span>
            <span class:swatch={item.content_type === "color"} class:media={item.content_type === "image" || item.content_type === "file"} class:file={item.content_type === "file"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>
              {#if thumbnailUrls[item.id]}<img src={thumbnailUrls[item.id]} alt="" />
              {:else if item.content_type === "image"}<span aria-hidden="true">▧</span>
              {:else if item.content_type === "file"}<span class="file-icon" aria-hidden="true"></span>{/if}
            </span>
            <span class="snippet">{item.preview}</span>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class="preview" aria-live="polite">
    {#if detail}
      <div class="preview-body">
        {#if detail.content_type === "color"}
          <div class="color-preview"><span style:background={detail.preview}></span><code>{detail.preview}</code></div>
        {:else if detail.content_type === "file"}
          {#if assetUrl}<img class="asset" src={assetUrl} alt="文件缩略图" />{/if}
          <div class="file-name">{detail.preview}</div>
          {#if Array.isArray(detail.metadata.files)}<div class="paths"><span>路径</span><code>{detail.metadata.files.join("\n")}</code></div>{/if}
        {:else if detail.content_type === "image"}
          {#if assetUrl}<div class="asset-frame"><img class="asset" src={assetUrl} alt="剪贴板图片预览" /></div>
          {:else}<div class="image-placeholder">图片 · {String(detail.metadata.width ?? "?")}×{String(detail.metadata.height ?? "?")}</div>{/if}
        {:else}
          <pre>{detail.plain_text ?? detail.preview}</pre>
        {/if}
      </div>
      <div class="preview-meta">
        <div class="meta-source">
        {#if sourceIconUrl}
          <img class="app-icon" src={sourceIconUrl} alt="" />
        {:else}
          <span class="app-fallback" aria-hidden="true">{detail.source_app?.name?.slice(0, 1) ?? "?"}</span>
        {/if}
          <div class="source-details"><span>{detail.source_app?.name ?? "未知来源"}</span><time>{exactTime(detail.created_at)}</time></div>
        </div>
        <dl class="meta-facts">
          {#each metadataFacts(detail) as fact}
            <div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>
          {/each}
        </dl>
      </div>
    {:else}
      <div class="empty">选择一条记录查看内容</div>
    {/if}
  </section>

  <footer class="pager">
    <button disabled={page.page <= 1} onclick={() => refresh(page.page - 1)} aria-label="上一页">←</button>
    <span>{page.total_pages === 0 ? 0 : page.page}/{page.total_pages}</span>
    <button disabled={page.page >= page.total_pages} onclick={() => refresh(page.page + 1)} aria-label="下一页">→</button>
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
        <button bind:this={menuButton} class="ghost" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => void openMenu()}><kbd>⌘K</kbd> 操作</button>
        {#if menuOpen}
          <div class="menu" role="menu" tabindex="-1" aria-label="操作菜单" onkeydown={onMenuKeydown}>
            <button data-menu-item role="menuitem" onclick={() => copy("plain")} disabled={!selectedId}>复制为纯文本 <kbd>⇧↵</kbd></button>
            <button data-menu-item role="menuitem" onclick={() => void openSelectedClip()} disabled={!selectedId}>{openActionLabel()} <kbd>Space</kbd></button>
            <button data-menu-item role="menuitem" onclick={ignoreSelectedSource} disabled={!detail?.source_app}>忽略此来源应用</button>
            <button data-menu-item role="menuitem" onclick={() => void requestPendingAction("delete")} disabled={!selectedId}>从 ClipClop 删除 <kbd>{deleteShortcut}</kbd></button>
            <button data-menu-item role="menuitem" class="danger" onclick={() => void requestPendingAction("clear")} disabled={page.total === 0}>清空全部历史</button>
          </div>
        {/if}
      </div>
      <button class="copy" onclick={() => copy("rich")} disabled={!selectedId}><kbd>⏎</kbd> 复制</button>
    {/if}
  </footer>
  {:else}
    <section class="preferences" aria-label="ClipClop 设置">
      {#if settings}
        <label><span><strong>开机启动</strong><small>登录系统后在后台启动 ClipClop。</small></span><input bind:this={settingsFirstControl} type="checkbox" bind:checked={settings.launch_at_login} /></label>
        <label><span><strong>保留期限</strong><small>超出期限的历史会在后续捕获时清理。</small></span><select bind:value={settings.retention_days}><option value={7}>7 天</option><option value={30}>30 天</option><option value={90}>90 天</option></select></label>
        <label><span><strong>外观</strong><small>跟随系统，或固定使用 Light/Dark。</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">跟随系统</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
        <div class="preference-row"><span><strong>全局快捷键</strong><small>当前版本使用平台默认值；暂不支持自定义。</small></span><kbd>{settings.hotkey}</kbd></div>
        {#if settings.ignored_apps.length > 0}
          <div class="ignored-apps"><strong>已忽略的应用</strong>{#each settings.ignored_apps as appId}<div><code title={appId}>{appLabel(appId)}</code><button onclick={() => removeIgnoredApp(appId)}>移除</button></div>{/each}</div>
        {/if}
      {:else}
        <div class="settings-loading">{settingsStatus || "正在读取设置…"}</div>
      {/if}
    </section>
    <footer class="settings-actions">
      <span aria-live="polite" class:error={settingsStatus !== "" && settingsStatus !== "已保存"}>{settingsStatus}</span>
      <button class="ghost" onclick={closeSettingsView}>返回 <kbd>Esc</kbd></button>
      <button class="copy" onclick={() => void saveSettings()} disabled={!settings}>保存</button>
    </footer>
  {/if}
</main>

<style>
  .panel { width:calc(100vw - 40px); height:calc(100vh - 40px); margin:20px; display:grid; grid-template-columns:300px 1fr; grid-template-rows:42px 1fr 48px; background:var(--bg-shell); border-radius:14px; box-shadow:var(--panel-shadow); overflow:hidden; }
  .titlebar { grid-column:1 / -1; grid-row:1; display:flex; align-items:center; padding:0 14px; border-bottom:1px solid var(--hairline); user-select:none; }
  .titlebar-drag { flex:1; align-self:stretch; }
  .brand { display:flex; align-items:center; gap:7px; color:var(--text-2); font-size:12px; font-weight:600; letter-spacing:.01em; }
  .app-menu-wrap { position:relative; }
  .app-menu-trigger { display:grid; place-items:center; width:24px; height:24px; padding:0; border-radius:5px; background:transparent; }
  .app-menu-trigger:hover { background:var(--bg-hover); }
  .brand img { width:18px; height:18px; border-radius:5px; }
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
  .row { width:100%; min-height:44px; display:flex; align-items:center; gap:8px; padding:7px 8px; border-radius:8px; color:var(--text-1); background:transparent; text-align:left; cursor:default; }
  .row:hover { background:var(--bg-hover); }
  .row.selected { background:var(--bg-selected); }
  .num { width:14px; flex:none; color:var(--text-3); font:11px var(--mono); text-align:center; }
  .lead { width:28px; height:28px; flex:none; display:flex; align-items:center; justify-content:center; border-radius:4px; color:var(--text-2); font:7px var(--mono); }
  .lead.swatch { color:transparent; border:1px solid var(--hairline); }
  .lead.media { overflow:hidden; background:var(--bg-raised); font:15px/1 system-ui; }
  .lead.media img { width:100%; height:100%; object-fit:cover; }
  .file-icon { width:13px; height:16px; position:relative; border:1px solid var(--text-2); border-radius:2px; }
  .file-icon::after { content:""; position:absolute; top:-1px; right:-1px; width:5px; height:5px; border-left:1px solid var(--text-2); border-bottom:1px solid var(--text-2); background:var(--bg-raised); }
  .snippet { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font:13px/1.5 var(--mono); }
  .preview { grid-column:2; grid-row:2; min-width:0; min-height:0; display:flex; flex-direction:column; }
  .preview-body { flex:1; min-height:0; overflow:auto; padding:20px; }
  pre { margin:0; color:var(--text-1); font:13px/1.65 var(--mono); white-space:pre-wrap; overflow-wrap:anywhere; }
  .preview-meta { height:64px; flex:none; display:flex; align-items:center; justify-content:space-between; gap:20px; padding:8px 20px; border-top:1px solid var(--hairline); }
  .meta-source { min-width:0; display:flex; align-items:center; gap:8px; }
  .source-details { min-width:0; display:flex; flex-direction:column; gap:2px; color:var(--text-2); font:12px/1.2 var(--mono); }
  .source-details span { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .source-details time { color:var(--text-3); font-size:10px; }
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
  .file-name { margin-top:16px; color:var(--text-1); font:13px/1.5 var(--mono); overflow-wrap:anywhere; }
  .paths { display:flex; flex-direction:column; gap:5px; margin-top:12px; color:var(--text-2); }
  .paths span { color:var(--text-3); font-size:10px; }
  .paths code { color:inherit; font:11px/1.6 var(--mono); white-space:pre-wrap; overflow-wrap:anywhere; }
  .image-placeholder { min-height:180px; display:grid; place-items:center; color:var(--text-3); border:1px solid var(--hairline); border-radius:8px; }
  .asset-frame { width:100%; height:100%; min-height:180px; display:flex; align-items:center; justify-content:center; }
  .asset { display:block; max-width:100%; max-height:100%; border-radius:8px; object-fit:contain; }
  .empty { flex:1; display:grid; place-items:center; padding:24px; color:var(--text-3); font-size:13px; text-align:center; background:transparent; }
  .retry { width:100%; cursor:pointer; }
  .pager { grid-column:1; grid-row:3; display:flex; align-items:center; gap:10px; padding:0 16px; border-top:1px solid var(--hairline); border-right:1px solid var(--hairline); color:var(--text-2); font:12px var(--mono); }
  .pager button { padding:2px 7px; border:1px solid var(--hairline); border-radius:4px; color:var(--text-2); background:transparent; }
  .pager button:disabled { opacity:.35; }
  .actions { grid-column:2; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, .ghost, .destructive { display:flex; align-items:center; gap:6px; border-radius:6px; color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:650; }
  .copy:hover { background:var(--action-hover); }
  .copy kbd { color:inherit; border-color:currentColor; opacity:.9; }
  .destructive { color:var(--danger-on); background:var(--danger-fill); font-weight:600; }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu { position:absolute; right:0; bottom:38px; width:210px; padding:6px; border:1px solid var(--hairline); border-radius:8px; background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .app-menu { top:30px; bottom:auto; left:0; right:auto; width:180px; }
  .menu button { width:100%; display:flex; justify-content:space-between; padding:9px 10px; border-radius:6px; color:var(--text-1); background:transparent; text-align:left; }
  .menu button:hover { background:var(--bg-hover); }
  .menu .danger { color:var(--danger); }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:11px; }
  .message.error { color:var(--danger); }
  .confirmation { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  .confirmation > span { margin-right:auto; color:var(--text-1); font-size:12px; font-weight:600; }
  .confirmation small { display:block; margin-top:2px; color:var(--text-2); font-size:10px; font-weight:400; }
  .preferences { grid-column:1 / -1; grid-row:2; min-height:0; overflow:auto; padding:0 20px; }
  .preferences label, .preference-row { min-height:68px; display:flex; align-items:center; justify-content:space-between; gap:24px; border-bottom:1px solid var(--hairline); }
  .preferences label > span, .preference-row > span { display:flex; flex-direction:column; gap:3px; }
  .preferences strong { color:var(--text-1); font-size:13px; font-weight:600; }
  .preferences small { color:var(--text-3); font-size:11px; }
  .preferences select { min-width:116px; padding:7px 28px 7px 9px; border:1px solid var(--hairline); border-radius:6px; color:var(--text-1); background:var(--bg-raised); }
  .preferences input { width:18px; height:18px; accent-color:var(--text-1); }
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
</style>
