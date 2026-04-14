<script lang="ts">
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
    showMinimap = $bindable<boolean>(),
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
    showMinimap: boolean;
    folderOptions: string[];
    tagOptions: string[];
    onReset: () => void;
    onFit: () => void;
    onResetLayout: () => void;
  } = $props();

  const VIEW_OPTIONS: { id: ViewMode; label: string }[] = [
    { id: "all", label: "All" },
    { id: "local-1", label: "±1" },
    { id: "local-2", label: "±2" },
  ];

  const NODE_SIZE_OPTIONS: { id: NodeSizePreset; label: string }[] = [
    { id: "xs", label: "XS" },
    { id: "sm", label: "S" },
    { id: "md", label: "M" },
    { id: "lg", label: "L" },
    { id: "xl", label: "XL" },
  ];

  const LABEL_SIZE_OPTIONS: { id: LabelSizePreset; label: string }[] = [
    { id: "hidden", label: "—" },
    { id: "xs", label: "XS" },
    { id: "sm", label: "S" },
    { id: "md", label: "M" },
    { id: "lg", label: "L" },
  ];

  const EDGE_WIDTH_OPTIONS: number[] = [1, 2, 3];

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

<div
  class="absolute top-2 left-2 right-2 z-10 flex flex-wrap items-center gap-1 bg-base-100/90 border border-base-300 rounded-md px-2 py-1 backdrop-blur shadow-sm"
