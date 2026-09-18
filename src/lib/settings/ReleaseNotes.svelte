<script lang="ts">
  import { LoaderCircle, RefreshCw } from "@lucide/svelte";
  import { onDestroy, untrack } from "svelte";
  import { formatDateTime, localizedError, t } from "$lib/i18n/index.svelte";
  import { listReleaseNotes, openLatestRelease, type ReleaseNote } from "$lib/updater/api";

  let { updateVersion, refreshRevision = 0, onerror }: { updateVersion?: string; refreshRevision?: number; onerror: (message: string) => void } = $props();
  let releases = $state<ReleaseNote[]>([]);
  let selected = $state<ReleaseNote | null>(null);
  let loading = $state(false);
  let error = $state("");
  let list = $state<HTMLDivElement>();
  let page = 0;
  let hasMore = $state(false);
  let requestId = 0;
  let initialized = false;
  let failedPage = 1;
  onDestroy(() => { requestId++; });

  $effect(() => {
    refreshRevision;
    untrack(() => { void load(1, initialized); initialized = true; });
  });
  $effect(() => { const version = updateVersion; untrack(() => { const release = forVersion(version); if (release) selected = release; }); });

  function forVersion(version: string | undefined) {
    if (!version) return undefined;
    const normalized = version.replace(/^v/i, "");
    return releases.find((release) => release.version.replace(/^v/i, "") === normalized);
  }
  async function load(nextPage = 1, refresh = false) {
    const request = ++requestId;
    loading = true; error = ""; failedPage = nextPage;
    try {
      const result = await listReleaseNotes(nextPage, refresh);
      if (request !== requestId) return;
      const previous = selected?.version;
      releases = nextPage === 1 ? result.releases : [...releases, ...result.releases.filter((item) => !releases.some((old) => old.version === item.version))];
      page = nextPage; hasMore = result.hasMore;
      selected = releases.find((item) => item.version === previous) ?? forVersion(updateVersion) ?? releases[0] ?? null;
    }
    catch (reason) { if (request === requestId) error = t(typeof reason === "object" && reason && "code" in reason && reason.code === "RELEASE_RATE_LIMITED" ? "settings.releaseRateLimited" : "settings.releaseUnavailable"); }
    finally { if (request === requestId) loading = false; }
  }
  function loadMore() {
    if (!loading && hasMore && !error) void load(page + 1);
  }
  function onScroll() {
    if (list && list.scrollHeight - list.scrollTop - list.clientHeight < 72) loadMore();
  }
  function select(index: number) {
    const release = releases[index]; if (!release) return; selected = release;
    requestAnimationFrame(() => list?.querySelector<HTMLElement>(`[data-release-index="${index}"]`)?.scrollIntoView({ block: "nearest" }));
  }
  function click(event: MouseEvent) {
    const option = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-release-index]") : null;
    if (option) select(Number(option.dataset.releaseIndex));
  }
  function keydown(event: KeyboardEvent) {
    const current = Math.max(0, releases.findIndex((release) => release.version === selected?.version));
    const page = Math.max(1, Math.floor((list?.clientHeight ?? 36) / 36));
    let next = current;
    if (event.key === "ArrowDown") next++; else if (event.key === "ArrowUp") next--;
    else if (event.key === "PageDown") next += page; else if (event.key === "PageUp") next -= page;
    else if (event.key === "Home") next = 0; else if (event.key === "End") next = releases.length - 1; else return;
    event.preventDefault(); select(Math.min(Math.max(next, 0), releases.length - 1));
    if (next >= releases.length - 1) loadMore();
  }
  async function openPage() {
    try { await openLatestRelease(); } catch (reason) { onerror(localizedError(reason)); }
  }
</script>

