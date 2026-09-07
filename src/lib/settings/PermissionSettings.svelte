<script lang="ts">
  import PermissionControls from "$lib/onboarding/PermissionControls.svelte";
  import { t } from "$lib/i18n/index.svelte";
  let { onerror, heading = $bindable(), active = false, focusRequested = false, beforeRestart = async () => true }: {
    onerror: (message: string) => void; heading?: HTMLHeadingElement; active?: boolean; focusRequested?: boolean;
    beforeRestart?: () => Promise<boolean>;
  } = $props();
</script>
<h1 bind:this={heading} id="settings-section-title" tabindex="-1">{t("settings.permissions")}</h1>
<PermissionControls {active} {focusRequested} {onerror} {beforeRestart} />
<style>
  h1{margin:18px 0 4px;font-size:var(--fs-heading);font-weight:680;line-height:1.3;letter-spacing:-.01em}h1:focus{outline:none}
</style>
