<script lang="ts">
  import {
    bottomPanel,
    persistBottomPanel,
    toggleBottomPanel,
  } from "./bottomPanelState.svelte";
  import { backlinks } from "$lib/workspace/backlinkIndex.svelte";
  import { tags } from "$lib/workspace/tagIndex.svelte";
  import BacklinksTab from "./BacklinksTab.svelte";
  import UnresolvedTab from "./UnresolvedTab.svelte";
  import TagsTab from "./TagsTab.svelte";
  import OutlineTab from "./OutlineTab.svelte";
  import PeekPanel from "$lib/peek/PeekPanel.svelte";
  import { peek } from "$lib/peek/peekState.svelte";
  import type { ActiveTab } from "./bottomPanelState.svelte";

  const MIN_HEIGHT = 120;
  const MAX_HEIGHT_FRACTION = 0.6;

  let dragging = $state(false);

  function clampHeight(h: number): number {
    const max = typeof window !== "undefined"
      ? window.innerHeight * MAX_HEIGHT_FRACTION
      : 800;
    return Math.max(MIN_HEIGHT, Math.min(max, h));
  }

  function onResizeStart(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
    const startY = e.clientY;
    const startHeight = bottomPanel.height;

    function onMove(ev: MouseEvent) {
      const dy = startY - ev.clientY;
      bottomPanel.height = clampHeight(startHeight + dy);
    }
    function onUp() {
      dragging = false;
      persistBottomPanel();
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  function onResizeKey(e: KeyboardEvent) {
    const step = e.shiftKey ? 64 : 16;
    if (e.key === "ArrowUp") {
      e.preventDefault();
      bottomPanel.height = clampHeight(bottomPanel.height + step);
      persistBottomPanel();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      bottomPanel.height = clampHeight(bottomPanel.height - step);
      persistBottomPanel();
    } else if (e.key === "Home") {
      e.preventDefault();
      bottomPanel.height = 220;
      persistBottomPanel();
    }
  }

  function selectTab(tab: ActiveTab) {
    bottomPanel.activeTab = tab;
    persistBottomPanel();
  }
</script>

<div
  class="bottom-panel border-t border-base-200 bg-base-100 flex flex-col shrink-0"
  style:height="{bottomPanel.height}px"
>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="resize-handle"
    class:dragging
    onmousedown={onResizeStart}
    onkeydown={onResizeKey}
    role="separator"
    aria-orientation="horizontal"
    aria-label="Resize bottom panel"
    aria-valuenow={Math.round(bottomPanel.height)}
    aria-valuemin={MIN_HEIGHT}
    tabindex="0"
  ></div>
  <div
    class="flex items-center justify-between px-2 py-1 border-b border-base-200 shrink-0"
  >
    <div class="flex items-center gap-0">
      <button
        type="button"
        class="bp-tab"
        class:active={bottomPanel.activeTab === "backlinks"}
        onclick={() => selectTab("backlinks")}
      >
        Backlinks
      </button>
      <button
        type="button"
        class="bp-tab"
        class:active={bottomPanel.activeTab === "unresolved"}
        onclick={() => selectTab("unresolved")}
      >
        Unresolved
        {#if backlinks.unresolvedBySource.size > 0}
          <span class="bp-tab-count">({backlinks.unresolvedBySource.size})</span>
        {/if}
      </button>
      <button
        type="button"
        class="bp-tab"
        class:active={bottomPanel.activeTab === "tags"}
        onclick={() => selectTab("tags")}
      >
        Tags
        {#if tags.byTag.size > 0}
          <span class="bp-tab-count">({tags.byTag.size})</span>
        {/if}
      </button>
      <button
        type="button"
        class="bp-tab"
        class:active={bottomPanel.activeTab === "outline"}
        onclick={() => selectTab("outline")}
      >
        Outline
      </button>
      <button
        type="button"
        class="bp-tab"
        class:active={bottomPanel.activeTab === "peek"}
        onclick={() => selectTab("peek")}
      >
        Peek
        {#if peek.depth > 0}
          <span class="bp-tab-count">({peek.depth})</span>
        {/if}
      </button>
      {#if backlinks.isBuilding || tags.isBuilding}
        <span class="ml-2 text-[10px] text-base-content/40 italic">
          building index…
        </span>
      {/if}
    </div>
    <button
      type="button"
      class="btn btn-ghost btn-xs btn-square"
      title="Close (Cmd+J)"
      onclick={toggleBottomPanel}
    >
      <span class="material-symbols-rounded text-[16px]">close</span>
    </button>
  </div>
  {#if bottomPanel.activeTab === "backlinks"}
    <BacklinksTab />
  {:else if bottomPanel.activeTab === "unresolved"}
    <UnresolvedTab />
  {:else if bottomPanel.activeTab === "tags"}
    <TagsTab />
  {:else if bottomPanel.activeTab === "outline"}
    <OutlineTab />
  {:else if bottomPanel.activeTab === "peek"}
    <PeekPanel />
  {/if}
</div>

<style>
  .resize-handle {
    position: absolute;
    top: -3px;
    left: 0;
    right: 0;
    height: 6px;
    cursor: row-resize;
    z-index: 5;
  }
  .resize-handle:hover,
  .resize-handle.dragging,
  .resize-handle:focus-visible {
    background-color: color-mix(in oklch, var(--color-primary) 30%, transparent);
    outline: none;
  }
  .bottom-panel {
    position: relative;
  }
  .bp-tab {
    padding: 3px 10px;
    font-size: 11px;
    color: var(--mw-ink-2);
    cursor: pointer;
    background: transparent;
    border: none;
    position: relative;
  }
  .bp-tab:hover {
    color: var(--mw-ink-1);
  }
  .bp-tab.active {
    color: var(--color-base-content);
  }
  .bp-tab.active::after {
    content: "";
    position: absolute;
    left: 6px;
    right: 6px;
    bottom: -2px;
    height: 1px;
    background: var(--mw-accent);
  }
  .bp-tab-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--mw-ink-3);
    margin-left: 3px;
  }
</style>
