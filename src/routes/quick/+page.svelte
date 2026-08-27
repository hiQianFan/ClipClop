<script lang="ts">
  import QuickPanel from "$lib/history/QuickPanel.svelte";
  import { showFullPanel } from "$lib/history/api";

  let selectedId = $state<string | null>(null);
</script>

<QuickPanel
  bind:selectedId
  onfull={() => void showFullPanel(selectedId)}
  onsettings={() => void showFullPanel(selectedId, true)}
/>
