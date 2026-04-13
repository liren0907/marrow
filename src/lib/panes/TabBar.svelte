<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import Tab from "./Tab.svelte";

  let { pane }: { pane: PaneType } = $props();
</script>

<div
  class="flex flex-row items-stretch h-9 border-b border-base-200 bg-base-100 overflow-x-auto shrink-0 tab-bar"
>
  {#each pane.tabs as tab (tab.id)}
    <Tab {tab} paneId={pane.id} active={tab.id === pane.activeTabId} />
  {/each}
</div>

<style>
  .tab-bar {
    scrollbar-width: none;
  }
  .tab-bar::-webkit-scrollbar {
    display: none;
  }
</style>
