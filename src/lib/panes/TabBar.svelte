<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import Tab from "./Tab.svelte";

  let { pane }: { pane: PaneType } = $props();

  let dropIndex = $state<number | null>(null);
  let barEl: HTMLDivElement | undefined = $state();

  function computeDropIndex(e: DragEvent): number {
    if (!barEl) return pane.tabs.length;
    const tabs = barEl.querySelectorAll<HTMLElement>("[data-tab-id]");
    const x = e.clientX;
    for (let i = 0; i < tabs.length; i++) {
      const r = tabs[i].getBoundingClientRect();
      if (x < r.left + r.width / 2) return i;
    }
    return tabs.length;
  }

  function onDragOver(e: DragEvent) {
    if (!e.dataTransfer) return;
    if (!Array.from(e.dataTransfer.types).includes("application/x-marrow-tab")) {
      return;
    }
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    dropIndex = computeDropIndex(e);
  }

  function onDragLeave(e: DragEvent) {
    // Only clear if leaving the bar entirely.
    if (!barEl) return;
    const r = barEl.getBoundingClientRect();
    const x = e.clientX;
    const y = e.clientY;
    if (x < r.left || x > r.right || y < r.top || y > r.bottom) {
      dropIndex = null;
    }
  }

  function onDrop(e: DragEvent) {
    if (!e.dataTransfer) return;
    const raw = e.dataTransfer.getData("application/x-marrow-tab");
    if (!raw) return;
    e.preventDefault();
    try {
      const { srcPaneId, tabId } = JSON.parse(raw) as {
        srcPaneId: string;
        tabId: string;
      };
      const idx = computeDropIndex(e);
      workspace.moveTab(srcPaneId, tabId, pane.id, idx);
    } catch (err) {
      console.warn("[TabBar] drop parse failed", err);
    } finally {
      dropIndex = null;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={barEl}
  class="relative flex flex-row items-stretch h-9 border-b border-base-200 bg-base-100 overflow-x-auto shrink-0 tab-bar"
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
>
  {#each pane.tabs as tab, i (tab.id)}
    {#if dropIndex === i}
      <div class="drop-indicator"></div>
    {/if}
    <Tab {tab} paneId={pane.id} active={tab.id === pane.activeTabId} />
  {/each}
  {#if dropIndex === pane.tabs.length}
    <div class="drop-indicator"></div>
  {/if}
</div>

<style>
  .tab-bar {
    scrollbar-width: none;
  }
  .tab-bar::-webkit-scrollbar {
    display: none;
  }
  .drop-indicator {
    width: 2px;
    background-color: oklch(var(--p));
    flex-shrink: 0;
  }
</style>
