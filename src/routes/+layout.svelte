<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { scheduleAutomaticUpdateCheck } from "$lib/updater/api";
  let { children } = $props();

  onMount(() => scheduleAutomaticUpdateCheck());
</script>

{@render children()}
