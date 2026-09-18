<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import ActionToolbar from "$lib/components/ActionToolbar.svelte";

  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

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
    deleting = false,
    deleteError = "",
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
    deleting?: boolean;
    deleteError?: string;
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

  function requestDelete() {
    onrequestdelete(menuButton);
  }

  function activateActionTrigger(event: MouseEvent, keyboardHandler: unknown) {
    if (event.detail === 0 && typeof keyboardHandler === "function") keyboardHandler(event);
    else onmenuopenchange(!menuOpen);
  }

</script>

<div class="history-actions">
  {#if error || permissionRecovery}
    <div class="feedback">
      {#if error}<p role="status">{error}</p>{/if}
      {#if permissionRecovery}<button class="pressable" onclick={onpermission}>{t("history.handlePermission")}</button>{/if}
    </div>
  {/if}
  <ActionToolbar class="actions">
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
  </ActionToolbar>
</div>
<ConfirmDialog open={deletePending} title={t("history.confirmDelete")} description={t("history.confirmDeleteHelp")}
  action={t("history.delete")} focusAction busy={deleting} error={deleteError}
  onopenchange={(open) => { if (open) ondeleteopenchange(true); else oncanceldelete(); }}
  onconfirm={onconfirmdelete} {onrestorefocus} />

<style>
  .history-actions { grid-column:2; grid-row:3; min-width:0; }
  .feedback { display:flex; align-items:center; flex-wrap:wrap; gap:8px; padding:10px 16px; border-top:1px solid var(--hairline); max-height:25vh; overflow:auto; }
  .feedback p { flex:1 1 220px; min-width:0; margin:0; overflow-wrap:anywhere; color:var(--text-2); font-size:var(--fs-ui); line-height:var(--lh-snug); }
  .feedback button { padding:6px 10px; border:1px solid var(--hairline); border-radius:var(--radius-md); color:var(--text-1); background:var(--bg-raised); font-size:var(--fs-ui); }
  .feedback button:focus-visible { outline:2px solid var(--text-1); outline-offset:2px; }
  :global(.actions .action-menu-trigger.expanded) { color:var(--text-1); background:var(--bg-hover); }
  :global(.actions .action-menu-trigger:active), :global(.actions .action-menu-trigger.expanded:active) { background:var(--bg-selected); }
  button:disabled { opacity:.45; }
  .menu-wrap { position:relative; }
  .menu-wrap :global(.menu) { position:absolute; z-index:var(--z-menu); right:0; bottom:38px; width:260px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .menu-wrap :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; transition:background-color var(--dur-fast) var(--ease-out),opacity var(--dur-fast) var(--ease-out),filter var(--dur-fast) var(--ease-out); }
  .menu-wrap :global(.menu [role="menuitem"] > span) { min-width:0; }
  .menu-wrap :global(.menu [role="menuitem"]:hover), .menu-wrap :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  .menu-wrap :global(.menu [role="menuitem"]:active:not([data-disabled])) { opacity:.88; filter:brightness(.94); background:var(--bg-selected); transition-duration:40ms; }
  .menu-wrap :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  .menu-wrap :global(.menu .danger) { color:var(--danger); }
</style>
