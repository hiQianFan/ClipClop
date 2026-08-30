<script lang="ts">
  import { onMount } from "svelte";
  import { formatDateTime, localizedError, t } from "$lib/i18n/index.svelte";
  import { listReleaseNotes, openLatestRelease, type ReleaseNote } from "$lib/updater/api";

  let { updateVersion, onerror }: { updateVersion?: string; onerror: (message: string) => void } = $props();
  let releases = $state<ReleaseNote[]>([]);
  let selected = $state<ReleaseNote | null>(null);
  let loading = $state(false);
  let error = $state("");
  let list = $state<HTMLDivElement>();

  onMount(() => { void load(); });
  $effect(() => { const release = forVersion(updateVersion); if (release) selected = release; });

  function forVersion(version: string | undefined) {
    if (!version) return undefined;
    const normalized = version.replace(/^v/i, "");
    return releases.find((release) => release.version.replace(/^v/i, "") === normalized);
  }
  async function load() {
    loading = true; error = "";
    try { releases = await listReleaseNotes(); selected = forVersion(updateVersion) ?? releases[0] ?? null; }
    catch (reason) { error = localizedError(reason); }
    finally { loading = false; }
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
  }
  async function openPage() {
    try { await openLatestRelease(); } catch (reason) { onerror(localizedError(reason)); }
  }
</script>

