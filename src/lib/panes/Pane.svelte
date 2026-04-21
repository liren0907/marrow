<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import TabBar from "./TabBar.svelte";
  import TabBody from "./TabBody.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { uiSettings } from "$lib/settings/uiSettings.svelte";
  import {
    outlines,
    tabScrollRegistry,
  } from "$lib/workspace/tabRegistry.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  let { pane }: { pane: PaneType } = $props();

  let bodyEl = $state<HTMLDivElement | undefined>();
  let wide = $state(false);

  function focus() {
    workspace.setActivePane(pane.id);
  }

  $effect(() => {
    const el = bodyEl;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        wide = e.contentRect.width >= 720;
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  });

  const activeTab = $derived.by(() => {
    if (!pane.activeTabId) return null;
    return pane.tabs.find((t) => t.id === pane.activeTabId) ?? null;
  });

  const headings = $derived.by(() => {
    if (!activeTab || activeTab.kind !== "markdown") return [];
    void outlines.byTab.size;
    return outlines.byTab.get(activeTab.id) ?? [];
  });

  const showOutline = $derived(
    uiSettings.showPaneOutline &&
      wide &&
      activeTab?.kind === "markdown" &&
      headings.length > 0,
  );

  function jumpTo(pos: number) {
    if (!activeTab) return;
    tabScrollRegistry.get(activeTab.id)?.(pos);
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
  <div
    bind:this={bodyEl}
    class="flex-1 min-h-0 relative overflow-hidden pane-body"
    class:show-outline={showOutline}
  >
    {#if pane.tabs.length === 0}
      <div
        class="absolute inset-0 flex items-center justify-center text-base-content/30 text-sm"
      >
        <div class="flex flex-col items-center gap-3">
          <Icon name="file-text" size={48} strokeWidth={1.25} />
          <p>Open a file from the sidebar</p>
        </div>
      </div>
    {:else}
      <TabBody {pane} />
    {/if}

    {#if showOutline}
      <aside class="pane-outline">
        <div class="pane-outline-title mw-meta">On this page</div>
        <ul class="pane-outline-list">
          {#each headings as h, i (i + ":" + h.pos)}
            <li>
              <button
                type="button"
                class="pane-outline-item"
                data-level={h.level}
                onclick={() => jumpTo(h.pos)}
                title={h.text}
              >
                {h.text || "(empty heading)"}
              </button>
            </li>
          {/each}
        </ul>
      </aside>
    {/if}
  </div>
</div>

<style>
  .pane-active {
    box-shadow: inset 0 2px 0 0 var(--mw-accent);
  }
  .pane-body {
    display: grid;
    grid-template-columns: 1fr 0;
    transition: grid-template-columns 0.15s ease;
  }
  .pane-body.show-outline {
    grid-template-columns: 1fr 200px;
  }
  .pane-outline {
    border-left: 1px solid var(--mw-rule);
    background: var(--color-base-100);
    padding: 28px 16px;
    overflow-y: auto;
    font-size: 11.5px;
    min-width: 0;
  }
  .pane-outline-title {
    margin-bottom: 10px;
  }
  .pane-outline-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .pane-outline-item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 3px 0;
    color: var(--mw-ink-2);
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: var(--font-ui);
    font-size: 11.5px;
  }
  .pane-outline-item:hover {
    color: var(--color-base-content);
  }
  .pane-outline-item[data-level="1"] {
    font-weight: 600;
    color: var(--mw-ink-1);
  }
  .pane-outline-item[data-level="2"] {
    padding-left: 10px;
  }
  .pane-outline-item[data-level="3"] {
    padding-left: 20px;
    font-size: 11px;
  }
  .pane-outline-item[data-level="4"],
  .pane-outline-item[data-level="5"],
  .pane-outline-item[data-level="6"] {
    padding-left: 30px;
    font-size: 10.5px;
  }
</style>
