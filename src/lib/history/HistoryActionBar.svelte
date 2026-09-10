<script lang="ts">
  import { AlertDialog, DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import ActionToolbar from "$lib/components/ActionToolbar.svelte";

  let {
    selected,
    favorite = false,
    favoritePending = false,
    onfavorite,
    canPreview,
    isLink,
    hasPlainText,
    isMac,
    error,
    permissionRecovery = false,
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
    onpermission,
    onrestorefocus,
  }: {
    selected: boolean;
    favorite?: boolean;
    favoritePending?: boolean;
    onfavorite?: () => void;
    canPreview: boolean;
    isLink: boolean;
    hasPlainText: boolean;
    isMac: boolean;
    error: string;
    permissionRecovery?: boolean;
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
    onpermission: () => void;
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
  <ActionToolbar class="actions">
    {#if deletePending}
      <AlertDialog.Content class="confirmation" aria-label={t("history.confirmDeleteLabel")} preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmButton?.focus(); }} onCloseAutoFocus={(event) => { event.preventDefault(); onrestorefocus(); }}>
        <span>{t("history.confirmDelete")}<small>{t("history.confirmDeleteHelp")}</small></span>
        <AlertDialog.Cancel class="toolbar-button pressable" onclick={oncanceldelete}>{t("common.cancel")} <ShortcutHint shortcut="Escape" variant="compact" /></AlertDialog.Cancel>
        <AlertDialog.Action bind:ref={confirmButton} class="toolbar-button destructive pressable" onclick={onconfirmdelete}>{t("history.delete")} <ShortcutHint shortcut="Enter" platform={isMac ? "macos" : "windows"} inherit /></AlertDialog.Action>
      </AlertDialog.Content>
    {:else}
      {#if error}<span class="message error" title={error}>{error}</span>{/if}
      {#if permissionRecovery}<button class="toolbar-button secondary pressable" onclick={onpermission}>{t("history.handlePermission")}</button>{/if}
      <DropdownMenu.Root open={menuOpen} onOpenChange={onmenuopenchange}>
        <div class="menu-wrap">
          <DropdownMenu.Trigger bind:ref={menuButton} class={`toolbar-button secondary action-menu-trigger pressable${menuOpen ? " expanded" : ""}`} disabled={!selected}>
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
            <DropdownMenu.Item disabled={favoritePending} onclick={onfavorite}><span>{t(favorite ? "history.unfavorite" : "history.favorite")}</span></DropdownMenu.Item>
            <DropdownMenu.Item class="danger" onclick={requestDelete}><span>{t("history.deleteFrom")}</span><ShortcutHint shortcut={deleteShortcut} platform={isMac ? "macos" : "windows"} /></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
        </div>
      </DropdownMenu.Root>
      <button class="toolbar-button primary pressable" onclick={onpaste} disabled={!selected}><ShortcutHint shortcut="Enter" platform={isMac ? "macos" : "windows"} inherit /> {t("history.paste")}</button>
    {/if}
  </ActionToolbar>
</AlertDialog.Root>

<style>
  :global(.actions) { grid-column:2; grid-row:3; }
  :global(.actions .action-menu-trigger.expanded) { color:var(--text-1); background:var(--bg-hover); }
  :global(.actions .action-menu-trigger:active), :global(.actions .action-menu-trigger.expanded:active) { background:var(--bg-selected); }
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
  :global(.actions .confirmation) { width:100%; min-width:0; display:flex; align-items:center; justify-content:flex-end; gap:8px; }
  :global(.actions .confirmation > span) { min-width:0; margin-right:auto; color:var(--text-1); font-size:var(--fs-ui); font-weight:600; }
  :global(.actions .confirmation small) { display:block; margin-top:2px; overflow:hidden; color:var(--text-2); font-size:var(--fs-caption); font-weight:400; text-overflow:ellipsis; white-space:nowrap; }
</style>
