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
        // 960px threshold — on 13"–14" laptops with the sidebar expanded a
        // pane often hovers around 800–900px; raising from the legacy 720px
        // means the outline only appears once the user has actually
        // committed pane width to it, instead of cramming the editor.
        wide = e.contentRect.width >= 960;
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

  // Two-stage derived so the aside stays mounted across width changes —
  // wantOutline says "this file/preference combo could have an outline",
  // showOutline adds the actual visibility gate (width). When the pane
  // narrows below threshold the aside stays in the DOM and just collapses
  // its grid column + fades, so the transition is smooth instead of a
  // pop-out unmount.
  const wantOutline = $derived(
    uiSettings.showPaneOutline &&
      activeTab?.kind === "markdown" &&
      headings.length > 0,
  );
  const showOutline = $derived(wantOutline && wide);

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
    <!-- Outline aside — explicit grid-column: 1 places it on the LEFT.
         Source-order first so it auto-flows there even without the
         explicit rule, but we set both for clarity. -->
    {#if wantOutline}
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

    <!-- TabBody wrapper — its `position: relative` is the new containing
         block for TabBody's `absolute inset-0` tab divs, so they fill
         only the editor column instead of bleeding across the whole
         pane-body (the bug that put aside's tint behind the editor). -->
    <div class="pane-tabbody-wrapper">
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
    </div>
  </div>
</div>

<style>
  .pane-active {
    box-shadow: inset 0 2px 0 0 var(--mw-accent);
  }
  .pane-body {
    display: grid;
    /* Outline column on the LEFT, editor on the RIGHT.
       Collapsed state = 0 width left column + 1fr editor. */
    grid-template-columns: 0 1fr;
    transition: grid-template-columns 0.15s ease;
  }
  .pane-body.show-outline {
    grid-template-columns: 200px 1fr;
  }
  /* TabBody wrapper — explicit grid-column: 2 keeps the editor in the
     RIGHT column even when the aside isn't rendered (otherwise the only
     flow item would auto-flow to column 1 = the 0px collapsed lane).
     `position: relative` makes it the containing block for TabBody's
     `absolute inset-0` tab divs, so they no longer cover the whole
     pane-body. */
  .pane-tabbody-wrapper {
    grid-column: 2;
    grid-row: 1;
    position: relative;
    min-width: 0;
    min-height: 0;
  }
  .pane-outline {
    /* Explicit placement matches markup intent — column 1, LEFT. */
    grid-column: 1;
    grid-row: 1;
    /* Background + border use color-mix against --color-base-content rather
       than --color-base-200 / --color-base-300 tokens. Reason: the default
       marrow-pro-light theme inverts the standard daisyUI lightness ladder
       (its base-300 is LIGHTER than base-100, on purpose, for the cream-
       paper aesthetic). Mixing alpha against base-content always lands on
       "slightly darker than the editor" in every theme regardless of how
       the lightness tokens are arranged. The opacity transition pairs with
       .pane-body's grid-template-columns transition so crossing the 960px
       threshold fades smoothly instead of popping. */
    border-right: 1px solid color-mix(in oklch, var(--color-base-content) 12%, transparent);
    background: color-mix(in oklch, var(--color-base-content) 4%, transparent);
    /* Padding tilted to the right (toward editor) so outline text isn't
       glued to the divider line; left side hugs the pane edge tighter. */
    padding: 28px 22px 28px 16px;
    overflow-y: auto;
    font-size: 11.5px;
    min-width: 0;
    opacity: 1;
    transition: opacity 0.15s ease;
  }
  .pane-body:not(.show-outline) .pane-outline {
    opacity: 0;
    pointer-events: none;
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
