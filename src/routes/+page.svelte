<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { openDirectoryDialog } from "$lib/workspace/tauri";
  import { Button } from "$lib/components/ui";
  import PaneContainer from "$lib/panes/PaneContainer.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";

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
  <div class="flex-1 flex items-center justify-center p-8">
    <div class="flex flex-col items-center gap-4 text-center max-w-md">
      <span class="material-symbols-rounded text-[72px] text-base-content/20"
        >folder_open</span
      >
      <h1 class="text-xl font-semibold">No workspace open</h1>
      <p class="text-sm text-base-content/50">
        Drag a folder onto this window, or click below to pick one. Markdown
        files will open in the editor; images, video, audio, and PDF get
        in-app previews.
      </p>
      <Button onclick={pickFolder}>
        <span class="material-symbols-rounded text-[16px] mr-1">folder_open</span
        >
        Open folder…
      </Button>
    </div>
  </div>
{/if}
