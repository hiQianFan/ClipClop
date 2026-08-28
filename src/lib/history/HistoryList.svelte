<script lang="ts">
  import { onDestroy } from "svelte";
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import { Popover } from "bits-ui";
  import { ArrowLeft, ArrowRight, ChevronRight, File, Image, Search, SlidersHorizontal } from "@lucide/svelte";
  import { formatNumber, t } from "$lib/i18n/index.svelte";
  import type { StaticMessageKey } from "$lib/i18n/index.svelte";
  import { performPagerHaptic } from "./api";
  import { draggedPage, PAGE_DRAG_STEP, visiblePageTicks } from "./pager";
  import { canExpand, clipPreview, fileName, groupedFiles } from "./presentation";
  import type { ContentType, HistoryFilters, HistoryPage, HistorySourceOption } from "./types";

  let {
    page,
    query = $bindable(),
    filters,
    sources,
    typeTotal,
    typeCounts,
    selectedId,
    expandedId,
    fileIndex,
    loading,
    error,
    thumbnailUrls,
    reducedMotion,
    rowReorderMotion,
    onsearch,
    onsearchfocus,
    onsearchkeydown,
    onfilterschange,
    onsourcequery,
    onclearsearch,
    onlistfocus,
    onselect,
    onpaste,
    onfile,
    onkeydown,
    onpage,
  }: {
    page: HistoryPage;
    query: string;
    filters: HistoryFilters;
    sources: HistorySourceOption[];
    typeTotal: number;
    typeCounts: Partial<Record<ContentType, number>>;
    selectedId: string | null;
    expandedId: string | null;
    fileIndex: number;
    loading: boolean;
    error: string;
    thumbnailUrls: Record<string, string>;
    reducedMotion: boolean;
    rowReorderMotion: boolean;
    onsearch: () => void;
    onsearchfocus: () => void;
    onsearchkeydown: (event: KeyboardEvent) => void;
    onfilterschange: () => void;
    onsourcequery: (query: string) => void;
    onclearsearch: () => void;
    onlistfocus: () => void;
    onselect: (id: string) => void;
    onpaste: () => void;
    onfile: (index: number) => void;
    onkeydown: (event: KeyboardEvent) => void;
    onpage: (page: number) => void;
  } = $props();

  const contentTypes: Array<{ value: ContentType | null; label: StaticMessageKey }> = [
    { value: null, label: "filter.all" }, { value: "text", label: "filter.text" },
    { value: "link", label: "filter.link" },
    { value: "image", label: "filter.image" }, { value: "file", label: "filter.file" },
    { value: "color", label: "filter.color" },
  ];
  const timeRanges: Array<{ value: HistoryFilters["time_range"]; label: StaticMessageKey }> = [
    { value: "any", label: "filter.anyTime" }, { value: "day", label: "filter.today" },
    { value: "week", label: "filter.week" }, { value: "month", label: "filter.month" },
  ];
  const activeFilterCount = $derived(Number(filters.content_type !== null) + Number(filters.source_id !== null) + Number(filters.time_range !== "any"));

  function applyFilter(change: Partial<HistoryFilters>) {
    Object.assign(filters, change);
    onfilterschange();
  }

  let listbox: HTMLDivElement;
  let searchInput: HTMLInputElement;
  let emptyAnchor = $state<HTMLDivElement>();
  let retryButton = $state<HTMLButtonElement>();
  let sourceQuery = $state("");
  let filterOpen = $state(false);
  let draggingPage = $state(false);
  let dragPointer = 0;
  let dragStartX = 0;
  let dragStartPage = 1;
  let dragPage = $state(1);
  let dragOffset = $state(0);
  let visualPage = $state<number | null>(null);
  let keyDirection = $state<-1 | 0 | 1>(0);
  let keyTicksVisible = $state(false);
  let keyMotionTimer: number | undefined;
  let keyHideTimer: number | undefined;
  let repeatDelay: number | undefined;
  let repeatTimer: number | undefined;
  let repeatDirection: -1 | 0 | 1 = 0;
  let wheelDistance = 0;
  let wheelEndTimer: number | undefined;
  let lastHapticAt = Number.NEGATIVE_INFINITY;
  const currentPage = () => draggingPage ? dragPage : visualPage ?? page.page;

  $effect(() => {
    if (visualPage === page.page) visualPage = null;
  });

  function tickPhase() {
    return draggingPage && repeatDirection === 0 ? dragOffset / PAGE_DRAG_STEP : keyDirection;
  }

  function tickStyle(tickPage: number) {
    const current = currentPage();
    const position = tickPage - current + tickPhase();
    const distance = Math.abs(position);
    const height = Math.max(5, 16 - distance * 3.5);
    const opacity = Math.max(.2, 1 - distance * .2);
    return `height:${height}px;opacity:${opacity};transform:translate(${position * 18}px,-50%)`;
  }

  function performStepHaptic() {
    const now = performance.now();
    if (now - lastHapticAt < 40) return;
    lastHapticAt = now;
    void performPagerHaptic();
  }

  function startPageDrag(event: PointerEvent) {
    if (event.button !== 0) return;
    event.preventDefault();
    draggingPage = true;
    dragPointer = event.pointerId;
    dragStartX = event.clientX;
    dragStartPage = currentPage();
    dragPage = currentPage();
    dragOffset = 0;
    document.documentElement.classList.add("pager-dragging");
    if (event.currentTarget instanceof HTMLElement) event.currentTarget.setPointerCapture(event.pointerId);
  }

  function stopRepeat(resetX?: number) {
    window.clearTimeout(repeatDelay);
    window.clearInterval(repeatTimer);
    repeatDirection = 0;
    if (resetX !== undefined) {
      dragStartX = resetX;
      dragStartPage = dragPage;
      dragOffset = 0;
    }
  }

  function repeatPage(direction: -1 | 1) {
    const next = dragPage + direction;
    if (next < 1 || next > page.total_pages) return stopRepeat();
    dragPage = next;
    onpage(next);
    performStepHaptic();
    animatePageStep(direction);
  }

  function startRepeat(direction: -1 | 1) {
    if (repeatDirection === direction) return;
    stopRepeat();
    repeatDirection = direction;
    repeatDelay = window.setTimeout(() => {
      repeatPage(direction);
      repeatTimer = window.setInterval(() => repeatPage(direction), 220);
    }, 500);
  }

  function movePageDrag(event: PointerEvent) {
    if (!draggingPage || event.pointerId !== dragPointer) return;
    event.preventDefault();
    if (event.currentTarget instanceof HTMLElement) {
      const bounds = event.currentTarget.getBoundingClientRect();
      if (event.clientX < bounds.left - 24) startRepeat(1);
      else if (event.clientX > bounds.right + 24) startRepeat(-1);
      else if (repeatDirection) stopRepeat(event.clientX);
    }
    if (repeatDirection) return;
    const delta = event.clientX - dragStartX;
    const next = draggedPage(dragStartPage, delta, page.total_pages);
    const remainder = delta + (next - dragStartPage) * PAGE_DRAG_STEP;
    dragOffset = next === 1 || next === page.total_pages
      ? Math.max(-18, Math.min(remainder * 0.25, 18))
      : remainder;
    if (next === dragPage) return;
    dragPage = next;
    onpage(next);
    performStepHaptic();
  }

  function stopPageDrag(event: PointerEvent) {
    if (event.pointerId !== dragPointer) return;
    stopRepeat();
    draggingPage = false;
    dragOffset = 0;
    document.documentElement.classList.remove("pager-dragging");
  }

  function animatePageStep(direction: -1 | 1) {
    window.clearTimeout(keyMotionTimer);
    window.clearTimeout(keyHideTimer);
    keyTicksVisible = true;
    keyHideTimer = window.setTimeout(() => keyTicksVisible = false, 700);
    if (reducedMotion) return;
    keyDirection = 0;
    requestAnimationFrame(() => {
      keyDirection = direction;
      keyMotionTimer = window.setTimeout(() => keyDirection = 0, 180);
    });
  }

  export function turnPage(direction: -1 | 1) {
    const next = currentPage() + direction;
    if (next < 1 || next > page.total_pages) return;
    visualPage = next;
    onpage(next);
    performStepHaptic();
    animatePageStep(direction);
  }

  function onScrubberWheel(event: WheelEvent) {
    if (Math.abs(event.deltaX) <= Math.abs(event.deltaY) * 1.25) return;
    event.preventDefault();
    window.clearTimeout(wheelEndTimer);
    wheelDistance += event.deltaX;
    while (Math.abs(wheelDistance) >= PAGE_DRAG_STEP) {
      const direction = wheelDistance > 0 ? 1 : -1;
      const before = currentPage();
      turnPage(direction);
      if (currentPage() === before) {
        wheelDistance = 0;
        break;
      }
      wheelDistance -= direction * PAGE_DRAG_STEP;
    }
    wheelEndTimer = window.setTimeout(() => wheelDistance = 0, 120);
  }

  export function focus() {
    if (page.items.length > 0) listbox?.focus();
    else if (error) retryButton?.focus();
    else emptyAnchor?.focus();
  }
  export function hasFocus() { return document.activeElement === listbox; }
  export function focusSearch() { searchInput?.focus(); }
  export function closeFilters() { filterOpen = false; sourceQuery = ""; }

  onDestroy(() => {
    if (typeof window === "undefined") return;
    stopRepeat();
    window.clearTimeout(keyMotionTimer);
    window.clearTimeout(keyHideTimer);
    window.clearTimeout(wheelEndTimer);
    document.documentElement.classList.remove("pager-dragging");
  });