<section class="release-history" aria-label={t("settings.releaseNotes")}>
  <h2>{t("settings.versionHistory")}</h2>
  {#if !releases.length && (loading || error)}
    <div class="release-browser" class:release-loading={loading} aria-busy={loading} aria-label={loading ? t("settings.loadingReleases") : undefined}>
      <div class="release-list" aria-hidden="true">{#each Array(7) as _, index}<span class:active={index === 0} class="release-skeleton-row"><i></i><i></i></span>{/each}</div>
      {#if error}
        <div class="release-load-failed"><div class="release-load-message">
          <div role="alert"><h3>{t("settings.releaseLoadFailed")}</h3><p>{error}</p></div>
          <button class="release-retry" onclick={() => void load(1, true)}><RefreshCw size={15} aria-hidden="true" />{t("settings.retry")}</button>
        </div></div>
      {:else}
        <div class="release-detail" aria-hidden="true"><header><span><i class="release-skeleton title"></i><i class="release-skeleton date"></i></span></header><div class="release-skeleton-body"><i class="release-skeleton heading"></i><i class="release-skeleton line"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line"></i><i class="release-skeleton heading second"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line short"></i></div></div>
      {/if}
    </div>
  {:else if releases.length}<div class="release-browser"><div class="release-list-column"><div bind:this={list} class="release-list-scroll" onscroll={onScroll}><div class="release-list" role="listbox" aria-label={t("settings.releaseNotes")} aria-activedescendant={selected ? `release-option-${selected.version}` : undefined} tabindex="0" aria-busy={loading} onclick={click} onkeydown={keydown}>{#each releases as release, index}<div id={`release-option-${release.version}`} class="release-option" class:active={selected?.version === release.version} data-release-index={index} role="option" aria-selected={selected?.version === release.version}><strong>{release.version}</strong><small>{formatDateTime(release.publishedAt)}</small></div>{/each}</div>
      {#if loading}<div class="load-more-status" role="status"><LoaderCircle size={16} aria-hidden="true" /><span class="loading-label">{t("settings.loadingReleases")}</span></div>
      {:else if error}<div class="list-status"><p class="inline-error" role="alert">{error}</p><button onclick={() => void load(failedPage, failedPage === 1)}>{t("settings.retry")}</button></div>
      {/if}</div>
    </div>{#if selected}<article class="release-detail"><header><span><span class="release-detail-title"><strong>{selected.version}</strong>{#if selected.isLatest}<em>{t("settings.latestRelease")}</em>{/if}</span><small>{formatDateTime(selected.publishedAt)}</small></span>{#if selected.isLatest}<button class="release-page" onclick={() => void openPage()}>{t("settings.releasePage")} ↗</button>{/if}</header><div class="release-body" class:raw-release-body={!selected.notesHtml}>{#if selected.notesHtml}{@html selected.notesHtml}{:else}{selected.notes}{/if}</div></article>{/if}</div>{:else}<p class="list-status">{t("settings.noReleases")}</p>{/if}
</section>

<style>
  h2{margin:0 0 10px;color:var(--text-1);font-size:var(--fs-body);font-weight:700}
  .release-list-column{display:flex;flex-direction:column;min-height:0;min-width:0}
  .release-list-scroll{min-height:0;overflow-y:auto;padding:4px 8px 0 0;border-right:1px solid var(--hairline)}
  .list-status{margin:0;padding:8px;font-size:var(--fs-ui);color:var(--text-2);overflow-wrap:anywhere}
  .load-more-status{display:grid;place-items:center;min-height:32px;padding:4px 0 8px;color:var(--text-2)}
  .load-more-status :global(svg){animation:release-spin 1s linear infinite}
  .loading-label{position:absolute;width:1px;height:1px;padding:0;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
  @keyframes release-spin{to{transform:rotate(360deg)}}
  @media(prefers-reduced-motion:reduce){.load-more-status :global(svg){animation:none}.loading-label{position:static;width:auto;height:auto;clip-path:none;font-size:var(--fs-meta)}.load-more-status :global(svg){display:none}}

  .release-history{flex:1 1 auto;min-height:0;margin-top:16px;overflow:hidden;display:grid;grid-template-rows:auto minmax(0,1fr)}.release-browser{height:100%;min-height:0;overflow:hidden;display:grid;grid-template-columns:180px minmax(0,1fr)}.release-list{min-height:0}.release-list-scroll,.release-body{scrollbar-width:thin;scrollbar-color:color-mix(in srgb,var(--text-3) 52%,transparent) transparent}.release-list-scroll::-webkit-scrollbar,.release-body::-webkit-scrollbar{width:9px}.release-list-scroll::-webkit-scrollbar-track,.release-body::-webkit-scrollbar-track{background:transparent}.release-list-scroll::-webkit-scrollbar-thumb,.release-body::-webkit-scrollbar-thumb{border:3px solid transparent;border-radius:var(--radius-pill);background:color-mix(in srgb,var(--text-3) 52%,transparent);background-clip:padding-box}.release-list-scroll::-webkit-scrollbar-thumb:hover,.release-body::-webkit-scrollbar-thumb:hover{background-color:var(--text-3)}.release-option{display:flex;min-height:36px;padding:4px 8px;align-items:center;justify-content:space-between;gap:8px;border-radius:var(--radius-lg);cursor:pointer}.release-option strong,.release-option small{white-space:nowrap}.release-option small{color:var(--text-3);font-size:var(--fs-ui);font-variant-numeric:tabular-nums}.release-option.active{color:var(--text-1);background:var(--bg-selected)}.release-list:focus-visible{outline:none}.release-list:focus-visible .release-option.active{outline:2px solid var(--text-1);outline-offset:-2px}.release-detail{min-width:0;min-height:0;display:grid;grid-template-rows:auto minmax(0,1fr);padding:10px 0 0 14px}.release-detail header{display:flex;align-items:center;justify-content:space-between;gap:12px}.release-detail header>span{display:flex;min-width:0;flex-direction:column;gap:2px}.release-detail-title{display:flex;align-items:center;gap:6px}.release-detail-title em{padding:1px 4px;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-hover);font-size:var(--fs-meta);font-style:normal;font-weight:400;line-height:1.3;white-space:nowrap}.release-page{padding:4px 6px;color:var(--text-3);white-space:nowrap}.release-body{min-height:0;margin-top:8px;padding:10px 12px;overflow-y:auto;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-shell);font-size:var(--fs-ui);line-height:1.55}.release-body.raw-release-body{white-space:pre-wrap}.release-body :global(h2),.release-body :global(h3){margin:0 0 8px;color:var(--text-1);font-size:var(--fs-body);line-height:1.35}.release-body :global(h2:not(:first-child)),.release-body :global(h3:not(:first-child)){margin-top:18px}.release-body :global(p),.release-body :global(ul),.release-body :global(blockquote){margin:0 0 12px}.release-body :global(ul){padding-left:20px}.release-body :global(li+li){margin-top:4px}.release-body :global(blockquote){padding:8px 10px;border-left:2px solid var(--hairline);border-radius:0 var(--radius-sm) var(--radius-sm) 0;background:var(--bg-raised)}.release-body :global(a){color:var(--action);text-decoration:underline;text-underline-offset:2px}.release-loading .release-detail{padding-right:14px}.release-skeleton,.release-skeleton-row i{display:block;border-radius:var(--radius-sm);background:color-mix(in srgb,var(--text-3) 14%,transparent)}.release-skeleton-row{min-height:36px;margin-bottom:4px;padding:7px 8px;display:flex;align-items:center;justify-content:space-between;gap:12px;border-radius:var(--radius-lg)}.release-skeleton-row.active{background:color-mix(in srgb,var(--bg-selected) 55%,transparent)}.release-skeleton-row i:first-child{width:52px;height:14px}.release-skeleton-row i:last-child{width:70px;height:12px}.release-loading .release-detail header{min-height:54px;padding-top:10px}.release-loading .release-detail header>span{gap:6px}.release-skeleton.title{width:72px;height:16px}.release-skeleton.date{width:92px;height:11px}.release-skeleton-body{display:flex;flex-direction:column;gap:10px;padding:12px;border-radius:var(--radius-md);background:color-mix(in srgb,var(--bg-raised) 55%,transparent)}.release-skeleton.heading{width:64px;height:15px;margin-bottom:3px}.release-skeleton.heading.second{margin-top:12px}.release-skeleton.line{width:76%;height:10px}.release-skeleton.line.wide{width:91%}.release-skeleton.line.short{width:58%}.release-load-failed{min-height:0;overflow:auto;display:grid;place-items:center;padding:24px}.release-load-message{max-width:360px;display:flex;flex-direction:column;align-items:center;gap:16px;text-align:center}.release-load-message h3{margin:0 0 8px;color:var(--text-1);font-size:var(--fs-body);font-weight:600}.release-load-message p{margin:0;color:var(--text-2);font-size:var(--fs-ui);line-height:1.6;overflow-wrap:anywhere}.release-load-message .release-retry{display:inline-flex;align-items:center;justify-content:center;gap:6px;min-width:92px;flex-shrink:0;white-space:nowrap;border:1px solid var(--hairline);background:var(--bg-raised)}.inline-error{margin:0;color:var(--danger);font-size:var(--fs-ui)}button{min-height:32px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
</style>