>
  <!-- View mode segmented control -->
  <div class="flex items-center text-[11px]">
    {#each VIEW_OPTIONS as opt (opt.id)}
      <button
        type="button"
        class="px-2 py-0.5 rounded transition-colors"
        class:bg-primary={viewMode === opt.id}
        class:text-primary-content={viewMode === opt.id}
        class:hover:bg-base-200={viewMode !== opt.id}
        onclick={() => (viewMode = opt.id)}
      >
        {opt.label}
      </button>
    {/each}
  </div>

  <div class="w-px h-4 bg-base-300 mx-1"></div>

  <!-- Color mode -->
  <select
    class="text-[11px] bg-transparent border-0 outline-none focus:outline-none cursor-pointer pr-1"
    bind:value={colorMode}
    title="Color nodes by"
  >
    <option value="default">Default</option>
    <option value="folder">By folder</option>
    <option value="tag">By tag</option>
  </select>

  <div class="w-px h-4 bg-base-300 mx-1"></div>

  <!-- Folder multi-select dropdown -->
  <!-- svelte-ignore a11y_label_has_associated_control -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="dropdown dropdown-hover">
    <label
      tabindex="0"
      class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200 cursor-pointer block"
    >
      Folders{folderFilter.length > 0 ? ` (${folderFilter.length})` : ""}
    </label>
    <div
      class="dropdown-content z-20 mt-1 max-h-60 overflow-y-auto bg-base-100 border border-base-300 rounded-md shadow-lg p-1 min-w-[180px]"
    >
      {#if folderOptions.length === 0}
        <div class="text-[11px] text-base-content/50 italic px-2 py-1">
          No folders
        </div>
      {:else}
        {#each folderOptions as f (f)}
          <label
            class="flex items-center gap-2 text-[11px] px-2 py-1 hover:bg-base-200 rounded cursor-pointer"
          >
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

  <!-- Tag multi-select dropdown -->
  <!-- svelte-ignore a11y_label_has_associated_control -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="dropdown dropdown-hover">
    <label
      tabindex="0"
      class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200 cursor-pointer block"
    >
      Tags{tagFilter.length > 0 ? ` (${tagFilter.length})` : ""}
    </label>
    <div
      class="dropdown-content z-20 mt-1 max-h-60 overflow-y-auto bg-base-100 border border-base-300 rounded-md shadow-lg p-1 min-w-[180px]"
    >
      {#if tagOptions.length === 0}
        <div class="text-[11px] text-base-content/50 italic px-2 py-1">
          No tags
        </div>
      {:else}
        {#each tagOptions as t (t)}
          <label
            class="flex items-center gap-2 text-[11px] px-2 py-1 hover:bg-base-200 rounded cursor-pointer"
          >
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

  <div class="w-px h-4 bg-base-300 mx-1"></div>

  <!-- Search input -->
  <input
    type="text"
    class="text-[11px] bg-base-200/50 border border-base-300 rounded px-2 py-0.5 outline-none focus:border-primary w-32"
    placeholder="Search nodes…"
    bind:value={searchFilter}
  />

  <div class="w-px h-4 bg-base-300 mx-1"></div>

  <!-- Display preferences dropdown -->
  <!-- svelte-ignore a11y_label_has_associated_control -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="dropdown dropdown-hover">
    <label
      tabindex="0"
      class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200 cursor-pointer block"
    >
      Display
    </label>
    <div
      class="dropdown-content z-20 mt-1 bg-base-100 border border-base-300 rounded-md shadow-lg p-3 min-w-[220px] space-y-3"
    >
      <div>
        <div class="text-[10px] text-base-content/50 mb-1">Node size</div>
        <div class="flex gap-0.5 text-[10px]">
          {#each NODE_SIZE_OPTIONS as opt (opt.id)}
            <button
              type="button"
              class="px-2 py-0.5 rounded flex-1 transition-colors"
              class:bg-primary={nodeSize === opt.id}
              class:text-primary-content={nodeSize === opt.id}
              class:hover:bg-base-200={nodeSize !== opt.id}
              onclick={() => (nodeSize = opt.id)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
      </div>

      <div>
        <div class="text-[10px] text-base-content/50 mb-1">Label size</div>
        <div class="flex gap-0.5 text-[10px]">
          {#each LABEL_SIZE_OPTIONS as opt (opt.id)}
            <button
              type="button"
              class="px-2 py-0.5 rounded flex-1 transition-colors"
              class:bg-primary={labelSize === opt.id}
              class:text-primary-content={labelSize === opt.id}
              class:hover:bg-base-200={labelSize !== opt.id}
              onclick={() => (labelSize = opt.id)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
      </div>

      <label
        class="flex items-center gap-2 text-[11px] cursor-pointer select-none"
      >
        <input
          type="checkbox"
          class="checkbox checkbox-xs"
          checked={labelMode === "hover"}
          onchange={() =>
            (labelMode = labelMode === "always" ? "hover" : "always")}
        />
        <span>Labels on hover only</span>
      </label>

      <label
        class="flex items-center gap-2 text-[11px] cursor-pointer select-none"
      >
        <input
          type="checkbox"
          class="checkbox checkbox-xs"
          bind:checked={showEdgeArrows}
        />
        <span>Show edge arrows</span>
      </label>

      <div>
        <div class="text-[10px] text-base-content/50 mb-1">Edge width</div>
        <div class="flex gap-0.5 text-[10px]">
          {#each EDGE_WIDTH_OPTIONS as w (w)}
            <button
              type="button"
              class="px-2 py-0.5 rounded flex-1 transition-colors"
              class:bg-primary={edgeWidth === w}
              class:text-primary-content={edgeWidth === w}
              class:hover:bg-base-200={edgeWidth !== w}
              onclick={() => (edgeWidth = w)}
            >
              {w}px
            </button>
          {/each}
        </div>
      </div>

      <label
        class="flex items-center gap-2 text-[11px] cursor-pointer select-none"
      >
        <input
          type="checkbox"
          class="checkbox checkbox-xs"
          bind:checked={showMinimap}
        />
        <span>Show minimap</span>
      </label>
    </div>
  </div>

  <div class="w-px h-4 bg-base-300 mx-1"></div>

  <button
    type="button"
    class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200"
    onclick={onReset}
    title="Clear filters"
  >
    Reset
  </button>
  <button
    type="button"
    class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200"
    onclick={onFit}
    title="Zoom to fit"
  >
    Fit
  </button>
  <button
    type="button"
    class="text-[11px] px-2 py-0.5 rounded hover:bg-base-200"
    onclick={onResetLayout}
    title="Reset graph layout (clear saved node positions)"
  >
    Reset Layout
  </button>
</div>
