<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import TabBar from "./TabBar.svelte";
  import TabBody from "./TabBody.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { uiSettings } from "$lib/settings/uiSettings.svelte";
  import {
    outlines,
    activeHeading,
    collapsedHeadings,
    tabScrollRegistry,
    type Heading,
  } from "$lib/workspace/tabRegistry.svelte";
  import { SvelteSet } from "svelte/reactivity";
  import Icon from "$lib/components/ui/Icon.svelte";

  let { pane }: { pane: PaneType } = $props();

  let bodyEl = $state<HTMLDivElement | undefined>();
  let outlineEl = $state<HTMLElement | undefined>();
  let outlineHover = $state(false);
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

  // Active heading position pushed by the editor's scroll listener.
  // `null` when the editor is scrolled above the first heading (or hasn't
  // reported yet on first mount).
  const activePos = $derived.by(() => {
    if (!activeTab) return null;
    void activeHeading.byTab.size;
    return activeHeading.byTab.get(activeTab.id) ?? null;
  });

  // Per-tab collapsed set (lazy-init). Wrapped in a derived so reads stay
  // reactive when the SvelteMap mutates.
  const collapsedSet = $derived.by<SvelteSet<number>>(() => {
    if (!activeTab) return new SvelteSet<number>();
    void collapsedHeadings.byTab.size;
    let set = collapsedHeadings.byTab.get(activeTab.id);
    if (!set) {
      set = new SvelteSet<number>();
      collapsedHeadings.byTab.set(activeTab.id, set);
    }
    return set;
  });

  // Prune stale collapsed positions whenever the outline changes — once a
  // heading at position P is removed/edited away, P should drop out of the
  // collapsed set so a new heading that happens to land at the same pos
  // doesn't inherit "collapsed" state from a deleted ancestor.
  $effect(() => {
    if (!activeTab) return;
    const valid = new Set(headings.map((h) => h.pos));
    const set = collapsedSet;
    for (const pos of set) {
      if (!valid.has(pos)) set.delete(pos);
    }
  });

  // Walk the flat heading list and stamp each with depth + hasChildren +
  // isHidden. Indentation is by tree depth (not raw heading level) so a
  // doc that starts at H2 still has its top-level H2s flush-left, with H3
  // visually nested under them. Hidden flag flows from any ancestor whose
  // pos lives in `collapsedSet`.
  type FlatNode = Heading & {
    depth: number;
    hasChildren: boolean;
    isHidden: boolean;
  };

  const flatTree = $derived.by<FlatNode[]>(() => {
    const out: FlatNode[] = [];
    // Stack of { level, depth, pos } — stack-top is current ancestor chain.
    const stack: Array<{ level: number; depth: number; pos: number }> = [];
    // Track which depth was last "collapsed by ancestor" so descendants
    // inherit hidden=true even through level skips.
    let hiddenAncestorDepth: number | null = null;

    for (let i = 0; i < headings.length; i++) {
      const h = headings[i];
      // Pop stack entries whose level >= current — they're siblings or
      // upward, not ancestors of this heading.
      while (stack.length > 0 && stack[stack.length - 1].level >= h.level) {
        stack.pop();
      }
      const depth = stack.length;

      // Reset / update the inherited-hidden tracker as we walk back up.
      if (hiddenAncestorDepth !== null && depth <= hiddenAncestorDepth) {
        hiddenAncestorDepth = null;
      }

      // hasChildren: peek ahead — first subsequent heading with higher
      // level (= deeper) before any sibling/upward one is a child.
      let hasChildren = false;
      for (let j = i + 1; j < headings.length; j++) {
        const next = headings[j];
        if (next.level <= h.level) break;
        hasChildren = true;
        break;
      }

      const isHidden = hiddenAncestorDepth !== null;
      out.push({ ...h, depth, hasChildren, isHidden });

      stack.push({ level: h.level, depth, pos: h.pos });

      // If this heading is collapsed AND has children, mark its depth so
      // every descendant in the upcoming iterations gets isHidden=true.
      if (hasChildren && collapsedSet.has(h.pos) && hiddenAncestorDepth === null) {
        hiddenAncestorDepth = depth;
      }
    }
    return out;
  });

  const visibleNodes = $derived(flatTree.filter((n) => !n.isHidden));

  // Two-stage derived so the aside stays mounted across width changes —
  // wantOutline says "this file/preference combo could have an outline",
  // showOutline adds the actual visibility gate (width). When the pane
  // narrows below threshold the aside stays in the DOM and just collapses
  // its grid column + fades, so the transition is smooth instead of a
  // pop-out unmount. Empty-headings doc still mounts the aside — we want
  // the "On this page / no headings yet" placeholder to be visible so the
  // user can see at a glance that outline is on, just empty.
  const wantOutline = $derived(
    uiSettings.showPaneOutline && activeTab?.kind === "markdown",
  );
  const showOutline = $derived(wantOutline && wide);

  function jumpTo(pos: number) {
    if (!activeTab) return;
    tabScrollRegistry.get(activeTab.id)?.(pos);
  }

  function toggleCollapse(pos: number, event: MouseEvent) {
    // Stop propagation so the parent button (jumpTo) doesn't fire too —
    // chevron click is a pure folding action, not navigation.
    event.stopPropagation();
    if (collapsedSet.has(pos)) collapsedSet.delete(pos);
    else collapsedSet.add(pos);
  }

  // Auto-scroll the active heading button into view in the outline aside,
  // but only when the user isn't actively interacting with the outline
  // (would otherwise steal their scroll while they're trying to look at
  // a different section). `block: "nearest"` avoids unnecessary jumps
  // when the active item is already on-screen. We querySelector by
  // data-pos rather than `bind:this` because bind inside an each-block
  // overwrites with each iteration — only the last button would survive.
  $effect(() => {
    void activePos;
    if (activePos === null || !outlineEl || outlineHover) return;
    const btn = outlineEl.querySelector<HTMLElement>(
      `.pane-outline-item[data-pos="${activePos}"]`,
    );
    btn?.scrollIntoView({ block: "nearest" });
  });
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
      <aside
        class="pane-outline"
        bind:this={outlineEl}
        onpointerenter={() => (outlineHover = true)}
        onpointerleave={() => (outlineHover = false)}
      >
        <div class="pane-outline-title mw-meta">On this page</div>
        {#if headings.length === 0}
          <div class="pane-outline-empty">
            No headings yet — add <code>#</code> to a line to start your
            outline.
          </div>
        {:else}
          <ul class="pane-outline-list">
            {#each visibleNodes as n, i (i + ":" + n.pos)}
              <li>
                <div
                  class="pane-outline-row"
                  data-depth={n.depth}
                  class:active={n.pos === activePos}
                >
                  {#if n.hasChildren}
                    <button
                      type="button"
                      class="pane-outline-chevron"
                      aria-label={collapsedSet.has(n.pos) ? "Expand section" : "Collapse section"}
                      onclick={(e) => toggleCollapse(n.pos, e)}
                    >
                      <Icon
                        name={collapsedSet.has(n.pos) ? "chevron-right" : "chevron-down"}
                        size={12}
                      />
                    </button>
                  {:else}
                    <span class="pane-outline-chevron-spacer"></span>
                  {/if}
                  <button
                    type="button"
                    class="pane-outline-item"
                    data-level={n.level}
                    data-pos={n.pos}
                    onclick={() => jumpTo(n.pos)}
                    title={n.text}
                  >
                    {n.text || "(empty heading)"}
                  </button>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
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
  /* Empty-state placeholder. Stays inside the same 200px column so the
     outline width doesn't jump when the user adds the first # heading. */
  .pane-outline-empty {
    color: var(--mw-ink-3);
    font-style: italic;
    line-height: 1.45;
    font-size: 11px;
  }
  .pane-outline-empty code {
    font-family: var(--font-mono);
    background: color-mix(in oklch, var(--color-base-content) 8%, transparent);
    padding: 0 4px;
    border-radius: 3px;
    font-style: normal;
    font-size: 10.5px;
  }
  .pane-outline-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  /* Row = chevron + heading button. Indent here (NOT on the button) so
     the chevron stays glued to the heading text rather than floating
     out at column 1 of every nesting level. */
  .pane-outline-row {
    display: flex;
    align-items: center;
    gap: 2px;
    position: relative;
  }
  .pane-outline-row[data-depth="1"] {
    padding-left: 10px;
  }
  .pane-outline-row[data-depth="2"] {
    padding-left: 20px;
  }
  .pane-outline-row[data-depth="3"] {
    padding-left: 30px;
  }
  .pane-outline-row[data-depth="4"],
  .pane-outline-row[data-depth="5"],
  .pane-outline-row[data-depth="6"] {
    padding-left: 40px;
  }
  /* Active indicator — 2px accent bar bleeding outside the row's left
     padding into the aside's left padding zone. Catches the eye without
     stealing space from the heading text. */
  .pane-outline-row.active::before {
    content: "";
    position: absolute;
    left: -10px;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: var(--mw-accent);
    border-radius: 1px;
  }
  .pane-outline-row.active .pane-outline-item {
    color: var(--color-base-content);
    font-weight: 500;
  }
  .pane-outline-chevron {
    flex-shrink: 0;
    width: 14px;
    height: 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--mw-ink-3);
    border-radius: 3px;
    padding: 0;
  }
  .pane-outline-chevron:hover {
    color: var(--color-base-content);
    background: color-mix(in oklch, var(--color-base-content) 8%, transparent);
  }
  /* Spacer keeps leaf rows visually aligned with rows that have a chevron
     — without it, leaves would shift 14px left of their siblings. */
  .pane-outline-chevron-spacer {
    flex-shrink: 0;
    display: inline-block;
    width: 14px;
    height: 14px;
  }
  .pane-outline-item {
    flex: 1;
    min-width: 0;
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
  .pane-outline-item[data-level="3"] {
    font-size: 11px;
  }
  .pane-outline-item[data-level="4"],
  .pane-outline-item[data-level="5"],
  .pane-outline-item[data-level="6"] {
    font-size: 10.5px;
  }
</style>
