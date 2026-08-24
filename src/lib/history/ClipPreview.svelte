<script lang="ts">
  import { File } from "@lucide/svelte";
  import { Tabs } from "bits-ui";
  import { formatDateTime, formatNumber, t } from "$lib/i18n/index.svelte";
  import { clipPreview, detailText, fileName, filePaths, metadataFacts } from "./presentation";
  import type { ClipDetail, HistoryPage } from "./types";

  let {
    detail,
    selectedId,
    page,
    pending,
    assetUrl,
    fileAccessDenied,
    sourceIconUrl,
    fileThumbnailUrls,
    fileByteSizes,
    fileIndex,
    trimWhitespace,
    previousFileShortcut,
    nextFileShortcut,
    onfile,
    onfilekeydown,
    onfilefocus,
    oninert,
  }: {
    detail: ClipDetail | null;
    selectedId: string | null;
    page: HistoryPage;
    pending: boolean;
    assetUrl: string | null;
    fileAccessDenied: boolean;
    sourceIconUrl: string | null;
    fileThumbnailUrls: Array<string | null>;
    fileByteSizes: Array<number | null>;
    fileIndex: number;
    trimWhitespace: boolean;
    previousFileShortcut: string;
    nextFileShortcut: string;
    onfile: (index: number) => void;
    onfilekeydown: (event: KeyboardEvent) => void;
    onfilefocus: () => void;
    oninert: () => void;
  } = $props();
</script>

