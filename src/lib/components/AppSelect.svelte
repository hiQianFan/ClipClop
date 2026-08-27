<script lang="ts">
  import { Select } from "bits-ui";
  import { Check, ChevronDown } from "@lucide/svelte";

  export type AppSelectItem = { value: string; label: string; disabled?: boolean };

  let {
    value,
    items,
    ariaLabel,
    onchange,
  }: {
    value: string;
    items: AppSelectItem[];
    ariaLabel: string;
    onchange: (value: string) => void;
  } = $props();
</script>

<Select.Root type="single" {value} {items} onValueChange={onchange}>
  <Select.Trigger class="select-trigger" aria-label={ariaLabel}>
    <Select.Value />
    <ChevronDown size={14} strokeWidth={2} aria-hidden="true" />
  </Select.Trigger>
  <Select.Portal>
    <Select.Content class="select-content" sideOffset={4} align="end">
      <Select.Viewport class="select-viewport">
        {#each items as item (item.value)}
          <Select.Item class="select-item" value={item.value} label={item.label} disabled={item.disabled}>
            {#snippet children({ selected })}
              <span>{item.label}</span>
              {#if selected}<Check size={14} strokeWidth={2.25} aria-hidden="true" />{/if}
            {/snippet}
          </Select.Item>
        {/each}
      </Select.Viewport>
    </Select.Content>
  </Select.Portal>
</Select.Root>

<style>
  :global(.select-trigger){min-width:156px;height:32px;display:inline-flex;align-items:center;justify-content:space-between;gap:12px;padding:0 9px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-1);background:var(--bg-raised);font-size:var(--fs-ui);font-weight:500;text-align:left;cursor:default;user-select:none;-webkit-user-select:none}
  :global(.select-trigger:hover){background:var(--bg-hover)}
  :global(.select-trigger:focus-visible){outline:2px solid var(--text-1);outline-offset:2px}
  :global(.select-trigger svg){flex:none;color:var(--text-3);transition:transform var(--dur-fast) var(--ease-out)}
  :global(.select-trigger[data-state="open"] svg){transform:rotate(180deg)}
  :global(.select-content){z-index:var(--z-menu);min-width:var(--bits-select-anchor-width);max-width:min(280px,var(--bits-select-content-available-width));padding:4px;border:1px solid var(--hairline);border-radius:var(--radius-lg);color:var(--text-1);background:var(--bg-raised);box-shadow:var(--menu-shadow);transform-origin:var(--bits-select-content-transform-origin);cursor:default;user-select:none;-webkit-user-select:none}
  :global(.select-content *){cursor:default;user-select:none;-webkit-user-select:none}
  :global(.select-viewport){max-height:min(280px,var(--bits-select-content-available-height));outline:none}
  :global(.select-item){min-height:30px;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:0 7px;border-radius:var(--radius-md);font-size:var(--fs-ui);font-weight:500;outline:none;user-select:none}
  :global(.select-item[data-highlighted]){background:var(--bg-hover)}
  :global(.select-item[data-disabled]){opacity:.45}
  :global(.select-item svg){flex:none;color:var(--text-2)}
  @media(prefers-reduced-motion:no-preference){:global(.select-content){transition:opacity var(--dur-fast) var(--ease-out),transform var(--dur-fast) var(--ease-out)}:global(.select-content[data-starting-style]),:global(.select-content[data-ending-style]){opacity:0;transform:scale(.98)}}
</style>
