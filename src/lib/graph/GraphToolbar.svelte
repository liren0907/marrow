<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
  import ToggleButtonGroup from "$lib/components/ui/ToggleButtonGroup.svelte";

  type ViewMode = "all" | "local-1" | "local-2";
  type ColorMode = "default" | "folder" | "tag";
  type NodeSizePreset = "xs" | "sm" | "md" | "lg" | "xl";
  type LabelSizePreset = "hidden" | "xs" | "sm" | "md" | "lg";
  type LabelMode = "always" | "hover";

  let {
    viewMode = $bindable<ViewMode>(),
    colorMode = $bindable<ColorMode>(),
    folderFilter = $bindable<string[]>(),
    tagFilter = $bindable<string[]>(),
    searchFilter = $bindable<string>(),
    nodeSize = $bindable<NodeSizePreset>(),
    labelSize = $bindable<LabelSizePreset>(),
    labelMode = $bindable<LabelMode>(),
    showEdgeArrows = $bindable<boolean>(),
    edgeWidth = $bindable<number>(),
    folderOptions,
    tagOptions,
    onReset,
    onFit,
    onResetLayout,
  }: {
    viewMode: ViewMode;
    colorMode: ColorMode;
    folderFilter: string[];
    tagFilter: string[];
    searchFilter: string;
    nodeSize: NodeSizePreset;
    labelSize: LabelSizePreset;
    labelMode: LabelMode;
    showEdgeArrows: boolean;
    edgeWidth: number;
    folderOptions: string[];
    tagOptions: string[];
    onReset: () => void;
    onFit: () => void;
    onResetLayout: () => void;
  } = $props();

  const SCOPE_LABELS: Record<ViewMode, string> = {
    "all": "All",
    "local-1": "±1 (direct neighbors)",
    "local-2": "±2 (neighbors of neighbors)",
  };

  const SCOPE_OPTIONS: {
    id: ViewMode;
    label: string;
    description: string;
  }[] = [
    { id: "all", label: "All", description: "Show full vault" },
    {
      id: "local-1",
      label: "±1",
      description: "Direct neighbors of active note",
    },
    { id: "local-2", label: "±2", description: "Neighbors of neighbors" },
  ];

  const NODE_SIZE_OPTIONS = [
    { value: "xs", label: "XS" },
    { value: "sm", label: "S" },
    { value: "md", label: "M" },
    { value: "lg", label: "L" },
    { value: "xl", label: "XL" },
  ];

  const LABEL_SIZE_OPTIONS = [
    { value: "hidden", label: "—" },
    { value: "xs", label: "XS" },
    { value: "sm", label: "S" },
    { value: "md", label: "M" },
    { value: "lg", label: "L" },
  ];

  const EDGE_WIDTH_OPTIONS = [
    { value: 1, label: "1px" },
    { value: 2, label: "2px" },
    { value: 3, label: "3px" },
  ];

  const COLOR_LABELS: Record<ColorMode, string> = {
    default: "Default",
    folder: "By folder",
    tag: "By tag",
  };

  const COLOR_OPTIONS: { id: ColorMode; label: string }[] = [
    { id: "default", label: "Default" },
    { id: "folder", label: "By folder" },
    { id: "tag", label: "By tag" },
  ];

  const hasActiveFilters = $derived(
    viewMode !== "all" ||
      folderFilter.length > 0 ||
      tagFilter.length > 0 ||
      searchFilter.trim() !== "",
  );

  function toggleFolder(name: string) {
    folderFilter = folderFilter.includes(name)
      ? folderFilter.filter((f) => f !== name)
      : [...folderFilter, name];
  }
  function toggleTag(name: string) {
    tagFilter = tagFilter.includes(name)
      ? tagFilter.filter((t) => t !== name)
      : [...tagFilter, name];
  }

  // Each popover holds its own open state. Opening one doesn't auto-close
  // the others, but Popover's window-level mousedown handler will close any
  // open popover when the user clicks a different trigger — so in practice
  // only one is open at a time without explicit mutex logic.
  let scopeOpen = $state(false);
  let colorOpen = $state(false);
  let foldersOpen = $state(false);
  let tagsOpen = $state(false);
  let settingsOpen = $state(false);
</script>

