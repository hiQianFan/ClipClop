<script lang="ts">
  import type { Snippet } from "svelte";

  let { children, class: className = "" }: { children?: Snippet; class?: string } = $props();
</script>

<footer class={`action-toolbar ${className}`}>{@render children?.()}</footer>

<style>
  .action-toolbar{min-height:49px;display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:8px 16px;border-top:1px solid var(--hairline)}
  :global(.action-toolbar .toolbar-button){min-width:92px;min-height:32px;display:flex;align-items:center;justify-content:center;gap:6px;padding:0 12px;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}
  :global(.action-toolbar .toolbar-button:hover){color:var(--text-1);background:var(--bg-hover)}
  :global(.action-toolbar .toolbar-button.secondary){border:1px solid var(--hairline)}
  :global(.action-toolbar .toolbar-button:focus-visible){outline:2px solid var(--text-1);outline-offset:2px}
  :global(.action-toolbar .toolbar-button.primary){color:var(--action-on);background:var(--action);font-weight:650}
  :global(.action-toolbar .toolbar-button.primary:hover){color:var(--action-on);background:var(--action-hover)}
  :global(.action-toolbar .toolbar-button.destructive){color:var(--danger-on);background:var(--danger-fill);font-weight:600}
  :global(.action-toolbar .toolbar-button.destructive:hover){color:var(--danger-on);filter:brightness(1.08)}
  :global(.action-toolbar .toolbar-button:disabled){opacity:.45;cursor:not-allowed}
</style>
