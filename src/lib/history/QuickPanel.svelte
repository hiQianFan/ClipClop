<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getClipThumbnail, hidePanel, pasteClip, previewClip, queryHistory, setQuickSelection } from "./api";
  import { fileName } from "./presentation";
  import type { ClipSummary } from "./types";
  import { localizedError, t } from "$lib/i18n/index.svelte";
  import { quitApp } from "$lib/settings/api";
  import { routeQuickKey } from "./quick-keyboard";

  let { selectedId = $bindable<string | null>(null), onfull, onsettings }: {
    selectedId?: string | null;
    onfull: () => void;
    onsettings: () => void;
  } = $props();
  let items = $state<ClipSummary[]>([]);
  let thumbnails = $state<Record<string, string>>({});
  let loading = $state(true);
  let error = $state("");
  let list = $state<HTMLDivElement>();
  const visibleCount = $derived(Math.max(1, Math.min(10, Math.floor((window.innerHeight - 188) / 40))));
  const visibleItems = $derived(items.slice(0, visibleCount));

  $effect(() => {
    void setQuickSelection(selectedId);
  });

  onMount(() => {
    void refresh();
    const unlisten = listen("history_changed", () => void refresh());
    requestAnimationFrame(() => list?.focus());
    return () => { unlisten.then((fn) => fn()); };
  });

  async function refresh() {
    try {
      const page = await queryHistory("", 1);
      items = page.items;
      if (!items.some((item) => item.id === selectedId)) selectedId = items[0]?.id ?? null;
      for (const item of items) {
        if (item.content_type !== "image" || thumbnails[item.id]) continue;
        void getClipThumbnail(item.id).then(({ data_url }) => {
          if (data_url) thumbnails = { ...thumbnails, [item.id]: data_url };
        });
      }
      error = "";
    } catch (reason) {
      error = localizedError(reason);
    } finally {
      loading = false;
    }
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
    const index = visibleItems.findIndex((item) => item.id === selectedId);
    const action = routeQuickKey(event.key, index, visibleItems.length);
    if (!action) return;
    event.preventDefault();
    if (action.type === "close") void hidePanel();
    else if (action.type === "copy") void copy(visibleItems[action.index]);
    else if (action.type === "preview") void preview(visibleItems[action.index]);
    else selectedId = visibleItems[action.index]?.id ?? null;
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

<main class="quick-shell">
  <section class="quick-panel" aria-label={t("quick.title")}>
    <header><img src="/app-icon.png" alt="" /><strong>{t("quick.title")}</strong></header>
    <div bind:this={list} class="quick-list" role="listbox" aria-label={t("quick.list")} aria-busy={loading} tabindex="0" aria-activedescendant={selectedId ? `quick-${selectedId}` : undefined} onkeydown={onkeydown}>
      {#if loading && items.length === 0}
        {#each Array(3) as _}<span class="skeleton" aria-hidden="true"></span>{/each}
      {:else if error && items.length === 0}
        <button class="state" onclick={refresh}>{t("quick.loadFailed")}</button>
      {:else if items.length === 0}
        <span class="state">{t("quick.empty")}</span>
      {:else}
        {#each visibleItems as item, index (item.id)}
          <button id={`quick-${item.id}`} class:selected={item.id === selectedId} class:image={item.content_type === "image"} class="quick-item" role="option" aria-selected={item.id === selectedId} onclick={() => selectedId = item.id} ondblclick={() => void copy(item)}>
            <span class="number">{index === 9 ? 0 : index + 1}</span>
            {#if item.content_type === "image"}
              <span class="image-thumb">{#if thumbnails[item.id]}<img src={thumbnails[item.id]} alt="" />{/if}</span>
            {:else if item.content_type === "color"}
              <span class="swatch" style:background={item.preview}></span><span class="summary">{summary(item)}</span>
            {:else}
              <span class="summary">{summary(item)}</span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
    {#if error && items.length > 0}<p class="inline-error" role="alert">{error}</p>{/if}
    <nav aria-label={t("quick.title")}>
      <button onclick={onfull}><span>{t("quick.openHistory")}</span></button>
      <button onclick={onsettings}><span>{t("history.settings")}</span><kbd>{navigator.platform.includes("Mac") ? "⌘," : "Ctrl,"}</kbd></button>
      <span class="separator"></span>
      <button class="danger" onclick={() => void quitApp()}><span>{t("history.quit")} ClipClop</span><kbd>{navigator.platform.includes("Mac") ? "⌘Q" : "Ctrl+Q"}</kbd></button>
    </nav>
  </section>
</main>

<style>
  .quick-shell{width:100%;height:100%;padding:10px;display:flex}
  .quick-panel{width:100%;min-height:0;display:flex;flex-direction:column;overflow:hidden;border-radius:var(--radius-xl);background:var(--bg-raised);box-shadow:var(--panel-shadow)}
  header{height:40px;flex:none;display:flex;align-items:center;gap:7px;padding:0 12px;border-bottom:1px solid var(--hairline);user-select:none}
  header img{width:20px;height:20px;border-radius:var(--radius-sm)}
  header strong{font-size:var(--fs-ui);font-weight:650}
  .quick-list{min-height:40px;flex:1;overflow:hidden;padding:4px 6px;outline:0}
  .quick-item{width:100%;height:40px;display:flex;align-items:center;gap:8px;padding:4px 8px;border-radius:var(--radius-lg);color:var(--text-1);background:transparent;text-align:left;font-weight:500}
  .quick-item:hover{background:var(--bg-hover)}
  .quick-item.selected{background:color-mix(in srgb,var(--bg-selected) 55%,transparent)}
  .quick-list:focus .quick-item.selected{background:var(--bg-selected)}
  .number{width:16px;flex:none;color:var(--text-3);font:650 var(--fs-ui)/1 var(--mono);text-align:center}
  .summary{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:var(--fs-body);font-weight:500;line-height:var(--lh-normal)}
  .swatch{width:24px;height:24px;flex:none;border:1px solid var(--hairline);border-radius:var(--radius-sm)}
  .image-thumb{width:64px;height:36px;overflow:hidden;border-radius:var(--radius-sm);background:var(--bg-hover)}
  .image-thumb img{width:100%;height:100%;object-fit:cover}
  .state{height:100%;min-height:40px;width:100%;display:grid;place-items:center;color:var(--text-2);background:transparent;font-size:var(--fs-ui)}
  .skeleton{display:block;height:40px;margin-bottom:2px;border-radius:var(--radius-lg);background:color-mix(in srgb,var(--text-3) 12%,transparent)}
  .inline-error{margin:0;padding:6px 12px;color:var(--danger);font-size:var(--fs-meta)}
  nav{flex:none;padding:5px 6px 6px;border-top:1px solid var(--hairline)}
  nav button{width:100%;height:32px;display:flex;align-items:center;justify-content:space-between;padding:0 8px;border-radius:var(--radius-md);color:var(--text-1);background:transparent;text-align:left;font-size:var(--fs-ui);font-weight:500}
  nav button:hover,nav button:focus-visible{background:var(--bg-hover)}
  nav button.danger,nav button.danger kbd{color:var(--danger)}
  nav kbd{color:var(--text-2);font-family:inherit;font-size:var(--fs-body);font-weight:500;line-height:1}
  .separator{display:block;height:1px;margin:5px 6px;background:var(--hairline)}
  @media(prefers-reduced-motion:no-preference){.quick-panel{animation:appear var(--dur-fast) var(--ease-out)}}
  @keyframes appear{from{opacity:0;transform:translateY(2px)}to{opacity:1;transform:none}}
</style>