<div class="mw-gtb">
  <!-- Group 1: Filtering — scope / color / folders / tags.
       All four shape WHAT the graph shows; grouped together so the eye reads
       them as one cluster. -->
  <div class="mw-gtb-group">
    <!-- Scope -->
    <div class="tooltip tooltip-bottom" data-tip="Scope: {SCOPE_LABELS[viewMode]}">
      <Popover
        bind:open={scopeOpen}
        triggerClass={`mw-gtb-icon-trigger${viewMode !== "all" ? " is-active" : ""}`}
        panelClass="mw-gtb-panel"
        triggerAriaLabel="Scope: {SCOPE_LABELS[viewMode]}"
      >
        {#snippet trigger()}
          <Icon name="target" size={14} />
        {/snippet}
        {#snippet children()}
          {#each SCOPE_OPTIONS as opt (opt.id)}
            <label class="mw-gtb-row mw-gtb-row-stack">
              <input
                type="radio"
                name="gtb-scope"
                class="radio radio-xs"
                checked={viewMode === opt.id}
                onchange={() => (viewMode = opt.id)}
              />
              <span class="mw-gtb-row-text">
                <span class="mw-gtb-row-title">{opt.label}</span>
                <span class="mw-gtb-row-desc">{opt.description}</span>
              </span>
            </label>
          {/each}
        {/snippet}
      </Popover>
    </div>

    <!-- Color mode -->
    <div class="tooltip tooltip-bottom" data-tip="Color: {COLOR_LABELS[colorMode]}">
      <Popover
        bind:open={colorOpen}
        triggerClass={`mw-gtb-icon-trigger${colorMode !== "default" ? " is-active" : ""}`}
        panelClass="mw-gtb-panel"
        triggerAriaLabel="Color: {COLOR_LABELS[colorMode]}"
      >
        {#snippet trigger()}
          <Icon name="palette" size={14} />
        {/snippet}
        {#snippet children()}
          {#each COLOR_OPTIONS as opt (opt.id)}
            <label class="mw-gtb-row">
              <input
                type="radio"
                name="gtb-color"
                class="radio radio-xs"
                checked={colorMode === opt.id}
                onchange={() => (colorMode = opt.id)}
              />
              <span>{opt.label}</span>
            </label>
          {/each}
        {/snippet}
      </Popover>
    </div>

    <!-- Folders dropdown -->
    <div class="tooltip tooltip-bottom" data-tip="Filter by folder">
      <Popover
        bind:open={foldersOpen}
        triggerClass="mw-gtb-icon-trigger"
        panelClass="mw-gtb-panel mw-gtb-panel-scroll"
        triggerAriaLabel="Filter by folder"
      >
        {#snippet trigger()}
          <Icon name="folder" size={14} />
          {#if folderFilter.length > 0}
            <span class="mw-gtb-count-badge">{folderFilter.length}</span>
          {/if}
        {/snippet}
        {#snippet children()}
          {#if folderOptions.length === 0}
            <div class="mw-gtb-empty">No folders</div>
          {:else}
            {#each folderOptions as f (f)}
              <label class="mw-gtb-row">
                <input
                  type="checkbox"
                  class="checkbox checkbox-xs"
                  checked={folderFilter.includes(f)}
                  onchange={() => toggleFolder(f)}
                />
                <span class="truncate">{f}</span>
              </label>
            {/each}
          {/if}
        {/snippet}
      </Popover>
    </div>

    <!-- Tags dropdown -->
    <div class="tooltip tooltip-bottom" data-tip="Filter by tag">
      <Popover
        bind:open={tagsOpen}
        triggerClass="mw-gtb-icon-trigger"
        panelClass="mw-gtb-panel mw-gtb-panel-scroll"
        triggerAriaLabel="Filter by tag"
      >
        {#snippet trigger()}
          <Icon name="tag" size={14} />
          {#if tagFilter.length > 0}
            <span class="mw-gtb-count-badge">{tagFilter.length}</span>
          {/if}
        {/snippet}
        {#snippet children()}
          {#if tagOptions.length === 0}
            <div class="mw-gtb-empty">No tags</div>
          {:else}
            {#each tagOptions as t (t)}
              <label class="mw-gtb-row">
                <input
                  type="checkbox"
                  class="checkbox checkbox-xs"
                  checked={tagFilter.includes(t)}
                  onchange={() => toggleTag(t)}
                />
                <span class="truncate font-mono">#{t}</span>
              </label>
            {/each}
          {/if}
        {/snippet}
      </Popover>
    </div>
  </div>

  <!-- Group 3: Search -->
  <div class="mw-gtb-search-wrap">
    <Icon name="search" size={12} class="mw-gtb-search-icon" />
    <input
      type="text"
      class="mw-gtb-search"
      placeholder="Search nodes…"
      bind:value={searchFilter}
    />
  </div>

  <div class="mw-gtb-div"></div>

  <!-- Group 4: Actions -->
  <div class="mw-gtb-group">
    <IconButton
      icon="rotate-ccw"
      tooltip="Clear filters"
      size="sm"
      disabled={!hasActiveFilters}
      onclick={onReset}
    />
    <IconButton
      icon="move-right"
      tooltip={showEdgeArrows ? "Hide edge arrows" : "Show edge arrows"}
      size="sm"
      active={showEdgeArrows}
      onclick={() => (showEdgeArrows = !showEdgeArrows)}
    />
    <IconButton
      icon="maximize-2"
      tooltip="Fit to view"
      size="sm"
      onclick={onFit}
    />

    <!-- Appearance popover -->
    <div class="tooltip tooltip-bottom" data-tip="Appearance">
      <Popover
        bind:open={settingsOpen}
        align="end"
        triggerClass="mw-gtb-icon-trigger"
        panelClass="mw-gtb-panel mw-gtb-settings-panel"
        triggerAriaLabel="Appearance"
      >
        {#snippet trigger()}
          <Icon name="sliders-horizontal" size={14} />
        {/snippet}
        {#snippet children()}
          <div class="mw-gtb-setting">
            <div class="mw-gtb-setting-label">Node size</div>
            <ToggleButtonGroup
              options={NODE_SIZE_OPTIONS}
              bind:value={nodeSize}
              size="sm"
            />
          </div>

          <div class="mw-gtb-setting">
            <div class="mw-gtb-setting-label">Label size</div>
            <ToggleButtonGroup
              options={LABEL_SIZE_OPTIONS}
              bind:value={labelSize}
              size="sm"
            />
          </div>

          <label class="mw-gtb-row">
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              checked={labelMode === "hover"}
              onchange={() =>
                (labelMode = labelMode === "always" ? "hover" : "always")}
            />
            <span>Labels on hover only</span>
          </label>

          <div class="mw-gtb-setting">
            <div class="mw-gtb-setting-label">Edge width</div>
            <ToggleButtonGroup
              options={EDGE_WIDTH_OPTIONS}
              bind:value={edgeWidth}
              size="sm"
            />
          </div>

          <div class="mw-gtb-settings-divider"></div>

          <button
            type="button"
            class="mw-gtb-danger"
            onclick={onResetLayout}
            title="Clear saved node positions and re-run layout"
          >
            Reset layout
          </button>
        {/snippet}
      </Popover>
    </div>
  </div>
</div>

<style>
  /* Two-state toolbar: passive (focused on graph) vs active (focused on toolbar).
     Passive sinks the bar into the background so the graph content stays the
     visual focus. Active restores the full chrome — solid surface, stronger
     border, soft shadow — so controls are easy to read while the user is
     actually using them.

     `.mw-gtb:has(.popover-panel)` is the important second selector: without it
     the bar would snap back to passive the moment the user moved the mouse
     from the bar down into a just-opened popover panel (the panel sits
     absolutely positioned below the bar, so leaving the bar's bounding box
     while the popover is still open would otherwise fade everything out). */
  .mw-gtb {
    position: absolute;
    top: 8px;
    left: 8px;
    right: 8px;
    z-index: 10;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 3px 6px;
    opacity: 0.45;
    background: color-mix(in oklch, var(--mw-bg-elev) 40%, transparent);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    backdrop-filter: blur(8px);
    font-size: 11px;
    color: var(--mw-ink-1);
    transition:
      opacity 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }
  .mw-gtb:hover,
  .mw-gtb:has(:global(.popover-panel)) {
    opacity: 1;
    background: color-mix(in oklch, var(--mw-bg-elev) 88%, transparent);
    border-color: var(--mw-rule-strong);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  }

  .mw-gtb-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .mw-gtb-div {
    width: 1px;
    height: 14px;
    background: var(--mw-rule-strong);
    flex-shrink: 0;
  }

  /* Icon popover trigger — used by Color / Folders / Tags / Appearance.
     Marked :global because the trigger element lives inside Popover.svelte's
     template and won't inherit this component's scope hash via the
     triggerClass prop (it's just a string, not parsed by Svelte's CSS scoper).
     `position: relative` is the anchor for the count-badge. */
  :global(.mw-gtb-icon-trigger) {
    position: relative;
    display: grid;
    place-items: center;
    width: 28px;
    height: 24px;
    border-radius: var(--mw-radius-xs);
    color: var(--mw-ink-2);
    cursor: pointer;
    transition: color 0.1s, background 0.1s;
    user-select: none;
  }
  :global(.mw-gtb-icon-trigger:hover) {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  /* Active state — used by Color trigger when colorMode !== "default" to
     signal "you've changed the default coloring" without text. */
  :global(.mw-gtb-icon-trigger.is-active) {
    color: var(--mw-accent);
  }

  /* Two-line row variant for popovers that benefit from a description under
     the main label (Scope's All / ±1 / ±2 entries). The radio sits aligned
     with the title, description wraps below. Marked :global to match the
     `.mw-gtb-row` precedent — Svelte's CSS scoper can't statically see
     classes referenced inside snippets passed to another component. */
  :global(.mw-gtb-row-stack) {
    align-items: flex-start;
    padding: 6px;
  }
  :global(.mw-gtb-row-text) {
    display: flex;
    flex-direction: column;
    gap: 1px;
    line-height: 1.3;
  }
  :global(.mw-gtb-row-title) {
    font-size: 11px;
    color: var(--mw-ink-1);
    font-weight: 500;
  }
  :global(.mw-gtb-row-desc) {
    font-size: 10px;
    color: var(--mw-ink-3);
  }

  /* Floating count on top-right of an icon trigger (Folders / Tags).
     Lives inside the trigger snippet, so stays in this component's scope. */
  .mw-gtb-count-badge {
    position: absolute;
    top: 1px;
    right: 1px;
    min-width: 12px;
    height: 12px;
    padding: 0 3px;
    background: var(--mw-accent);
    color: var(--mw-accent-content, #1a1613);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    pointer-events: none;
  }

  /* Dropdown panels */
  :global(.mw-gtb-panel) {
    margin-top: 4px !important;
    background: var(--color-base-100) !important;
    border: 1px solid var(--mw-rule-strong) !important;
    border-radius: var(--mw-radius-sm) !important;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.08) !important;
    padding: 4px !important;
    min-width: 180px !important;
    z-index: 20 !important;
  }
  :global(.mw-gtb-panel-scroll) {
    max-height: 240px;
    overflow-y: auto;
  }
  :global(.mw-gtb-settings-panel) {
    min-width: 260px !important;
    padding: 10px !important;
    display: flex !important;
    flex-direction: column !important;
    gap: 10px !important;
  }

  /* Panel row (checkbox / radio + label) */
  :global(.mw-gtb-row) {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    font-size: 11px;
    color: var(--mw-ink-1);
    border-radius: var(--mw-radius-xs);
    cursor: pointer;
  }
  :global(.mw-gtb-row:hover) {
    background: var(--color-base-300);
  }
  :global(.mw-gtb-empty) {
    padding: 8px 10px;
    font-size: 11px;
    color: var(--mw-ink-3);
    font-style: italic;
  }

  /* Settings sub-blocks */
  :global(.mw-gtb-setting) {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  :global(.mw-gtb-setting-label) {
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mw-ink-3);
  }
  :global(.mw-gtb-settings-divider) {
    height: 1px;
    background: var(--mw-rule);
    margin: 2px 0;
  }
  :global(.mw-gtb-danger) {
    padding: 6px 8px;
    font-size: 11px;
    color: var(--color-error);
    background: transparent;
    border: 1px solid var(--mw-rule-strong);
    border-radius: var(--mw-radius-xs);
    cursor: pointer;
    text-align: center;
    transition: background 0.1s, color 0.1s;
  }
  :global(.mw-gtb-danger:hover) {
    background: color-mix(in oklch, var(--color-error) 10%, transparent);
  }

  /* Search input */
  .mw-gtb-search-wrap {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 8px;
    background: color-mix(in oklch, var(--mw-rule) 60%, transparent);
    border: 1px solid var(--mw-rule-strong);
    border-radius: var(--mw-radius-xs);
    color: var(--mw-ink-3);
    transition: border-color 0.1s;
    flex: 0 1 180px;
    min-width: 120px;
  }
  .mw-gtb-search-wrap:focus-within {
    border-color: var(--mw-accent);
    color: var(--mw-ink-2);
  }
  :global(.mw-gtb-search-icon) {
    flex-shrink: 0;
  }
  .mw-gtb-search {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: none;
    outline: none;
    font-size: 11px;
    color: var(--mw-ink-1);
  }
  .mw-gtb-search::placeholder {
    color: var(--mw-ink-3);
  }
</style>
