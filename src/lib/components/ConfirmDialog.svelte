<script lang="ts">
  import { AlertDialog } from "bits-ui";
  import { t } from "$lib/i18n/index.svelte";
  import ActionToolbar from "./ActionToolbar.svelte";

  let { open, title, description, action, focusAction = false, busy = false, error = "", onopenchange, onconfirm, onrestorefocus }: {
    open: boolean;
    title: string;
    description: string;
    action: string;
    focusAction?: boolean;
    busy?: boolean;
    error?: string;
    onopenchange: (open: boolean) => void;
    onconfirm: () => void;
    onrestorefocus: () => void;
  } = $props();
  let cancelButton = $state<HTMLButtonElement | null>(null);
  let actionButton = $state<HTMLButtonElement | null>(null);

  function onKeydown(event: KeyboardEvent) {
    if (busy || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    event.stopPropagation();
    (event.key === "ArrowLeft" ? cancelButton : actionButton)?.focus();
  }
</script>

<AlertDialog.Root {open} onOpenChange={(value) => { if (!busy) onopenchange(value); }}>
  <AlertDialog.Portal>
    <AlertDialog.Overlay class="confirm-overlay" />
    <AlertDialog.Content class="confirm-dialog"
      onkeydown={onKeydown}
      onEscapeKeydown={(event) => { if (busy) event.preventDefault(); }}
      onOpenAutoFocus={(event) => { event.preventDefault(); (focusAction ? actionButton : cancelButton)?.focus(); }}
      onCloseAutoFocus={(event) => { event.preventDefault(); onrestorefocus(); }}>
      <div class="confirm-copy">
        <AlertDialog.Title class="confirm-title">{title}</AlertDialog.Title>
        <AlertDialog.Description class="confirm-description">{description}</AlertDialog.Description>
        {#if error}<p class="confirm-error" role="alert">{error}</p>{/if}
      </div>
      <ActionToolbar>
        <AlertDialog.Cancel bind:ref={cancelButton} class="toolbar-button pressable" disabled={busy}>{t("common.cancel")}</AlertDialog.Cancel>
        <button bind:this={actionButton} class="toolbar-button destructive pressable" disabled={busy} aria-busy={busy} onclick={onconfirm}>{action}</button>
      </ActionToolbar>
    </AlertDialog.Content>
  </AlertDialog.Portal>
</AlertDialog.Root>

<style>
  :global(.confirm-overlay){position:fixed;inset:20px;z-index:var(--z-modal-backdrop);border-radius:var(--radius-xl);background:color-mix(in srgb,var(--text-1) 28%,transparent)}
  :global(.confirm-dialog){position:fixed;z-index:var(--z-modal);top:50%;left:50%;width:min(400px,calc(100vw - 64px));max-height:calc(100vh - 64px);overflow:auto;transform:translate(-50%,-50%);border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised);box-shadow:var(--menu-shadow)}
  .confirm-copy{padding:var(--space-10);overflow-wrap:anywhere}
  :global(.confirm-title){margin:0 0 var(--space-4);color:var(--text-1);font-size:var(--fs-heading);font-weight:680;line-height:var(--lh-tight);text-wrap:balance}
  :global(.confirm-description){margin:0;color:var(--text-2);font-size:var(--fs-ui);line-height:var(--lh-snug);text-wrap:pretty}
  .confirm-error{margin:var(--space-6) 0 0;color:var(--danger);font-size:var(--fs-ui);line-height:var(--lh-snug)}
  :global(.confirm-dialog .action-toolbar){flex-wrap:wrap}
  :global(.confirm-dialog .toolbar-button:focus){outline:2px solid var(--text-1);outline-offset:2px}
  @media(prefers-reduced-motion:no-preference){:global(.confirm-overlay){transition:opacity var(--dur-mid) var(--ease-out)}:global(.confirm-dialog){transition:opacity var(--dur-mid) var(--ease-out),transform var(--dur-mid) var(--ease-out)}:global(.confirm-overlay[data-starting-style]),:global(.confirm-overlay[data-ending-style]){opacity:0}:global(.confirm-dialog[data-starting-style]),:global(.confirm-dialog[data-ending-style]){opacity:0;transform:translate(-50%,-50%) scale(.97)}}
</style>
