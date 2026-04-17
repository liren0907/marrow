<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import TabBar from "./TabBar.svelte";
  import TabBody from "./TabBody.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { uiSettings } from "$lib/settings/uiSettings.svelte";

  let { pane }: { pane: PaneType } = $props();

  function focus() {
    workspace.setActivePane(pane.id);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="flex-1 flex flex-col min-w-0 min-h-0"
  class:pane-active={workspace.panes.length > 1 && workspace.activePaneId === pane.id}
  onmousedown={focus}
>
  <TabBar {pane} />
  {#if uiSettings.showBreadcrumb && pane.activeTabId}
    <Breadcrumb {pane} />
  {/if}
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

<style>
  .pane-active {
    box-shadow: inset 0 2px 0 0 var(--color-primary);
  }
</style>
