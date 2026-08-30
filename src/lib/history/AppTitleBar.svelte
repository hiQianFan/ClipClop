<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";

  let {
    history,
    open,
    settingsShortcut,
    quitShortcut,
    onopenchange,
    onsettings,
    onupdates,
    onabout,
    onquit,
  }: {
    history: boolean;
    open: boolean;
    settingsShortcut: string;
    quitShortcut: string;
    onopenchange: (open: boolean) => void;
    onsettings: () => void;
    onupdates: () => void;
    onabout: () => void;
    onquit: () => void;
  } = $props();

  let menuButton = $state<HTMLButtonElement | null>(null);
</script>

<header class="titlebar">
  {#if history}
    <div class="brand">
      <DropdownMenu.Root {open} onOpenChange={onopenchange}>
        <div class="app-menu-wrap">
          <DropdownMenu.Trigger bind:ref={menuButton} class="app-menu-trigger" aria-label={t("history.appMenu")}>
            <span class="brand-mark" aria-hidden="true"></span>
            <span>ClipClop</span>
          </DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu app-menu" aria-label={t("history.appMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); menuButton?.focus(); }}>
            <DropdownMenu.Item onclick={onsettings}>{t("history.settings")} <kbd>{settingsShortcut}</kbd></DropdownMenu.Item>
            <DropdownMenu.Item onclick={onupdates}>{t("history.checkUpdates")}</DropdownMenu.Item>
            <DropdownMenu.Item onclick={onabout}>{t("history.about")}</DropdownMenu.Item>
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={onquit}><span>{t("history.quit")}</span><kbd>{quitShortcut}</kbd></DropdownMenu.Item>
          </DropdownMenu.ContentStatic>
        </div>
      </DropdownMenu.Root>
    </div>
  {:else}
    <span class="settings-title">{t("settings.title")}</span>
  {/if}
  <div class="titlebar-drag" data-tauri-drag-region></div>
</header>

<style>
  .titlebar { grid-column:1 / -1; grid-row:1; display:flex; align-items:center; padding:0 14px; border-bottom:1px solid var(--hairline); user-select:none; }
  .titlebar-drag { flex:1; align-self:stretch; }
  .brand { display:flex; align-items:center; color:var(--text-2); }
  .app-menu-wrap { position:relative; }
  :global(.app-menu-trigger) { height:24px; display:flex; align-items:center; gap:4px; padding:0 4px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; font-size:var(--fs-ui); font-weight:600; letter-spacing:.01em; }
  :global(.app-menu-trigger:hover) { background:var(--bg-hover); }
  .brand-mark { width:14px; height:14px; flex:none; background:currentColor; mask:url("/clipclop-mark.svg") center/contain no-repeat; -webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat; }
  .settings-title { color:var(--text-1); font-size:var(--fs-emphasis); font-weight:600; }
  kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); color:var(--text-2); border:1px solid var(--hairline); border-radius:var(--radius-sm); padding:1px 5px; white-space:nowrap; }
  :global(.menu) { position:absolute; right:0; bottom:38px; width:210px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  :global(.app-menu) { top:30px; bottom:auto; left:0; right:auto; width:180px; }
  :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; }
  :global(.menu [role="menuitem"] > span) { min-width:0; }
  :global(.menu [role="menuitem"] > kbd) { flex:none; align-self:center; font-family:inherit; font-size:var(--fs-body); font-weight:500; line-height:1; }
  :global(.menu [role="menuitem"]:hover), :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  :global(.menu .danger) { color:var(--danger); }
  :global(.menu .danger kbd) { color:currentColor; border-color:currentColor; }
</style>
