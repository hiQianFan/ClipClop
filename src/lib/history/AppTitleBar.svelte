<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";

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

  function activateMenuTrigger(event: MouseEvent, keyboardHandler: unknown) {
    if (event.detail === 0 && typeof keyboardHandler === "function") keyboardHandler(event);
    else onopenchange(!open);
  }
</script>

<header class="titlebar">
  {#if history}
    <div class="brand">
      <DropdownMenu.Root {open} onOpenChange={onopenchange}>
        <div class="app-menu-wrap">
          <DropdownMenu.Trigger bind:ref={menuButton} class={`app-menu-trigger pressable${open ? " expanded" : ""}`} aria-label={t("history.appMenu")}>
            {#snippet child({ props: triggerProps })}
              <button
                {...triggerProps}
                onpointerdown={() => {}}
                onpointerup={() => {}}
                onclick={(event) => activateMenuTrigger(event, triggerProps.onclick)}
              >
                <span class="brand-mark" aria-hidden="true"></span>
                <span>ClipClop</span>
              </button>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.ContentStatic class="menu app-menu" aria-label={t("history.appMenu")} loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); menuButton?.focus(); }}>
            <DropdownMenu.Item onclick={onsettings}>{t("history.settings")} <ShortcutHint shortcut={settingsShortcut} platform={settingsShortcut.startsWith("Command") ? "macos" : "windows"} /></DropdownMenu.Item>
            <DropdownMenu.Item onclick={onupdates}>{t("history.checkUpdates")}</DropdownMenu.Item>
            <DropdownMenu.Item onclick={onabout}>{t("history.about")}</DropdownMenu.Item>
            <DropdownMenu.Separator class="menu-separator" />
            <DropdownMenu.Item class="danger" onclick={onquit}><span>{t("history.quit")}</span><ShortcutHint shortcut={quitShortcut} platform={quitShortcut.startsWith("Command") ? "macos" : "windows"} /></DropdownMenu.Item>
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
  .brand :global(.app-menu-trigger) { height:24px; display:flex; align-items:center; gap:4px; padding:0 4px; border-radius:var(--radius-md); color:var(--text-2); background:transparent; font-size:var(--fs-ui); font-weight:600; letter-spacing:.01em; }
  .brand :global(.app-menu-trigger:hover), .brand :global(.app-menu-trigger.expanded) { background:var(--bg-hover); }
  .brand-mark { width:14px; height:14px; flex:none; background:currentColor; mask:url("/clipclop-mark.svg") center/contain no-repeat; -webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat; }
  .settings-title { color:var(--text-1); font-size:var(--fs-emphasis); font-weight:600; }
  .app-menu-wrap :global(.menu) { position:absolute; top:30px; left:0; width:180px; padding:6px; border:1px solid var(--hairline); border-radius:var(--radius-lg); background:var(--bg-raised); box-shadow:var(--menu-shadow); }
  .app-menu-wrap :global(.menu [role="menuitem"]) { width:100%; display:flex; align-items:center; justify-content:space-between; gap:16px; padding:9px 10px; border-radius:var(--radius-md); color:var(--text-1); background:transparent; line-height:var(--lh-snug); text-align:left; transition:background-color var(--dur-fast) var(--ease-out),opacity var(--dur-fast) var(--ease-out),filter var(--dur-fast) var(--ease-out); }
  .app-menu-wrap :global(.menu [role="menuitem"] > span) { min-width:0; }
  .app-menu-wrap :global(.menu [role="menuitem"]:hover), .app-menu-wrap :global(.menu [role="menuitem"][data-highlighted]) { background:var(--bg-hover); }
  .app-menu-wrap :global(.menu [role="menuitem"]:active:not([data-disabled])) { opacity:.88; filter:brightness(.94); background:var(--bg-selected); transition-duration:40ms; }
  .app-menu-wrap :global(.menu-separator) { height:1px; margin:5px 6px; background:var(--hairline); }
  .app-menu-wrap :global(.menu .danger) { color:var(--danger); }
</style>
