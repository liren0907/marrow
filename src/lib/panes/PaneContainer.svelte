<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import Pane from "./Pane.svelte";
</script>

<div class="flex-1 flex flex-row min-h-0 min-w-0">
  {#each workspace.panes as pane (pane.id)}
    <Pane {pane} />
  {/each}
</div>
