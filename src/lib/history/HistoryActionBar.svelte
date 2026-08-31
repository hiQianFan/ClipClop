<script lang="ts">
  import { AlertDialog, DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";

  let {
    selected,
    canPreview,
    isLink,
    hasPlainText,
    isMac,
    error,
    copied,
    menuOpen,
    deletePending,
    actionMenuShortcut,
    deleteShortcut,
    onmenuopenchange,
    ondeleteopenchange,
    onbrowse,
    onpreview,
    onopenlink,
    onpasteplain,
    oncopy,
    oncopyplain,
    onrequestdelete,
    oncanceldelete,
    onconfirmdelete,
    onpaste,
    onrestorefocus,
  }: {
    selected: boolean;
    canPreview: boolean;
    isLink: boolean;
    hasPlainText: boolean;
    isMac: boolean;
    error: string;
    copied: string;
    menuOpen: boolean;
    deletePending: boolean;
    actionMenuShortcut: string;
    deleteShortcut: string;
    onmenuopenchange: (open: boolean) => void;
    ondeleteopenchange: (open: boolean) => void;
    onbrowse: () => void;
    onpreview: () => void;
    onopenlink: () => void;
    onpasteplain: () => void;
    oncopy: () => void;
    oncopyplain: () => void;
    onrequestdelete: (invoker: HTMLElement | null) => void;
    oncanceldelete: () => void;
    onconfirmdelete: () => void;
    onpaste: () => void;
    onrestorefocus: () => void;
  } = $props();

  let menuButton = $state<HTMLButtonElement | null>(null);
  let confirmButton = $state<HTMLButtonElement | null>(null);

  function requestDelete() {
    onrequestdelete(menuButton);
  }
</script>

<AlertDialog.Root open={deletePending} onOpenChange={ondeleteopenchange}>
  <footer class="actions">
    {#if deletePending}
      <AlertDialog.Content class="confirmation" aria-label={t("history.confirmDeleteLabel")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); onrestorefocus(); }}>
        <span>{t("history.confirmDelete")}<small>{t("history.confirmDeleteHelp")}</small></span>
        <AlertDialog.Cancel class="ghost" onclick={oncanceldelete}>{t("common.cancel")} <kbd>Esc</kbd></AlertDialog.Cancel>
        <AlertDialog.Action bind:ref={confirmButton} class="destructive" onclick={onconfirmdelete}>{t("history.delete")}</AlertDialog.Action>
      </AlertDialog.Content>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if copied}<span class="message">{copied}</span>{/if}
      <DropdownMenu.Root open={menuOpen} onOpenChange={onmenuopenchange}>
        <div class="menu-wrap">
          <DropdownMenu.Trigger bind:ref={menuButton} class={`ghost action-menu-trigger${menuOpen ? " expanded" : ""}`} disabled={!selected}><kbd>{actionMenuShortcut}</kbd> {t("history.actions")}</DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu action-menu" aria-label={t("history.actionMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); if (!deletePending) onbrowse(); }}>
            {#if canPreview}<DropdownMenu.Item onclick={onpreview}><span>{t("history.viewSelected")}</span><kbd>Space</kbd></DropdownMenu.Item>{/if}
            {#if isLink}<DropdownMenu.Item onclick={onopenlink}><span>{t("history.openLink")}</span></DropdownMenu.Item>{/if}
            <DropdownMenu.Separator class="menu-separator" />
            {#if hasPlainText}<DropdownMenu.Item onclick={onpasteplain}><span>{t("history.pastePlain")}</span><kbd>⇧⏎</kbd></DropdownMenu.Item>{/if}
            <DropdownMenu.Item onclick={oncopy}><span>{t("history.copy")}</span></DropdownMenu.Item>
            {#if hasPlainText}<DropdownMenu.Item onclick={oncopyplain}><span>{t("history.copyPlain")}</span><kbd>{isMac ? "⌘⇧C" : "Ctrl⇧C"}</kbd></DropdownMenu.Item>{/if}
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={requestDelete}><span>{t("history.deleteFrom")}</span><kbd>{deleteShortcut}</kbd></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
        </div>
      </DropdownMenu.Root>
      <button class="copy" onclick={onpaste} disabled={!selected}><kbd>⏎</kbd> {t("history.paste")}</button>
    {/if}
  </footer>
</AlertDialog.Root>

<style>
  kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:var(--radius-sm); padding:1px 5px; white-space:nowrap; }
  .actions { grid-column:2; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, .actions :global(.ghost), .actions :global(.destructive) { display:flex; align-items:center; gap:6px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:650; }
  .copy:hover { background:var(--action-hover); }
  .copy:active { filter:brightness(.92); }
  .copy kbd { color:inherit; border-color:currentColor; opacity:.9; }
  .actions :global(.ghost:hover), .actions :global(.ghost.expanded) { color:var(--text-1); background:var(--bg-hover); }
  .actions :global(.ghost:active), .actions :global(.ghost.expanded:active) { background:var(--bg-selected); }
  .actions :global(.action-menu-trigger.expanded kbd) { color:inherit; border-color:currentColor; }
  .actions :global(.destructive) { color:var(--danger-on); background:var(--danger-fill); font-weight:600; }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu-wrap :global(.menu) { position:absolute; right:0; bottom:38px; width:260px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .menu-wrap :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; }
  .menu-wrap :global(.menu [role="menuitem"] > span) { min-width:0; }
  .menu-wrap :global(.menu [role="menuitem"] > kbd) { flex:none; align-self:center; font-family:inherit; font-size:var(--fs-body); font-weight:500; line-height:1; }
  .menu-wrap :global(.menu [role="menuitem"]:hover), .menu-wrap :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  .menu-wrap :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  .menu-wrap :global(.menu .danger) { color:var(--danger); }
  .menu-wrap :global(.menu .danger kbd) { color:currentColor; border-color:currentColor; }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:var(--fs-meta); }
  .message.error { color:var(--danger); }
  .actions :global(.confirmation) { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  .actions :global(.confirmation button) { min-width:92px; min-height:32px; justify-content:center; padding:0 12px; }
  .actions :global(.confirmation > span) { margin-right:auto; color:var(--text-1); font-size:var(--fs-ui); font-weight:600; }
  .actions :global(.confirmation small) { display:block; margin-top:2px; color:var(--text-2); font-size:var(--fs-caption); font-weight:400; }
</style>
