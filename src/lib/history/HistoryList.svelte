<script lang="ts">
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import { ChevronLeft, ChevronRight, File, Image, Search } from "@lucide/svelte";
  import { formatNumber, t } from "$lib/i18n/index.svelte";
  import { canExpand, clipPreview, fileName, groupedFiles } from "./presentation";
  import type { HistoryPage } from "./types";

  let {
    page,
    query = $bindable(),
    selectedId,
    expandedId,
    fileIndex,
    loading,
    error,
    thumbnailUrls,
    reducedMotion,
    rowReorderMotion,
    onsearch,
    onselect,
    onpaste,
    onfile,
    onkeydown,
    onpage,
  }: {
    page: HistoryPage;
    query: string;
    selectedId: string | null;
    expandedId: string | null;
    fileIndex: number;
    loading: boolean;
    error: string;
    thumbnailUrls: Record<string, string>;
    reducedMotion: boolean;
    rowReorderMotion: boolean;
    onsearch: () => void;
    onselect: (id: string) => void;
    onpaste: () => void;
    onfile: (index: number) => void;
    onkeydown: (event: KeyboardEvent) => void;
    onpage: (page: number) => void;
  } = $props();

  let listbox: HTMLDivElement;
  let searchInput: HTMLInputElement;

  export function focus() { listbox?.focus(); }
  export function hasFocus() { return document.activeElement === listbox; }
  export function focusSearch() { searchInput?.focus(); }
</script>

<section class="left">
  <form class="search" onsubmit={(event) => { event.preventDefault(); onsearch(); }}>
    <span aria-hidden="true"><Search size={15} /></span>
    <input bind:this={searchInput} bind:value={query} oninput={onsearch} aria-label={t("history.searchLabel")} placeholder={t("history.searchPlaceholder")} />
    <kbd>/</kbd>
  </form>
  <div bind:this={listbox} class:full={page.items.length === page.page_size} class="list" role="listbox" aria-label={t("history.list")} aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined} onkeydown={onkeydown}>
    {#if loading && page.items.length === 0}
      <div class="empty">{t("history.loading")}</div>
    {:else if error && page.items.length === 0}
      <button class="empty retry" onclick={() => onpage(1)}>{t("history.retry")}</button>
    {:else if page.items.length === 0}
      <div class="empty">{query ? t("history.noMatches") : t("history.empty")}</div>
    {:else}
      {#each page.items as item, index (item.id)}
        <div class:expanded={canExpand(item) && expandedId === item.id} class="clip-item" animate:flip={{ duration: reducedMotion || !rowReorderMotion ? 0 : 180, easing: cubicOut }} out:fade={{ duration: reducedMotion || !rowReorderMotion ? 0 : 90 }}>
          <div id={`clip-${item.id}`} class:selected={item.id === selectedId} class="row" role="option" tabindex="-1" aria-selected={item.id === selectedId} aria-posinset={(page.page - 1) * page.page_size + index + 1} aria-setsize={page.total} ondblclick={onpaste} onclick={() => onselect(item.id)} onkeydown={onkeydown}>
            <span class="num">{formatNumber(index === 9 ? 0 : index + 1)}</span>
            <span class:swatch={item.content_type === "color"} class:media={item.content_type === "image" || item.content_type === "file"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>
              {#if thumbnailUrls[item.id]}<img src={thumbnailUrls[item.id]} alt="" />
              {:else if item.content_type === "image"}<span aria-hidden="true"><Image size={16} /></span>
              {:else if item.content_type === "file"}<File size={16} aria-hidden="true" />{/if}
            </span>
            <span class="snippet">{clipPreview(item)}</span>
            {#if canExpand(item)}<span class="disclosure" aria-hidden="true"><ChevronRight size={16} /></span>{/if}
          </div>
          {#if canExpand(item) && expandedId === item.id}
            <div class="row-details" role="group" in:fade={{ duration: reducedMotion ? 0 : 120 }} out:fade={{ duration: reducedMotion ? 0 : 90 }}>
              {#each groupedFiles(item) as path, index}
                <button tabindex="-1" class:selected={index === fileIndex} class="row-child" onclick={(event) => { event.stopPropagation(); onfile(index); }}>{fileName(path)}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</section>

<footer class="pager">
  <button disabled={page.page <= 1} onclick={() => onpage(page.page - 1)} aria-label={t("history.previousPage")}><ChevronLeft size={16} aria-hidden="true" /></button>
  <span>{formatNumber(page.total_pages === 0 ? 0 : page.page)}/{formatNumber(page.total_pages)}</span>
  <button disabled={page.page >= page.total_pages} onclick={() => onpage(page.page + 1)} aria-label={t("history.nextPage")}><ChevronRight size={16} aria-hidden="true" /></button>
</footer>

<style>
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
  .num { width:16px; flex:none; color:var(--text-3); font-size:12px; font-weight:650; line-height:1; font-variant-numeric:tabular-nums lining-nums; text-align:center; }
  .list:focus .row.selected .num { color:var(--text-2); }
  .lead { width:28px; height:28px; flex:none; display:flex; align-items:center; justify-content:center; border-radius:4px; color:var(--text-2); font:7px var(--mono); }
  .lead.swatch { color:transparent; border:1px solid var(--hairline); }
  .lead.media { overflow:hidden; background:var(--bg-raised); }
  .lead.media img { width:100%; height:100%; object-fit:cover; }
  .snippet { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font:13px/1.5 var(--mono); }
  .disclosure { width:16px; flex:none; display:flex; justify-content:center; color:var(--text-3); transition:transform 160ms cubic-bezier(.16, 1, .3, 1); }
  .clip-item.expanded .disclosure { transform:rotate(90deg); }
  .row-details { margin:0 8px 4px 50px; padding:3px 8px 7px; }
  .row-child { width:100%; overflow:hidden; padding:4px 6px; border-radius:4px; color:var(--text-2); background:transparent; font:11px/1.45 var(--mono); text-align:left; text-overflow:ellipsis; white-space:nowrap; }
  .row-child:hover, .row-child.selected { background:var(--bg-hover); color:var(--text-1); }
  .pager { grid-column:1; grid-row:3; display:grid; grid-template-columns:36px minmax(32px, auto) 36px; align-items:center; gap:8px; padding:0 14px; border-top:1px solid var(--hairline); border-right:1px solid var(--hairline); color:var(--text-2); font:12px var(--mono); }
  .pager span { min-width:32px; text-align:center; font-variant-numeric:tabular-nums; }
  .pager button { width:36px; height:30px; display:grid; place-items:center; padding:0; border:1px solid var(--hairline); border-radius:4px; color:var(--text-2); background:transparent; }
  .pager button:hover:not(:disabled) { background:var(--bg-hover); }
</style>