<section role="group" class:pending class:file-preview={detail?.content_type === "file"} class="preview" aria-live="polite" aria-busy={pending} onpointerdown={(event) => { if (!(event.target as Element).closest("button, a, input, [role='tab']")) oninert(); }}>
  {#if detail}
    <div class:text-preview={!["color", "file", "image"].includes(detail.content_type)} class="preview-body">
      {#if detail.content_type === "color"}
        <div class="color-preview"><span style:background={detail.preview}></span><code>{detail.preview}</code></div>
      {:else if detail.content_type === "file"}
        {#if assetUrl}<img class="asset" src={assetUrl} alt={t("history.fileThumbnail")} />
        {:else}<div class="file-preview-placeholder">{t(fileAccessDenied ? "history.fileAccessDenied" : "history.systemPreviewHint")}</div>{/if}
      {:else if detail.content_type === "image"}
        {#if assetUrl}<div class="asset-frame"><img class="asset" src={assetUrl} alt={t("history.imagePreview")} /></div>
        {:else}<div class="image-placeholder">{t("history.image")} · {typeof detail.metadata.width === "number" ? formatNumber(detail.metadata.width) : "?"}×{typeof detail.metadata.height === "number" ? formatNumber(detail.metadata.height) : "?"}</div>{/if}
      {:else}
        <pre>{detailText(detail, trimWhitespace)}</pre>
      {/if}
    </div>
    {#if detail.content_type === "file" && filePaths(detail).length > 1}
      <nav class="file-nav" aria-label={t("history.fileNavigation")}>
        <button tabindex="-1" class="file-nav-arrow" aria-label={t("history.previousFile", { shortcut: previousFileShortcut })} disabled={fileIndex === 0} onclick={() => onfile(fileIndex - 1)}><kbd>{previousFileShortcut}</kbd></button>
        <Tabs.Root class="file-tabs" value={String(fileIndex)} onValueChange={(value) => onfile(Number(value))} orientation="horizontal" activationMode="automatic" loop={false}>
        <Tabs.List class="file-strip" aria-label={t("history.fileCount", { count: formatNumber(filePaths(detail).length) })}>
          {#each filePaths(detail) as path, index}
            <Tabs.Trigger value={String(index)} data-file-index={index} class={`file-thumb${index === fileIndex ? " selected" : ""}`} aria-label={t("history.viewFile", { index: formatNumber(index + 1), name: fileName(path, t("meta.file")) })} title={fileName(path, t("meta.file"))} onfocus={onfilefocus} onkeydown={onfilekeydown}>
              {#if fileThumbnailUrls[index]}<img src={fileThumbnailUrls[index] ?? undefined} alt="" />
              {:else}<File size={16} aria-hidden="true" />{/if}
            </Tabs.Trigger>
          {/each}
        </Tabs.List>
        </Tabs.Root>
        <button tabindex="-1" class="file-nav-arrow" aria-label={t("history.nextFile", { shortcut: nextFileShortcut })} disabled={fileIndex === filePaths(detail).length - 1} onclick={() => onfile(fileIndex + 1)}><kbd>{nextFileShortcut}</kbd></button>
        <span class="file-nav-count" aria-live="polite">{formatNumber(fileIndex + 1)}/{formatNumber(filePaths(detail).length)}</span>
      </nav>
    {/if}
    <div class="preview-meta">
      {#if detail.content_type === "file"}
        <div class="meta-file">
          <span title={filePaths(detail)[fileIndex] ?? detail.preview}>{fileName(filePaths(detail)[fileIndex] ?? detail.preview, t("meta.file"))}</span>
          {#if filePaths(detail)[fileIndex]}<code title={filePaths(detail)[fileIndex]}>{filePaths(detail)[fileIndex]}</code>{/if}
        </div>
      {/if}
      <div class="meta-summary">
        <div class="meta-source">
          {#if detail.source_app}
            {#if sourceIconUrl}<img class="app-icon" src={sourceIconUrl} alt="" />
            {:else}<span class="app-fallback" aria-hidden="true">{detail.source_app.name.slice(0, 1)}</span>{/if}
            <div class="source-details"><span>{detail.source_app.name}</span><time datetime={detail.created_at}>{t("meta.firstCopied")} {formatDateTime(detail.created_at)}</time><time datetime={detail.last_used_at}>{t("meta.lastUsed")} {formatDateTime(detail.last_used_at)}</time></div>
          {:else}
            <div class="source-details"><time datetime={detail.created_at}>{t("meta.firstCopied")} {formatDateTime(detail.created_at)}</time><time datetime={detail.last_used_at}>{t("meta.lastUsed")} {formatDateTime(detail.last_used_at)}</time></div>
          {/if}
        </div>
        <dl class="meta-facts">
          {#each metadataFacts(detail, fileIndex, fileByteSizes, { dimensions: t("meta.dimensions"), size: t("meta.size"), file: t("meta.file"), files: t("meta.files"), hostname: t("meta.hostname"), type: t("meta.type"), characters: t("meta.characters") }, formatNumber) as fact}
            <div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>
          {/each}
        </dl>
      </div>
    </div>
  {:else if selectedId}
    <div class="preview-loading"><span>{t("history.previewLoading")}</span><pre>{page.items.find((item) => item.id === selectedId) ? clipPreview(page.items.find((item) => item.id === selectedId)!, t("meta.file")) : ""}</pre></div>
  {:else}
    <div class="empty">{t("history.select")}</div>
  {/if}
</section>

<style>
  .preview { grid-column:2; grid-row:2; min-width:0; min-height:0; display:flex; flex-direction:column; }
  .preview.pending { contain:content; }
  .preview-body { flex:1; min-height:0; overflow:hidden; display:flex; align-items:center; justify-content:center; padding:20px; }
  .preview-body.text-preview { align-items:flex-start; justify-content:flex-start; overflow:auto; }
  .preview-body.text-preview pre { max-height:none; overflow:visible; }
  pre { max-width:100%; max-height:100%; margin:0; overflow:hidden; color:var(--text-1); font:var(--fs-body)/var(--lh-relaxed) var(--mono); white-space:pre-wrap; overflow-wrap:anywhere; }
  .preview-meta { height:64px; flex:none; display:flex; align-items:center; padding:8px 20px; border-top:1px solid var(--hairline); }
  .preview.file-preview .preview-meta { height:96px; display:grid; grid-template-rows:minmax(0, 1fr) auto; gap:7px; padding-block:10px; }
  .meta-summary { min-width:0; width:100%; display:flex; align-items:center; justify-content:space-between; gap:20px; }
  .meta-source { min-width:0; display:flex; align-items:center; gap:8px; }
  .source-details { min-width:0; display:flex; flex-direction:column; gap:2px; color:var(--text-2); font:var(--fs-ui)/var(--lh-tight) var(--mono); }
  .source-details span, .meta-file > span, .meta-file code { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .source-details time { color:var(--text-3); font-size:var(--fs-caption); }
  .meta-file { min-width:0; width:100%; display:flex; flex-direction:column; gap:3px; color:var(--text-2); font:var(--fs-meta)/var(--lh-snug) var(--mono); }
  .meta-facts { display:flex; gap:16px; margin:0; }
  .meta-facts div { display:flex; flex-direction:column; align-items:flex-end; gap:2px; white-space:nowrap; }
  .meta-facts dt { color:var(--text-3); font:var(--fs-caption)/var(--lh-flush) var(--mono); }
  .meta-facts dd { margin:0; color:var(--text-2); font:var(--fs-meta)/var(--lh-tight) var(--mono); }
  .app-icon, .app-fallback { width:22px; height:22px; flex:none; border-radius:var(--radius-sm); }
  .app-icon { object-fit:contain; }
  .app-fallback { display:grid; place-items:center; color:var(--bg-shell); background:var(--text-2); font:600 var(--fs-meta) var(--mono); }
  .color-preview { display:flex; align-items:center; gap:14px; }
  .color-preview span { width:72px; height:72px; border:1px solid var(--hairline); border-radius:var(--radius-lg); }
  .asset-frame { width:100%; height:100%; min-height:180px; display:flex; align-items:center; justify-content:center; }
  .asset { display:block; max-width:100%; max-height:100%; border-radius:var(--radius-lg); object-fit:contain; }
  .image-placeholder, .empty { flex:1; display:grid; place-items:center; color:var(--text-3); }
  .file-preview-placeholder { color:var(--text-3); font-size:var(--fs-ui); }
  .file-nav { height:58px; flex:none; display:flex; align-items:center; gap:8px; padding:6px 20px; border-top:1px solid var(--hairline); }
  :global(.file-tabs) { min-width:0; flex:1; }
  :global(.file-strip) { min-width:0; flex:1; display:flex; gap:6px; overflow-x:auto; }
  :global(.file-thumb) { width:38px; height:38px; flex:none; display:grid; place-items:center; padding:3px; border-radius:var(--radius-md); background:transparent; }
  :global(.file-thumb.selected) { background:var(--bg-selected); }
  :global(.file-thumb) img { width:100%; height:100%; border-radius:var(--radius-sm); object-fit:cover; }
  .file-nav-arrow { min-width:38px; height:28px; padding:0 3px; color:var(--text-2); background:transparent; }
  .file-nav-count { color:var(--text-3); font:var(--fs-caption) var(--mono); }
  .preview-loading { flex:1; min-height:0; padding:20px; color:var(--text-2); }
  .preview-loading span { display:block; margin-bottom:8px; color:var(--text-3); font-size:var(--fs-meta); }
  kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:var(--radius-sm); padding:1px 5px; white-space:nowrap; }
</style>
