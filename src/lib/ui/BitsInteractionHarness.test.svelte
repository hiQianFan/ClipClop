<script lang="ts">
  import { AlertDialog, DropdownMenu, Progress, Tabs } from "bits-ui";

  let menuOpen = $state(false);
  let dialogOpen = $state(false);
  let tab = $state("first");
  let menuTrigger = $state<HTMLButtonElement | null>(null);
  let confirmButton = $state<HTMLButtonElement | null>(null);
</script>

<DropdownMenu.Root bind:open={menuOpen}>
  <DropdownMenu.Trigger bind:ref={menuTrigger}>Actions</DropdownMenu.Trigger>
  <DropdownMenu.ContentStatic loop={true} onCloseAutoFocus={(event) => { event.preventDefault(); menuTrigger?.focus(); }}>
    <DropdownMenu.Item>Copy</DropdownMenu.Item>
    <DropdownMenu.Item onclick={() => dialogOpen = true}>Delete</DropdownMenu.Item>
  </DropdownMenu.ContentStatic>
</DropdownMenu.Root>

<Tabs.Root bind:value={tab} activationMode="automatic" loop={false}>
  <Tabs.List aria-label="Files">
    <Tabs.Trigger value="first">First</Tabs.Trigger>
    <Tabs.Trigger value="second">Second</Tabs.Trigger>
  </Tabs.List>
  <Tabs.Content value="first">First file</Tabs.Content>
  <Tabs.Content value="second">Second file</Tabs.Content>
</Tabs.Root>

<AlertDialog.Root bind:open={dialogOpen}>
  <AlertDialog.Trigger>Clear</AlertDialog.Trigger>
  <AlertDialog.Content data-inline-confirmation preventScroll={false} onOpenAutoFocus={(event) => { event.preventDefault(); confirmButton?.focus(); }}>
    <AlertDialog.Title>Clear history?</AlertDialog.Title>
    <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
    <AlertDialog.Action bind:ref={confirmButton}>Confirm</AlertDialog.Action>
  </AlertDialog.Content>
</AlertDialog.Root>

<Progress.Root value={40} max={100} aria-label="Download progress" />
<Progress.Root value={null} max={100} aria-label="Preparing download" />
