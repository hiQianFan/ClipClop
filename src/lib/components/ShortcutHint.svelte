<script lang="ts">
  import { currentPlatform, shortcutKeycaps, shortcutSpokenLabel, type ShortcutPlatform } from "$lib/settings/shortcuts";

  let {
    shortcut,
    platform = currentPlatform(),
    variant = "accelerator",
    label,
    inherit = false,
  }: {
    shortcut: string;
    platform?: ShortcutPlatform;
    variant?: "accelerator" | "compact" | "keycaps";
    label?: string;
    inherit?: boolean;
  } = $props();

  const keys = $derived(shortcutKeycaps(shortcut, platform));
</script>

<kbd class:accelerator={variant === "accelerator"} class:compact={variant === "compact"} class:keycaps={variant === "keycaps"} class:inherit aria-label={label ?? shortcutSpokenLabel(shortcut, platform)}>
  {#each keys as key, index}
    {#if variant === "accelerator" && platform === "windows" && index > 0}<span class="plus" aria-hidden="true">+</span>{/if}
    <span class:keycap={variant === "keycaps"} aria-hidden="true">{key}</span>
  {/each}
</kbd>

<style>
  kbd{display:inline-flex;align-items:center;white-space:nowrap;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}
  .accelerator{gap:3px;padding:0;border:0;color:var(--text-3);background:transparent;font-size:var(--fs-ui);font-weight:500;line-height:1}
  .accelerator.inherit{color:inherit;opacity:.68}
  .plus{margin-inline:-1px;color:var(--text-3);font-size:var(--fs-caption);font-weight:500}
  .compact{gap:2px;padding:1px 5px;border:1px solid var(--hairline);border-radius:var(--radius-sm);color:var(--text-2);background:transparent;font-size:var(--fs-caption);font-weight:500;line-height:var(--lh-snug)}
  .compact.inherit{color:inherit;border-color:currentColor}
  .keycaps{gap:4px;border:0;background:transparent}
  .keycap{min-width:22px;height:22px;display:inline-grid;place-items:center;padding:0 5px;border:1px solid var(--hairline);border-radius:var(--radius-sm);color:var(--text-1);background:var(--bg-raised);font-size:var(--fs-ui);font-weight:500;line-height:1}
</style>
