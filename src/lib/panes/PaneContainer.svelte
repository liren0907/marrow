<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import Pane from "./Pane.svelte";
  import PaneResizer from "./PaneResizer.svelte";
</script>

<div class="flex-1 flex flex-row min-h-0 min-w-0">
  {#each workspace.panes as pane, i (pane.id)}
    <div
      class="flex flex-col min-h-0 min-w-0"
      style="flex: {workspace.panes.length === 1
        ? '1 1 100%'
        : i === 0
          ? `${workspace.splitRatio} 1 0`
          : `${1 - workspace.splitRatio} 1 0`};"
    >
      <Pane {pane} />
    </div>
    {#if i === 0 && workspace.panes.length > 1}
      <PaneResizer />
    {/if}
  {/each}
</div>
