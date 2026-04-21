<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
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

  const VIEW_OPTIONS = [
    { value: "all", label: "All", tooltip: "Show full vault" },
    { value: "local-1", label: "±1", tooltip: "Direct neighbors of active note" },
    { value: "local-2", label: "±2", tooltip: "Neighbors of neighbors" },
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
</script>

<div class="mw-gtb">
  <!-- Group 1: Scope -->
  <div class="mw-gtb-group">
    <ToggleButtonGroup
      options={VIEW_OPTIONS}
      bind:value={viewMode}
      size="sm"
      tooltipPosition="bottom"
    />
  </div>

  <div class="mw-gtb-div"></div>

  <!-- Group 2: Data shaping — color / folders / tags -->
  <div class="mw-gtb-group">
    <!-- Color mode -->
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="dropdown tooltip tooltip-bottom" data-tip="Color nodes by">
      <label tabindex="0" class="mw-gtb-trigger">
        <span class="mw-gtb-trigger-label">Color</span>
        <span class="mw-gtb-trigger-value">{COLOR_LABELS[colorMode]}</span>
        <Icon name="chevron-down" size={11} />
      </label>
      <div tabindex="-1" class="dropdown-content mw-gtb-panel">
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
      </div>
    </div>

    <!-- Folders dropdown -->
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="dropdown tooltip tooltip-bottom" data-tip="Filter by folder">
      <label tabindex="0" class="mw-gtb-trigger">
        <span class="mw-gtb-trigger-label">Folders</span>
        {#if folderFilter.length > 0}
          <span class="mw-gtb-count">{folderFilter.length}</span>
        {/if}
        <Icon name="chevron-down" size={11} />
      </label>
      <div tabindex="-1" class="dropdown-content mw-gtb-panel mw-gtb-panel-scroll">
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
      </div>
    </div>

    <!-- Tags dropdown -->
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="dropdown tooltip tooltip-bottom" data-tip="Filter by tag">
      <label tabindex="0" class="mw-gtb-trigger">
        <span class="mw-gtb-trigger-label">Tags</span>
        {#if tagFilter.length > 0}
          <span class="mw-gtb-count">{tagFilter.length}</span>
        {/if}
        <Icon name="chevron-down" size={11} />
      </label>
      <div tabindex="-1" class="dropdown-content mw-gtb-panel mw-gtb-panel-scroll">
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
      </div>
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

    <!-- Settings popover -->
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="dropdown dropdown-end tooltip tooltip-bottom" data-tip="Display settings">
      <label tabindex="0" class="mw-gtb-icon-trigger">
        <Icon name="sliders-horizontal" size={14} />
      </label>
      <div
        tabindex="-1"
        class="dropdown-content mw-gtb-panel mw-gtb-settings-panel"
      >
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
      </div>
    </div>
  </div>
</div>

<style>
  .mw-gtb {
    position: absolute;
    top: 8px;
    left: 8px;
    right: 8px;
    z-index: 10;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: color-mix(in oklch, var(--mw-bg-elev) 88%, transparent);
    border: 1px solid var(--mw-rule-strong);
    border-radius: var(--mw-radius-sm);
    backdrop-filter: blur(8px);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
    font-size: 11px;
    color: var(--mw-ink-1);
  }

  .mw-gtb-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .mw-gtb-div {
    width: 1px;
    height: 18px;
    background: var(--mw-rule-strong);
    flex-shrink: 0;
  }

  /* Text/value dropdown trigger (Color / Folders / Tags) */
  .mw-gtb-trigger {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 8px;
    border-radius: var(--mw-radius-xs);
    color: var(--mw-ink-2);
    cursor: pointer;
    transition: color 0.1s, background 0.1s;
    user-select: none;
  }
  .mw-gtb-trigger:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  .mw-gtb-trigger-label {
    font-size: 11px;
  }
  .mw-gtb-trigger-value {
    font-size: 11px;
    color: var(--mw-ink-1);
    font-weight: 500;
  }
  .mw-gtb-count {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 0 4px;
    min-width: 14px;
    height: 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--mw-accent);
    color: var(--mw-accent-content, #1a1613);
    border-radius: 7px;
  }

  /* Icon-only dropdown trigger (Settings gear) */
  .mw-gtb-icon-trigger {
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
  .mw-gtb-icon-trigger:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
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
