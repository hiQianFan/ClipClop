<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { clearHistory, copyClip, deleteClip, getClip, getClipAsset, listClips } from "$lib/clips/api";
  import type { AppError, ClipDetail, ClipPage, ClipSummary, ContentType } from "$lib/clips/types";
  import { applyTheme, getSettings, ignoreSource, openSettings } from "$lib/settings/api";

  let page = $state<ClipPage>({ items: [], page: 1, page_size: 10, total: 0, total_pages: 0 });
  let selectedId = $state<string | null>(null);
  let detail = $state<ClipDetail | null>(null);
  let assetUrl = $state<string | null>(null);
  let query = $state("");
  let loading = $state(true);
  let error = $state("");
  let copied = $state("");
  let menuOpen = $state(false);
  let searchInput: HTMLInputElement;
  let requestVersion = 0;

  const labels: Record<ContentType, string> = {
    text: "文本", formatted_text: "富文本", link: "链接", color: "颜色",
    code: "代码", image: "图片", file: "文件",
  };

  onMount(() => {
    getSettings().then((settings) => applyTheme(settings.theme)).catch(() => {});
    void refresh(1);
    const unlisten = listen("clips_changed", () => refresh(page.page));
    return () => { unlisten.then((fn) => fn()); };
  });

  async function refresh(targetPage = page.page) {
    loading = true;
    error = "";
    try {
      page = await listClips(query, targetPage);
      const next = page.items.some((item) => item.id === selectedId)
        ? selectedId : page.items[0]?.id ?? null;
      await select(next);
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      loading = false;
    }
  }

  async function select(id: string | null) {
    selectedId = id;
    detail = null;
    assetUrl = null;
    if (!id) return;
    const version = ++requestVersion;
    try {
      const next = await getClip(id);
      if (version === requestVersion) {
        detail = next;
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
    try { await deleteClip(selectedId); await refresh(page.page); }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function clearAll() {
    if (!confirm("清空全部剪贴板历史？此操作无法撤销。")) return;
    try { await clearHistory(); await refresh(1); }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  async function ignoreSelectedSource() {
    const source = detail?.source_app;
    if (!source) return;
    if (!confirm(`以后不再记录来自“${source.name}”的内容？`)) return;
    try { await ignoreSource(source.id); copied = `已忽略 ${source.name}`; }
    catch (reason) { error = errorMessage(reason); }
    menuOpen = false;
  }

  function onSearch() { void refresh(1); }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "/" && document.activeElement !== searchInput) {
      event.preventDefault(); searchInput.focus(); return;
    }
    if (event.key === "Escape") {
      if (document.activeElement === searchInput) { searchInput.blur(); return; }
      if (menuOpen) { menuOpen = false; return; }
      void getCurrentWindow().hide(); return;
    }
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "c") {
      event.preventDefault(); void copy("plain"); return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault(); menuOpen = !menuOpen; return;
    }
    if (document.activeElement === searchInput) return;
    const index = page.items.findIndex((item) => item.id === selectedId);
    if (event.key === "ArrowDown") {
      event.preventDefault(); void select(page.items[Math.min(index + 1, page.items.length - 1)]?.id ?? null);
    } else if (event.key === "ArrowUp") {
      event.preventDefault(); void select(page.items[Math.max(index - 1, 0)]?.id ?? null);
    } else if (event.key === "ArrowLeft" && page.page > 1) {
      event.preventDefault(); void refresh(page.page - 1);
    } else if (event.key === "ArrowRight" && page.page < page.total_pages) {
      event.preventDefault(); void refresh(page.page + 1);
    } else if (event.key === "Enter") {
      event.preventDefault(); void copy("rich");
    } else if (/^[0-9]$/.test(event.key)) {
      const target = event.key === "0" ? 9 : Number(event.key) - 1;
      if (page.items[target]) { event.preventDefault(); void select(page.items[target].id); }
    }
  }

  function errorMessage(reason: unknown) {
    if (typeof reason === "object" && reason && "message" in reason) return String((reason as AppError).message);
    return String(reason ?? "未知错误");
  }

  function relativeTime(value: string) {
    const seconds = Math.max(0, (Date.now() - new Date(value).getTime()) / 1000);
    if (seconds < 60) return "刚刚";
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
  }

  function lead(item: ClipSummary) {
    if (item.content_type === "color") return item.preview;
    if (item.content_type === "file") return "FILE";
    if (item.content_type === "image") return "IMG";
    if (item.content_type === "link") return "↗";
    if (item.content_type === "formatted_text") return "RTF";
    return "";
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="panel" aria-label="ClipClop 剪贴板历史">
  <section class="left">
    <form class="search" onsubmit={(e) => { e.preventDefault(); onSearch(); }}>
      <span aria-hidden="true">⌕</span>
      <input bind:this={searchInput} bind:value={query} oninput={onSearch} aria-label="搜索剪贴板历史" placeholder="搜索剪贴板…" />
      <kbd>/</kbd>
    </form>
    <div class="list" role="listbox" aria-label="剪贴板历史">
      {#if loading && page.items.length === 0}
        <div class="empty">正在读取历史…</div>
      {:else if error && page.items.length === 0}
        <button class="empty retry" onclick={() => refresh(1)}>读取失败，点击重试</button>
      {:else if page.items.length === 0}
        <div class="empty">{query ? "没有匹配结果" : "复制一点内容，然后再回来听见哒哒声。"}</div>
      {:else}
        {#each page.items as item, index (item.id)}
          <button class:selected={item.id === selectedId} class="row" role="option" aria-selected={item.id === selectedId} ondblclick={() => copy("rich")} onclick={() => select(item.id)}>
            <span class="num">{index === 9 ? 0 : index + 1}</span>
            <span class:swatch={item.content_type === "color"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>{lead(item)}</span>
            <span class="snippet">{item.preview}</span>
            <span class="time">{relativeTime(item.created_at)}</span>
          </button>
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
          <pre>{detail.preview}</pre>
          {#if Array.isArray(detail.metadata.files)}<div class="paths">{detail.metadata.files.join("\n")}</div>{/if}
        {:else if detail.content_type === "image"}
          {#if assetUrl}<img class="asset" src={assetUrl} alt="剪贴板图片预览" />
          {:else}<div class="image-placeholder">图片 · {String(detail.metadata.width ?? "?")}×{String(detail.metadata.height ?? "?")}</div>{/if}
        {:else}
          <pre>{detail.plain_text ?? detail.preview}</pre>
        {/if}
      </div>
      <div class="preview-meta">
        <span class="app-dot"></span>
        {detail.source_app?.name ?? "未知来源"} · {labels[detail.content_type]} · {relativeTime(detail.created_at)}
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
    {#if error}<span class="message error" title={error}>{error}</span>{/if}
    {#if copied}<span class="message">{copied}</span>{/if}
    <div class="menu-wrap">
      <button class="ghost" onclick={() => menuOpen = !menuOpen}><kbd>⌘K</kbd> 操作</button>
      {#if menuOpen}
        <div class="menu">
          <button onclick={() => copy("plain")} disabled={!selectedId}>复制为纯文本 <kbd>⇧⌘C</kbd></button>
          <button onclick={() => { menuOpen = false; void openSettings(); }}>设置</button>
          <button onclick={ignoreSelectedSource} disabled={!detail?.source_app}>忽略此来源应用</button>
          <button onclick={removeSelected} disabled={!selectedId}>删除此项</button>
          <button class="danger" onclick={clearAll} disabled={page.total === 0}>清空全部历史</button>
        </div>
      {/if}
    </div>
    <button class="copy" onclick={() => copy("rich")} disabled={!selectedId}><kbd>⏎</kbd> 复制</button>
  </footer>
</main>

<style>
  .panel { width:100vw; height:100vh; display:grid; grid-template-columns:300px 1fr; grid-template-rows:1fr 48px; background:var(--bg-shell); border:1px solid var(--hairline); border-radius:14px; overflow:hidden; }
  .left { min-height:0; display:flex; flex-direction:column; border-right:1px solid var(--hairline); }
  .search { height:42px; flex:none; display:flex; align-items:center; gap:8px; padding:0 14px; color:var(--text-3); border-bottom:1px solid var(--hairline); }
  .search input { min-width:0; flex:1; border:0; outline:0; padding:0; color:var(--text-1); background:transparent; font-size:13px; }
  .search input::placeholder { color:var(--text-2); }
  kbd { font:10px/1.4 var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:4px; padding:1px 5px; white-space:nowrap; }
  .list { flex:1; min-height:0; display:flex; flex-direction:column; gap:1px; padding:6px; }
  .row { width:100%; min-height:44px; display:flex; align-items:center; gap:8px; padding:7px 8px; border-radius:8px; color:var(--text-1); background:transparent; text-align:left; cursor:default; }
  .row:hover { background:var(--bg-hover); }
  .row.selected { background:var(--bg-selected); }
  .num { width:14px; flex:none; color:var(--text-3); font:11px var(--mono); text-align:center; }
  .lead { width:26px; height:26px; flex:none; display:flex; align-items:center; justify-content:center; border-radius:5px; color:var(--text-2); font:7px var(--mono); }
  .lead.swatch { color:transparent; border:1px solid var(--hairline); }
  .snippet { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font:13px/1.5 var(--mono); }
  .time { flex:none; color:var(--text-3); font-size:11px; }
  .preview { min-width:0; min-height:0; display:flex; flex-direction:column; }
  .preview-body { flex:1; min-height:0; overflow:auto; padding:20px; }
  pre { margin:0; color:var(--text-1); font:13px/1.65 var(--mono); white-space:pre-wrap; overflow-wrap:anywhere; }
  .preview-meta { min-height:40px; flex:none; display:flex; align-items:center; gap:6px; padding:8px 20px; border-top:1px solid var(--hairline); color:var(--text-3); font:11px var(--mono); }
  .app-dot { width:14px; height:14px; border-radius:3px; background:var(--text-3); }
  .color-preview { display:flex; align-items:center; gap:14px; }
  .color-preview span { width:72px; height:72px; border:1px solid var(--hairline); border-radius:10px; }
  .color-preview code, .paths { color:var(--text-2); font:12px/1.6 var(--mono); white-space:pre-wrap; }
  .image-placeholder { min-height:180px; display:grid; place-items:center; color:var(--text-3); border:1px solid var(--hairline); border-radius:8px; }
  .asset { display:block; max-width:100%; max-height:100%; margin-bottom:14px; border-radius:8px; object-fit:contain; }
  .empty { flex:1; display:grid; place-items:center; padding:24px; color:var(--text-3); font-size:13px; text-align:center; background:transparent; }
  .retry { width:100%; cursor:pointer; }
  .pager { display:flex; align-items:center; gap:10px; padding:0 16px; border-top:1px solid var(--hairline); border-right:1px solid var(--hairline); color:var(--text-2); font:12px var(--mono); }
  .pager button { padding:2px 7px; border:1px solid var(--hairline); border-radius:4px; color:var(--text-2); background:transparent; }
  .pager button:disabled { opacity:.35; }
  .actions { display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, .ghost { display:flex; align-items:center; gap:6px; border-radius:6px; color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:600; }
  .copy:hover { background:var(--action-hover); }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu { position:absolute; right:0; bottom:38px; width:210px; padding:6px; border:1px solid var(--hairline); border-radius:10px; background:var(--bg-raised); box-shadow:0 8px 20px rgba(0,0,0,.25); }
  .menu button { width:100%; display:flex; justify-content:space-between; padding:9px 10px; border-radius:6px; color:var(--text-1); background:transparent; text-align:left; }
  .menu button:hover { background:var(--bg-hover); }
  .menu .danger { color:var(--danger); }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:11px; }
  .message.error { color:var(--danger); }
  @media (max-width:680px) { .panel { grid-template-columns:280px 1fr; } }
  @media (prefers-reduced-motion:no-preference) { .panel { animation:enter 120ms ease-out; } @keyframes enter { from { opacity:0; transform:scale(.98); } } }
</style>
