<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { canPreviewClip, getClipThumbnail, getPreviewCapability, hidePanel, pasteClip, previewClip, queryHistory, setQuickSelection, type PreviewCapability } from "./api";
  import { fileName } from "./presentation";
  import type { ClipSummary, HistoryPage } from "./types";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import { quitApp } from "$lib/settings/api";
  import { currentPlatform } from "$lib/settings/shortcuts";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import { routeQuickKey } from "./quick-keyboard";
  import PageScrubber from "./PageScrubber.svelte";

  let { selectedId = $bindable<string | null>(null), onfull, onsettings }: {
    selectedId?: string | null;
    onfull: () => void;
    onsettings: () => void;
  } = $props();
  const slotCount = Math.max(1, Math.min(10, Math.floor((window.innerHeight - 188) / 40)));
  const platform = currentPlatform();
  let page = $state<HistoryPage>({ items: [], page: 1, page_size: slotCount, total: 0, total_pages: 1 });
  let thumbnails = $state<Record<string, string>>({});
  let loading = $state(true);
  let error = $state("");
  let previewCapability = $state<PreviewCapability>({ provider: "unavailable", reason: "detection_failed" });
  let list = $state<HTMLDivElement>();
  let pageScrubber: PageScrubber;
  let scrubberPage = $state<number | null>(null);
  let reducedMotion = $state(false);
  let stepEdge: "first" | "last" | null = null;
  let requestVersion = 0;
  const items = $derived(page.items);
  const currentPage = $derived(page.page || 1);
  const displayPage = $derived(scrubberPage ?? currentPage);
  const totalPages = $derived(Math.max(1, page.total_pages));

  $effect(() => {
    void setQuickSelection(selectedId);
  });

  onMount(() => {
    const motionQuery = window.matchMedia?.("(prefers-reduced-motion: reduce)");
    const updateReducedMotion = () => { reducedMotion = motionQuery?.matches ?? false; };
    updateReducedMotion();
    motionQuery?.addEventListener("change", updateReducedMotion);
    void loadPage(1, "first");
    void refreshPreviewCapability();
    const unlisten = listen("history_changed", () => void loadPage(1, "first"));
    const unlistenShown = listen("quick_panel_shown", () => {
      void loadPage(1, "first");
      void refreshPreviewCapability();
      requestAnimationFrame(() => list?.focus());
    });
    requestAnimationFrame(() => list?.focus());
    return () => {
      motionQuery?.removeEventListener("change", updateReducedMotion);
      unlisten.then((fn) => fn());
      unlistenShown.then((fn) => fn());
    };
  });

  async function refreshPreviewCapability() {
    try { previewCapability = await getPreviewCapability(); }
    catch { previewCapability = { provider: "unavailable", reason: "detection_failed" }; }
  }

  async function loadPage(target: number, edge: "first" | "last") {
    const version = ++requestVersion;
    loading = true;
    try {
      const result = await queryHistory("", target, undefined, slotCount);
      if (version !== requestVersion) return;
      page = result;
      selectedId = (edge === "last" ? result.items.at(-1) : result.items[0])?.id ?? null;
      pageScrubber?.reset();
      thumbnails = {};
      for (const item of result.items) {
        if (item.content_type !== "image") continue;
        void getClipThumbnail(item.id).then(({ data_url }) => {
          if (data_url && version === requestVersion) thumbnails = { ...thumbnails, [item.id]: data_url };
        });
      }
      error = "";
    } catch (reason) {
      if (version === requestVersion) {
        pageScrubber?.reset();
        error = localizedError(reason);
      }
    } finally {
      if (version === requestVersion) loading = false;
    }
  }

  function turnPage(direction: -1 | 1) {
    if (loading) return;
    stepEdge = direction < 0 ? "last" : "first";
    pageScrubber?.turnPage(direction);
  }

  function onScrubberPage(target: number) {
    const edge = stepEdge ?? "first";
    stepEdge = null;
    void loadPage(target, edge);
  }

  function summary(item: ClipSummary) {
    if (item.content_type === "file") {
      const files = item.metadata.files ?? [];
      const name = fileName(files[0] ?? item.preview, t("meta.file"));
      return files.length > 1 ? t("quick.moreFiles", { name, count: files.length - 1 }) : name;
    }
    if (item.content_type === "link") return item.preview.replace(/^https?:\/\//, "");
    return item.preview.replace(/\s+/g, " ").trim();
  }

  async function copy(item: ClipSummary | undefined) {
    if (!item) return;
    selectedId = item.id;
    try {
      await pasteClip(item.id);
    } catch (reason) {
      error = localizedError(reason) || t("quick.copyFailed");
    }
  }

  function onkeydown(event: KeyboardEvent) {
    const index = items.findIndex((item) => item.id === selectedId);
    const selected = items[Math.max(index, 0)];
    const canPreview = canPreviewClip(previewCapability, selected?.content_type);
    const action = routeQuickKey(event.key, index, items.length, canPreview, currentPage, totalPages);
    if (!action) return;
    event.preventDefault();
    if (action.type === "close") void hidePanel();
    else if (action.type === "copy") void copy(items[action.index]);
    else if (action.type === "preview") void preview(items[action.index]);
    else if (action.type === "page") {
      turnPage(action.page < currentPage ? -1 : 1);
    } else selectedId = items[action.index]?.id ?? null;
  }

  function selectFromList(id: string) {
    list?.focus();
    selectedId = id;
  }

  function clearWindowFocus() {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
  }

  async function preview(item: ClipSummary | undefined) {
    if (!item) return;
    selectedId = item.id;
    try {
      await previewClip(item.id);
    } catch (reason) {
      error = localizedError(reason);
    }
  }
</script>

<svelte:window onblur={clearWindowFocus} />

<main class="quick-shell">
  <section class="quick-panel" aria-label={t("quick.title")}>
    <header>
      <span class="brand"><img src="/app-icon.png" alt="" /><strong>{t("quick.title")}</strong></span>
      <span class="navigation">
        <PageScrubber bind:this={pageScrubber} bind:displayPage={scrubberPage} page={currentPage} totalPages={totalPages} {reducedMotion} disabled={totalPages <= 1} onpage={onScrubberPage} />
        <button aria-label={t("quick.previousPage")} disabled={loading || currentPage <= 1} onclick={() => turnPage(-1)}><span aria-hidden="true">‹</span></button>
        <span class="page-status" aria-live="polite">{t("quick.pageStatus", { current: displayPage, total: totalPages })}</span>
        <button aria-label={t("quick.nextPage")} disabled={loading || currentPage >= totalPages} onclick={() => turnPage(1)}><span aria-hidden="true">›</span></button>
      </span>
    </header>
    <div bind:this={list} class="quick-list" style:grid-template-rows={`repeat(${slotCount},minmax(0,1fr))`} role="listbox" aria-label={t("quick.list")} aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `quick-${selectedId}` : undefined} onkeydown={onkeydown}>
      {#if loading && items.length === 0}
        {#each Array(slotCount) as _}<span class="skeleton" aria-hidden="true"></span>{/each}
      {:else if error && items.length === 0}
        <button class="state" onclick={() => void loadPage(1, "first")}>{t("quick.loadFailed")}</button>
      {:else if items.length === 0}
        <span class="state">{t("quick.empty")}</span>
      {:else}
        {#each Array(slotCount) as _, index}
          {@const item = items[index]}
          {#if item}
            <button id={`quick-${item.id}`} class:selected={item.id === selectedId} class:image={item.content_type === "image"} class="quick-item" role="option" tabindex="-1" aria-selected={item.id === selectedId} onclick={() => selectFromList(item.id)} ondblclick={() => void copy(item)}>
              <span class="number">{index === 9 ? 0 : index + 1}</span>
              {#if item.content_type === "image"}
                <span class="image-thumb">{#if thumbnails[item.id]}<img src={thumbnails[item.id]} alt="" />{/if}</span>
              {:else if item.content_type === "color"}
                <span class="swatch" style:background={item.preview}></span><span class="summary">{summary(item)}</span>
              {:else}
                <span class="summary">{summary(item)}</span>
              {/if}
            </button>
            {:else}
            <span class="empty-slot" aria-hidden="true"></span>
          {/if}
        {/each}
      {/if}
    </div>
    {#if error && items.length > 0}<p class="inline-error" role="alert">{error}</p>{/if}
    <nav aria-label={t("quick.title")}>
      <button onclick={onfull}><span>{t("quick.openHistory")}</span></button>
      <button onclick={onsettings}><span>{t("history.settings")}</span><ShortcutHint shortcut={platform === "macos" ? "Command+," : "Ctrl+,"} {platform} /></button>
      <span class="separator"></span>
      <button class="danger" onclick={() => void quitApp()}><span>{t("history.quit")} ClipClop</span><ShortcutHint shortcut={platform === "macos" ? "Command+Q" : "Ctrl+Q"} {platform} /></button>
    </nav>
  </section>
</main>

<style>
  .quick-shell{width:100%;height:100%;padding:10px;display:flex}
  .quick-panel{width:100%;min-width:0;min-height:0;display:flex;flex-direction:column;overflow:hidden;border-radius:var(--radius-xl);background:var(--bg-raised);box-shadow:var(--panel-shadow)}
  header{height:40px;flex:none;display:flex;align-items:center;justify-content:space-between;padding:0 8px 0 12px;border-bottom:1px solid var(--hairline);user-select:none}
  .brand{flex:none;display:flex;align-items:center;gap:7px}
  header img{width:20px;height:20px;border-radius:var(--radius-sm)}
  header strong{font-size:var(--fs-ui);font-weight:650}
  .navigation{width:198px;flex:none;display:grid;grid-template-columns:70px 26px 64px 26px;align-items:center;gap:4px;color:var(--text-3)}
  .navigation>button{width:26px;height:26px;display:grid;place-items:center;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font:500 18px/1 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}
  .navigation>button:hover:not(:disabled),.navigation>button:focus-visible{color:var(--text-1);background:var(--bg-hover)}
  .navigation>button:disabled{opacity:.4}
  .page-status{width:64px;overflow:hidden;color:var(--text-3);font:500 var(--fs-meta)/1 var(--mono);font-variant-numeric:tabular-nums;text-align:center;white-space:nowrap}
  .quick-list{width:100%;min-width:0;min-height:40px;flex:1;display:grid;overflow:hidden;padding:4px 6px;outline:0}
  .quick-item{width:auto;min-width:0;max-width:100%;min-height:0;overflow:hidden;display:flex;align-items:center;gap:8px;padding:4px 8px;border-radius:var(--radius-lg);color:var(--text-1);background:transparent;text-align:left;font-weight:500}
  .quick-item:hover{background:var(--bg-hover)}
  .quick-item.selected{background:color-mix(in srgb,var(--bg-selected) 55%,transparent)}
  .quick-list:focus .quick-item.selected{background:var(--bg-selected)}
  .number{width:16px;flex:none;color:var(--text-3);font:650 var(--fs-ui)/1 var(--mono);text-align:center}
  .summary{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:var(--fs-body);font-weight:500;line-height:var(--lh-normal)}
  .swatch{width:24px;height:24px;flex:none;border:1px solid var(--hairline);border-radius:var(--radius-sm)}
  .image-thumb{width:64px;height:36px;overflow:hidden;border-radius:var(--radius-sm);background:var(--bg-hover)}
  .image-thumb img{width:100%;height:100%;object-fit:cover}
  .state{width:100%;grid-row:1/-1;display:grid;place-items:center;color:var(--text-2);background:transparent;font-size:var(--fs-ui)}
  .skeleton{display:block;margin:2px 0;border-radius:var(--radius-lg);background:color-mix(in srgb,var(--text-3) 12%,transparent)}
  .inline-error{margin:0;padding:6px 12px;color:var(--danger);font-size:var(--fs-meta)}
  nav{flex:none;padding:5px 6px 6px;border-top:1px solid var(--hairline)}
  nav button{width:100%;height:32px;display:flex;align-items:center;justify-content:space-between;padding:0 8px;border-radius:var(--radius-md);color:var(--text-1);background:transparent;text-align:left;font-size:var(--fs-ui);font-weight:500}
  nav button:hover,nav button:focus-visible{background:var(--bg-hover)}
  nav button.danger{color:var(--danger)}
  .separator{display:block;height:1px;margin:5px 6px;background:var(--hairline)}
  @media(prefers-reduced-motion:no-preference){.quick-panel{animation:appear var(--dur-fast) var(--ease-out)}}
  @keyframes appear{from{opacity:0;transform:translateY(2px)}to{opacity:1;transform:none}}
</style>
