<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import TabBar from "./TabBar.svelte";
  import TabBody from "./TabBody.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";

  let { pane }: { pane: PaneType } = $props();

  function focus() {
    workspace.setActivePane(pane.id);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="flex-1 flex flex-col min-w-0 min-h-0 border-r border-base-200 last:border-r-0"
  onmousedown={focus}
>
  <TabBar {pane} />
  <div class="flex-1 min-h-0 relative overflow-hidden">
    {#if pane.tabs.length === 0}
      <div
        class="absolute inset-0 flex items-center justify-center text-base-content/30 text-sm"
      >
        <div class="flex flex-col items-center gap-3">
          <span class="material-symbols-rounded text-[48px]">article</span>
          <p>Open a file from the sidebar</p>
        </div>
      </div>
    {:else}
      <TabBody {pane} />
    {/if}
  </div>
</div>