</script>

<section class="left">
  <form class="search" onsubmit={(event) => { event.preventDefault(); onsearch(); }}>
    <span aria-hidden="true"><Search size={15} /></span>
    <input bind:this={searchInput} bind:value={query} oninput={onsearch} onfocus={onsearchfocus} onkeydown={onsearchkeydown} aria-label={t("history.searchLabel")} placeholder={t("history.searchPlaceholder")} />
    <Popover.Root bind:open={filterOpen}>
      <Popover.Trigger class={`filter-trigger${activeFilterCount ? " active" : ""}`} aria-label={activeFilterCount ? t("filter.active", { count: activeFilterCount }) : t("filter.open")}>
        <SlidersHorizontal size={14} aria-hidden="true" />
        {#if activeFilterCount}<span>{activeFilterCount}</span>{:else}<kbd>/</kbd>{/if}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content class="filter-popover" align="end" sideOffset={6}>
          <section><h2>{t("filter.type")}</h2><div class="filter-options type-options">
            {#each contentTypes as option}
              {@const count = option.value === null ? typeTotal : typeCounts[option.value] ?? 0}
              <button type="button" disabled={count === 0 && option.value !== null} aria-pressed={filters.content_type === option.value} onclick={() => applyFilter({ content_type: option.value })}>{t(option.label)}<span class="facet-count">{formatNumber(count)}</span></button>
            {/each}
          </div></section>
          <section><h2>{t("filter.source")}</h2>
            <input class="source-search" value={sourceQuery} oninput={(event) => { sourceQuery = event.currentTarget.value; onsourcequery(sourceQuery); }} aria-label={t("filter.searchSources")} placeholder={t("filter.searchSources")} />
            <div class="filter-options sources">
            <button type="button" aria-pressed={filters.source_id === null} onclick={() => applyFilter({ source_id: null })}>{t("filter.all")}</button>
            {#each sources as source}<button type="button" disabled={!source.available} aria-pressed={filters.source_id === source.id} onclick={() => applyFilter({ source_id: source.id })}>{source.name}</button>{/each}
          </div></section>
          <section><h2>{t("filter.time")}</h2><div class="filter-options">
            {#each timeRanges as option}<button type="button" aria-pressed={filters.time_range === option.value} onclick={() => applyFilter({ time_range: option.value })}>{t(option.label)}</button>{/each}
          </div></section>
          {#if activeFilterCount}<button type="button" class="clear-filters" onclick={() => applyFilter({ content_type: null, source_id: null, time_range: "any" })}>{t("filter.clear")}</button>{/if}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  </form>
  <div bind:this={listbox} class:full={page.items.length > 0} class="list" role="listbox" aria-label={t("history.list")} aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined} onpointerdown={() => listbox.focus()} onfocus={onlistfocus} onkeydown={onkeydown}>
    {#if loading && page.items.length === 0}
      <div bind:this={emptyAnchor} class="empty" tabindex="-1">{t("history.loading")}</div>
    {:else if error && page.items.length === 0}
      <button bind:this={retryButton} class="empty retry" onclick={() => onpage(1)}>{t("history.retry")}</button>
    {:else if page.items.length === 0}
      <div bind:this={emptyAnchor} class="empty-state" tabindex="-1">
        {#if query || activeFilterCount}
          <strong>{t("history.noMatchesTitle")}</strong>
          <span>{t("history.noMatchesHelp")}</span>
          <button type="button" onclick={onclearsearch}>{t("history.clearSearchConditions")}</button>
        {:else}
          <strong>{t("history.emptyTitle")}</strong>
          <span>{t("history.emptyHelp")}</span>
        {/if}
      </div>
    {:else}
      {#each page.items as item, index (item.id)}
        <div class:expanded={canExpand(item) && expandedId === item.id} class="clip-item" animate:flip={{ duration: reducedMotion || !rowReorderMotion ? 0 : 180, easing: cubicOut }} out:fade={{ duration: reducedMotion || !rowReorderMotion ? 0 : 90 }}>
          <div id={`clip-${item.id}`} class:selected={item.id === selectedId} class="row" role="option" tabindex="-1" aria-selected={item.id === selectedId} aria-posinset={(page.page - 1) * page.page_size + index + 1} aria-setsize={page.total} ondblclick={onpaste} onclick={() => onselect(item.id)} onkeydown={onkeydown}>
            <span class="num">{formatNumber(index === 9 ? 0 : index + 1)}</span>
            <span class:swatch={item.content_type === "color"} class:media={item.content_type === "image" || item.content_type === "file"} class="lead" style:background={item.content_type === "color" ? item.preview : undefined}>
              {#if thumbnailUrls[item.id]}<img src={thumbnailUrls[item.id]} decoding="async" alt="" />
              {:else if item.content_type === "image"}<span aria-hidden="true"><Image size={16} /></span>
              {:else if item.content_type === "file"}<File size={16} aria-hidden="true" />{/if}
            </span>
            {#if item.content_type !== "image"}<span class="snippet">{clipPreview(item, t("meta.file"))}</span>{/if}
            {#if canExpand(item)}<span class="disclosure" aria-hidden="true"><ChevronRight size={16} /></span>{/if}
          </div>
          {#if canExpand(item) && expandedId === item.id}
            <div class="row-details" role="group" in:fade={{ duration: reducedMotion ? 0 : 120 }} out:fade={{ duration: reducedMotion ? 0 : 90 }}>
              {#each groupedFiles(item) as path, index}
                <button tabindex="-1" class:selected={index === fileIndex} class="row-child" onclick={(event) => { event.stopPropagation(); onfile(index); }}>{fileName(path, t("meta.file"))}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
      {#each Array(page.page_size - page.items.length) as _}
        <div class="list-slot" aria-hidden="true"></div>
      {/each}
    {/if}
  </div>
</section>

<footer class="pager">
  <button disabled={currentPage() <= 1} onclick={() => turnPage(-1)} aria-label={t("history.previousPage")}><ArrowLeft size={16} aria-hidden="true" /></button>
  <!-- svelte-ignore a11y_no_static_element_interactions Pointer-only enhancement; keyboard pagination stays on the history list. -->
  <div class="scrubber" class:dragging={draggingPage} onwheel={onScrubberWheel} onpointerdown={startPageDrag} onpointermove={movePageDrag} onpointerup={stopPageDrag} onpointercancel={stopPageDrag}>
    <span class="ticks" class:key-visible={keyTicksVisible} class:key-motion={keyDirection !== 0} aria-hidden="true">
      {#each visiblePageTicks(currentPage(), page.total_pages) as tick}<i style={tickStyle(tick)}></i>{/each}
    </span>
  </div>
  <span class="page-count">{formatNumber(page.total_pages === 0 ? 0 : currentPage())}/{formatNumber(page.total_pages)}</span>
  <button disabled={currentPage() >= page.total_pages} onclick={() => turnPage(1)} aria-label={t("history.nextPage")}><ArrowRight size={16} aria-hidden="true" /></button>
</footer>

<style>
  .left { grid-column:1; grid-row:2; min-height:0; display:flex; flex-direction:column; border-right:1px solid var(--hairline); }
  .search { height:42px; flex:none; display:flex; align-items:center; gap:8px; padding:0 14px; color:var(--text-3); border-bottom:1px solid var(--hairline); }
  .search input { min-width:0; flex:1; border:0; outline:0; padding:0; color:var(--text-1); background:transparent; font-size:var(--fs-body); }
  .search input::placeholder { color:var(--text-2); }
  :global(.filter-trigger) { min-width:30px; height:28px; flex:none; display:flex; align-items:center; justify-content:center; gap:4px; padding:0 5px; border-radius:var(--radius-sm); color:var(--text-2); background:transparent; font:var(--fs-caption)/1 var(--mono); }
  :global(.filter-trigger:hover),:global(.filter-trigger.active) { color:var(--text-1); background:var(--bg-hover); }
  :global(.filter-trigger:focus-visible) { outline:2px solid var(--text-1); outline-offset:2px; }
  :global(.filter-trigger kbd) { pointer-events:none; }
  :global(.filter-popover) { z-index:var(--z-menu); width:280px; padding:10px; border:1px solid var(--hairline); border-radius:var(--radius-lg); color:var(--text-1); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  :global(.filter-popover section+section) { margin-top:12px; }
  :global(.filter-popover h2) { margin:0 0 6px; color:var(--text-2); font:600 var(--fs-ui)/var(--lh-snug) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  :global(.filter-options) { display:flex; flex-wrap:wrap; gap:4px; }
  :global(.type-options) { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); }
  :global(.type-options button) { min-width:0; display:flex; align-items:center; justify-content:space-between; gap:4px; }
  :global(.filter-options.sources) { max-height:116px; overflow:auto; }
  :global(.source-search) { width:100%; height:30px; margin-bottom:6px; padding:0 8px; border:1px solid var(--hairline); border-radius:var(--radius-md); outline:0; color:var(--text-1); background:var(--bg-shell); font:var(--fs-ui)/var(--lh-snug) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  :global(.source-search::placeholder) { color:var(--text-2); }
  :global(.source-search:focus-visible) { outline:2px solid var(--text-1); outline-offset:1px; }
  :global(.filter-options button),:global(.clear-filters) { min-height:28px; padding:4px 8px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; font:600 var(--fs-ui)/var(--lh-snug) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  :global(.filter-options button:hover),:global(.clear-filters:hover) { color:var(--text-1); background:var(--bg-hover); }
  :global(.filter-options button[aria-pressed="true"]) { color:var(--text-1); background:var(--bg-selected); }
  :global(.filter-options button:disabled) { color:var(--text-3); opacity:.45; cursor:not-allowed; }
  :global(.facet-count) { min-width:2.2ch; flex:none; padding:1px 4px; border-radius:var(--radius-pill); color:var(--text-3); background:var(--bg-shell); font:var(--fs-caption)/var(--lh-tight) var(--mono); font-variant-numeric:tabular-nums; text-align:center; white-space:nowrap; }
  :global(.filter-options button[aria-pressed="true"] .facet-count) { color:var(--text-2); background:var(--bg-raised); }
  :global(.filter-options button:focus-visible),:global(.clear-filters:focus-visible) { outline:2px solid var(--text-1); outline-offset:1px; }
  :global(.clear-filters) { width:100%; margin-top:10px; border-top:1px solid var(--hairline); border-radius:0 0 var(--radius-md) var(--radius-md); }
  kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:var(--radius-sm); padding:1px 5px; white-space:nowrap; }
  .list { flex:1; min-height:0; display:flex; flex-direction:column; gap:1px; padding:6px; overflow-y:auto; }
  .list:focus-visible { outline:none; }
  .empty-state { flex:1; min-height:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:6px; padding:24px; color:var(--text-2); text-align:center; }
  .empty-state strong { color:var(--text-1); font:600 var(--fs-body)/var(--lh-snug) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  .empty-state span { max-width:28ch; font:var(--fs-ui)/var(--lh-normal) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  .empty-state button { min-height:30px; margin-top:4px; padding:0 10px; border-radius:var(--radius-md); color:var(--text-1); background:var(--bg-hover); font:600 var(--fs-ui)/var(--lh-snug) -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif; }
  .empty-state button:hover { background:var(--bg-selected); }
  .empty-state button:focus-visible { outline:2px solid var(--text-1); outline-offset:2px; }
  .clip-item { width:100%; }
  .list.full .clip-item:not(.expanded), .list-slot { flex:1 0 44px; }
  .list.full .clip-item:not(.expanded) .row { height:100%; }
  .row { width:100%; min-height:44px; display:flex; align-items:center; gap:8px; padding:7px 8px; border-radius:var(--radius-lg); color:var(--text-1); background:transparent; text-align:left; cursor:default; }
  .row:hover { background:var(--bg-hover); }
  .list:focus .row.selected { background:var(--bg-selected); }
  .row.selected { background:color-mix(in srgb, var(--bg-selected) 55%, transparent); }
  .num { width:16px; flex:none; color:var(--text-3); font-size:var(--fs-ui); font-weight:650; line-height:var(--lh-flush); font-variant-numeric:tabular-nums lining-nums; text-align:center; }
  .list:focus .row.selected .num { color:var(--text-2); }
  .lead { width:28px; height:28px; flex:none; display:flex; align-items:center; justify-content:center; border-radius:var(--radius-sm); color:var(--text-2); font:7px var(--mono); }
  .lead.swatch { color:transparent; border:1px solid var(--hairline); }
  .lead.media { overflow:hidden; background:var(--bg-raised); }
  .lead.media img { width:100%; height:100%; object-fit:cover; }
  .snippet { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font:var(--fs-body)/var(--lh-normal) var(--mono); }
  .disclosure { width:16px; flex:none; display:flex; justify-content:center; color:var(--text-3); transition:transform var(--dur-mid) var(--ease-out); }
  .clip-item.expanded .disclosure { transform:rotate(90deg); }
  .row-details { margin:0 8px 4px 50px; padding:3px 8px 7px; }
  .row-child { width:100%; overflow:hidden; padding:4px 6px; border-radius:var(--radius-sm); color:var(--text-2); background:transparent; font:var(--fs-meta)/var(--lh-snug) var(--mono); text-align:left; text-overflow:ellipsis; white-space:nowrap; }
  .row-child:hover, .row-child.selected { background:var(--bg-hover); color:var(--text-1); }
  .pager { grid-column:1; grid-row:3; display:grid; grid-template-columns:36px minmax(70px,1fr) auto 36px; align-items:center; gap:16px; padding:0 14px; border-top:1px solid var(--hairline); border-right:1px solid var(--hairline); color:var(--text-2); font:var(--fs-ui) var(--mono); }
  .scrubber { height:30px; min-width:70px; position:relative; overflow:hidden; border-radius:var(--radius-sm); cursor:grab; touch-action:none; user-select:none; -webkit-user-select:none; }
  .scrubber.dragging { cursor:grabbing; }
  .ticks { position:absolute; inset:0; opacity:0; pointer-events:none; transition:opacity 220ms ease-out; }
  .scrubber:hover .ticks,.scrubber.dragging .ticks,.ticks.key-visible { opacity:1; }
  .ticks i { width:2px; position:absolute; left:50%; top:50%; margin-left:-1px; border-radius:var(--radius-pill); background:var(--text-2); transform-origin:center; transition:none; }
  .ticks.key-motion i { transition:height var(--dur-fast) var(--ease-out),opacity var(--dur-fast) ease-out,transform var(--dur-fast) var(--ease-out); }
  .page-count { min-width:34px; text-align:right; font-variant-numeric:tabular-nums; white-space:nowrap; }
  .pager button { width:36px; height:30px; display:grid; place-items:center; padding:0; border:1px solid var(--hairline); border-radius:var(--radius-sm); color:var(--text-2); background:transparent; }
  .pager button:hover:not(:disabled) { background:var(--bg-hover); }
  .pager button:disabled { color:var(--text-3); opacity:.35; cursor:default; }
  @media(prefers-reduced-motion:reduce){.ticks,.ticks.key-motion i{transition:none}}
  :global(html.pager-dragging),:global(html.pager-dragging *){cursor:grabbing!important}
</style>
