<script lang="ts">
  import { AlertDialog, DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";

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

  function activateActionTrigger(event: MouseEvent, keyboardHandler: unknown) {
    if (event.detail === 0 && typeof keyboardHandler === "function") keyboardHandler(event);
    else onmenuopenchange(!menuOpen);
  }

</script>

<AlertDialog.Root open={deletePending} onOpenChange={ondeleteopenchange}>
  <footer class="actions">
    {#if deletePending}
      <AlertDialog.Content class="confirmation" aria-label={t("history.confirmDeleteLabel")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); onrestorefocus(); }}>
        <span>{t("history.confirmDelete")}<small>{t("history.confirmDeleteHelp")}</small></span>
        <AlertDialog.Cancel class="ghost" onclick={oncanceldelete}>{t("common.cancel")} <ShortcutHint shortcut="Escape" variant="compact" /></AlertDialog.Cancel>
        <AlertDialog.Action bind:ref={confirmButton} class="destructive" onclick={onconfirmdelete}>{t("history.delete")}</AlertDialog.Action>
      </AlertDialog.Content>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if copied}<span class="message">{copied}</span>{/if}
      <DropdownMenu.Root open={menuOpen} onOpenChange={onmenuopenchange}>
        <div class="menu-wrap">
          <DropdownMenu.Trigger bind:ref={menuButton} class={`ghost action-menu-trigger pressable${menuOpen ? " expanded" : ""}`} disabled={!selected}>
            {#snippet child({ props: triggerProps })}
              <button
                {...triggerProps}
                onpointerdown={() => {}}
                onpointerup={() => {}}
                onclick={(event) => activateActionTrigger(event, triggerProps.onclick)}
              ><ShortcutHint shortcut={actionMenuShortcut} platform={isMac ? "macos" : "windows"} /> {t("history.actions")}</button>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu action-menu" aria-label={t("history.actionMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); if (!deletePending) onbrowse(); }}>
            {#if canPreview}<DropdownMenu.Item onclick={onpreview}><span>{t("history.viewSelected")}</span><ShortcutHint shortcut="Space" platform={isMac ? "macos" : "windows"} /></DropdownMenu.Item>{/if}
            {#if isLink}<DropdownMenu.Item onclick={onopenlink}><span>{t("history.openLink")}</span></DropdownMenu.Item>{/if}
            <DropdownMenu.Separator class="menu-separator" />
            {#if hasPlainText}<DropdownMenu.Item onclick={onpasteplain}><span>{t("history.pastePlain")}</span><ShortcutHint shortcut="Shift+Enter" platform={isMac ? "macos" : "windows"} /></DropdownMenu.Item>{/if}
            <DropdownMenu.Item onclick={oncopy}><span>{t("history.copy")}</span></DropdownMenu.Item>
            {#if hasPlainText}<DropdownMenu.Item onclick={oncopyplain}><span>{t("history.copyPlain")}</span><ShortcutHint shortcut={isMac ? "Command+Shift+C" : "Ctrl+Shift+C"} platform={isMac ? "macos" : "windows"} /></DropdownMenu.Item>{/if}
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={requestDelete}><span>{t("history.deleteFrom")}</span><ShortcutHint shortcut={deleteShortcut} platform={isMac ? "macos" : "windows"} /></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
        </div>
      </DropdownMenu.Root>
      <button class="copy pressable" onclick={onpaste} disabled={!selected}><ShortcutHint shortcut="Enter" platform={isMac ? "macos" : "windows"} inherit /> {t("history.paste")}</button>
    {/if}
  </footer>
</AlertDialog.Root>

<style>
  .actions { grid-column:2; grid-row:3; display:flex; align-items:center; justify-content:flex-end; gap:12px; padding:0 16px; border-top:1px solid var(--hairline); }
  .copy, .actions :global(.ghost), .actions :global(.destructive) { display:flex; align-items:center; gap:6px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; padding:7px 10px; }
  .copy { color:var(--action-on); background:var(--action); padding-inline:15px; font-weight:650; }
  .copy:hover { background:var(--action-hover); }
  .actions :global(.ghost:hover), .actions :global(.ghost.expanded) { color:var(--text-1); background:var(--bg-hover); }
  .actions :global(.action-menu-trigger) { border:1px solid var(--hairline); }
  .actions :global(.ghost:active), .actions :global(.ghost.expanded:active) { background:var(--bg-selected); }
  .actions :global(.destructive) { color:var(--danger-on); background:var(--danger-fill); font-weight:600; }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu-wrap :global(.menu) { position:absolute; right:0; bottom:38px; width:260px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .menu-wrap :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; transition:background-color var(--dur-fast) var(--ease-out),opacity var(--dur-fast) var(--ease-out),filter var(--dur-fast) var(--ease-out); }
  .menu-wrap :global(.menu [role="menuitem"] > span) { min-width:0; }
  .menu-wrap :global(.menu [role="menuitem"]:hover), .menu-wrap :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  .menu-wrap :global(.menu [role="menuitem"]:active:not([data-disabled])) { opacity:.88; filter:brightness(.94); background:var(--bg-selected); transition-duration:40ms; }
  .menu-wrap :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  .menu-wrap :global(.menu .danger) { color:var(--danger); }
  .message { min-width:0; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-right:auto; color:var(--text-2); font-size:var(--fs-meta); }
  .message.error { color:var(--danger); }
  .actions :global(.confirmation) { width:100%; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  .actions :global(.confirmation button) { min-width:92px; min-height:32px; justify-content:center; padding:0 12px; }
  .actions :global(.confirmation > span) { margin-right:auto; color:var(--text-1); font-size:var(--fs-ui); font-weight:600; }
  .actions :global(.confirmation small) { display:block; margin-top:2px; color:var(--text-2); font-size:var(--fs-caption); font-weight:400; }
</style>
