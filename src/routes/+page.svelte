<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { openDirectoryDialog } from "$lib/workspace/tauri";
  import { Button } from "$lib/components/ui";
  import PaneContainer from "$lib/panes/PaneContainer.svelte";
  import RecentList from "$lib/workspace/RecentList.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  async function pickFolder() {
    try {
      const path = await openDirectoryDialog();
      if (path) await workspace.open(path);
    } catch (e) {
      showError(
        `Failed to open folder: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }
</script>

<svelte:head>
  <title>Marrow</title>
</svelte:head>

{#if workspace.info}
  <PaneContainer />
{:else}
  <div class="flex-1 flex items-center justify-center p-8 overflow-y-auto">
    <div class="flex flex-col items-center gap-4 text-center max-w-md">
      <Icon name="folder-open" size={72} strokeWidth={1.25} class="text-base-content/20" />
      <h1 class="text-xl font-semibold">No workspace open</h1>
      <p class="text-sm text-base-content/50">
        Drag a folder onto this window, or click below to pick one. Markdown
        files will open in the editor; images, video, audio, and PDF get
        in-app previews.
      </p>
      <Button onclick={pickFolder}>
        <Icon name="folder-open" size={16} class="mr-1" />
        Open folder…
      </Button>
      <RecentList />
    </div>
  </div>
{/if}