<section class="release-history" aria-label={t("settings.releaseNotes")}>
  {#if loading}<div class="release-browser release-loading" aria-busy="true" aria-label={t("settings.loading")}><div class="release-list" aria-hidden="true">{#each Array(7) as _, index}<span class:active={index === 0} class="release-skeleton-row"><i></i><i></i></span>{/each}</div><div class="release-detail" aria-hidden="true"><header><span><i class="release-skeleton title"></i><i class="release-skeleton date"></i></span></header><div class="release-skeleton-body"><i class="release-skeleton heading"></i><i class="release-skeleton line"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line"></i><i class="release-skeleton heading second"></i><i class="release-skeleton line wide"></i><i class="release-skeleton line short"></i></div></div></div>
  {:else if error}<div class="release-browser release-load-failed"><div class="release-load-message"><p class="inline-error" role="alert">{error}</p><button onclick={() => void load()}>{t("settings.refreshReleaseNotes")}</button></div></div>
  {:else if releases.length}<div class="release-browser"><div bind:this={list} class="release-list" role="listbox" aria-label={t("settings.releaseNotes")} aria-activedescendant={selected ? `release-option-${selected.version}` : undefined} tabindex="0" onclick={click} onkeydown={keydown}>{#each releases as release, index}<div id={`release-option-${release.version}`} class="release-option" class:active={selected?.version === release.version} data-release-index={index} role="option" aria-selected={selected?.version === release.version}><strong>{release.version}</strong><small>{formatDateTime(release.publishedAt)}</small></div>{/each}</div>{#if selected}<article class="release-detail"><header><span><span class="release-detail-title"><strong>{selected.version}</strong>{#if selected.isLatest}<em>{t("settings.latestRelease")}</em>{/if}</span><small>{formatDateTime(selected.publishedAt)}</small></span>{#if selected.isLatest}<button class="release-page" onclick={() => void openPage()}>{t("settings.releasePage")} ↗</button>{/if}</header><div class="release-body" class:raw-release-body={!selected.notesHtml}>{#if selected.notesHtml}{@html selected.notesHtml}{:else}{selected.notes}{/if}</div></article>{/if}</div>{/if}
</section>

<style>
  .release-history{flex:1 1 auto;min-height:0;margin-top:16px;overflow:hidden;display:grid;grid-template-rows:minmax(0,1fr)}.release-browser{height:100%;min-height:0;overflow:hidden;display:grid;grid-template-columns:180px minmax(0,1fr)}.release-list{min-height:0;overflow-y:auto;padding:4px 8px 0 0;border-right:1px solid var(--hairline)}.release-list,.release-body{scrollbar-width:thin;scrollbar-color:color-mix(in srgb,var(--text-3) 52%,transparent) transparent}.release-list::-webkit-scrollbar,.release-body::-webkit-scrollbar{width:9px}.release-list::-webkit-scrollbar-track,.release-body::-webkit-scrollbar-track{background:transparent}.release-list::-webkit-scrollbar-thumb,.release-body::-webkit-scrollbar-thumb{border:3px solid transparent;border-radius:var(--radius-pill);background:color-mix(in srgb,var(--text-3) 52%,transparent);background-clip:padding-box}.release-list::-webkit-scrollbar-thumb:hover,.release-body::-webkit-scrollbar-thumb:hover{background-color:var(--text-3)}.release-option{display:flex;min-height:36px;padding:4px 8px;align-items:center;justify-content:space-between;gap:8px;border-radius:var(--radius-lg);cursor:pointer}.release-option strong,.release-option small{white-space:nowrap}.release-option small{color:var(--text-3);font-size:var(--fs-ui);font-variant-numeric:tabular-nums}.release-option.active{color:var(--text-1);background:var(--bg-selected)}.release-list:focus-visible{outline:none}.release-list:focus-visible .release-option.active{outline:2px solid var(--text-1);outline-offset:-2px}.release-detail{min-width:0;min-height:0;display:grid;grid-template-rows:auto minmax(0,1fr);padding:10px 0 0 14px}.release-detail header{display:flex;align-items:center;justify-content:space-between;gap:12px}.release-detail header>span{display:flex;min-width:0;flex-direction:column;gap:2px}.release-detail-title{display:flex;align-items:center;gap:6px}.release-detail-title em{padding:1px 4px;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-hover);font-size:var(--fs-meta);font-style:normal;font-weight:400;line-height:1.3;white-space:nowrap}.release-page{padding:4px 6px;color:var(--text-3);white-space:nowrap}.release-body{min-height:0;margin-top:8px;padding:10px 12px;overflow-y:auto;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-shell);font-size:var(--fs-ui);line-height:1.55}.release-body.raw-release-body{white-space:pre-wrap}.release-body :global(h2),.release-body :global(h3){margin:0 0 8px;color:var(--text-1);font-size:var(--fs-body);line-height:1.35}.release-body :global(h2:not(:first-child)),.release-body :global(h3:not(:first-child)){margin-top:18px}.release-body :global(p),.release-body :global(ul),.release-body :global(blockquote){margin:0 0 12px}.release-body :global(ul){padding-left:20px}.release-body :global(li+li){margin-top:4px}.release-body :global(blockquote){padding:8px 10px;border-left:2px solid var(--hairline);border-radius:0 var(--radius-sm) var(--radius-sm) 0;background:var(--bg-raised)}.release-body :global(a){color:var(--action);text-decoration:underline;text-underline-offset:2px}.release-loading .release-detail{padding-right:14px}.release-skeleton,.release-skeleton-row i{display:block;border-radius:var(--radius-sm);background:color-mix(in srgb,var(--text-3) 14%,transparent)}.release-skeleton-row{min-height:36px;margin-bottom:4px;padding:7px 8px;display:flex;align-items:center;justify-content:space-between;gap:12px;border-radius:var(--radius-lg)}.release-skeleton-row.active{background:color-mix(in srgb,var(--bg-selected) 55%,transparent)}.release-skeleton-row i:first-child{width:52px;height:14px}.release-skeleton-row i:last-child{width:70px;height:12px}.release-loading .release-detail header{min-height:54px;padding-top:10px}.release-loading .release-detail header>span{gap:6px}.release-skeleton.title{width:72px;height:16px}.release-skeleton.date{width:92px;height:11px}.release-skeleton-body{display:flex;flex-direction:column;gap:10px;padding:12px;border-radius:var(--radius-md);background:color-mix(in srgb,var(--bg-raised) 55%,transparent)}.release-skeleton.heading{width:64px;height:15px;margin-bottom:3px}.release-skeleton.heading.second{margin-top:12px}.release-skeleton.line{width:76%;height:10px}.release-skeleton.line.wide{width:91%}.release-skeleton.line.short{width:58%}.release-load-failed{place-items:center}.release-load-message{display:flex;align-items:center;gap:12px}.inline-error{margin:0;color:var(--danger);font-size:var(--fs-ui)}button{min-height:32px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}button:hover{color:var(--text-1);background:var(--bg-hover)}button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
</style>
