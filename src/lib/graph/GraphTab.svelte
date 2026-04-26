<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinks } from "$lib/workspace/backlinkIndex.svelte";
  import { tags, tagList } from "$lib/workspace/tagIndex.svelte";
  import type { Tab } from "$lib/workspace/types";
  import GraphToolbar from "./GraphToolbar.svelte";
  import HoverPreview from "./HoverPreview.svelte";
  import {
    loadDisplayPrefs,
    saveDisplayPrefs,
    loadSavedFilters,
    saveFilters,
    type NodeSizePreset,
    type LabelSizePreset,
    type LabelMode,
    type ViewMode,
    type ColorMode,
  } from "./prefs";
  import { buildGraphData } from "./data";
  import { styleFromTheme } from "./style";
  import { ensureCytoscapeExtensions } from "./layoutConfig";
  import {
    scheduleHoverPreview,
    clearHoverPreview,
  } from "./hoverPreviewState.svelte";
  import {
    annotateDegree,
    annotateColors,
    runInitialLayout,
    runColaLayout,
    reconcileGraph as ctrlReconcileGraph,
    hardRebuild as ctrlHardRebuild,
    applyFilters as ctrlApplyFilters,
    refitToVisible,
  } from "./graphController";
  import {
    loadLayout,
    clearSavedLayout,
    createLayoutSaveScheduler,
    type LayoutSaveScheduler,
  } from "./layoutPersistence";

  let { tab }: { tab: Tab } = $props();

  let host: HTMLDivElement;
  // IMPORTANT: cytoscape Core handle MUST live in a plain `let` (not $state).
  // Svelte 5's proxy will deep-observe internal handles and corrupt them.
  // Same rule as Milkdown Editor / ProseMirror EditorView / CodeMirror EditorView.
  let cy: cytoscape.Core | null = null;
  // The currently-running cola layout (infinite mode). Held so we can stop
  // it when the tab leaves view — also a plain `let`, not $state.
  let currentLayout: cytoscape.Layouts | null = null;
  let themeObserver: MutationObserver | null = null;
  let visibilityObserver: IntersectionObserver | null = null;
  let lastFileIndexLength = 0;
  let lastBacklinksBuilt = 0;

  // Persisted filter state loaded from localStorage.
  const initialFilters = loadSavedFilters();
  let viewMode: ViewMode = $state(initialFilters.viewMode);
  let colorMode: ColorMode = $state(initialFilters.colorMode);

  // Display preferences (persisted to localStorage, exposed via GraphToolbar)
  const initialDisplay = loadDisplayPrefs();
  let nodeSize = $state<NodeSizePreset>(initialDisplay.nodeSize);
  let labelSize = $state<LabelSizePreset>(initialDisplay.labelSize);
  let labelMode = $state<LabelMode>(initialDisplay.labelMode);
  let showEdgeArrows = $state(initialDisplay.showEdgeArrows);
  let edgeWidth = $state(initialDisplay.edgeWidth);

  // Phase 6 filters (folder/tag persisted; search intentionally not)
  let folderFilter: string[] = $state(initialFilters.folderFilter);
  let tagFilter: string[] = $state(initialFilters.tagFilter);
  let searchFilter = $state("");

  const folderOptions = $derived.by(() => {
    void workspace.fileIndex;
    const root = workspace.info?.root ?? "";
    const set = new Set<string>();
    for (const f of workspace.fileIndex) {
      if (f.kind !== "markdown") continue;
      const rel = f.path.startsWith(root)
        ? f.path.slice(root.length).replace(/^[/\\]/, "")
        : f.path;
      const segs = rel.split(/[/\\]/);
      set.add(segs.length > 1 ? segs[0] : "(root)");
    }
    return Array.from(set).sort((a, b) => a.localeCompare(b));
  });

  const tagOptions = $derived.by(() => {
    void tags.lastBuilt;
    return tagList().map((t) => t.tag);
  });

  // Path of the currently-active markdown tab (or null if active tab
  // isn't a markdown file). Drives the `.active` node highlight.
  const activeNotePath = $derived.by(() => {
    const pane = workspace.activePane;
    const t = pane.tabs.find((x) => x.id === pane.activeTabId);
    return t?.kind === "markdown" ? t.path : null;
  });

  // Bundle current display state into the shape styleFromTheme expects.
  // Used at the 3 call sites (initial mount, theme observer, display-prefs
  // effect) to avoid repeating the property list.
  function currentStyleInputs() {
    return { nodeSize, labelSize, labelMode, showEdgeArrows, edgeWidth };
  }

  // ─── Layout lifecycle ──────────────────────────────────────────────────
  // currentLayout is GraphTab-owned mutable state (NOT $state — cytoscape
  // Layouts handle would be corrupted by Svelte's proxy). The controller
  // exposes pure makeColaLayout/runColaLayout; this component manages the
  // start/stop bookkeeping so the cy ref never leaves a plain `let`.

  function startLayout(): void {
    if (!cy) return;
    currentLayout?.stop();
    currentLayout = runColaLayout(cy);
  }

  function stopLayout(): void {
    currentLayout?.stop();
    currentLayout = null;
  }

  // ─── Wrappers around the controller ────────────────────────────────────
  // Adapt the stateless controller functions to component closure
  // (cy + reactive state + currentLayout lifecycle).

  function reconcileGraph(): void {
    if (!cy) return;
    const hasNewNodes = ctrlReconcileGraph(cy, colorMode);
    // Only restart cola if genuinely new nodes appeared — existing positions
    // stay put so a rename or new link doesn't scramble the layout.
    if (hasNewNodes) startLayout();
  }

  function hardRebuild(): void {
    if (!cy) return;
    stopLayout();
    ctrlHardRebuild(cy, colorMode);
    startLayout();
  }

  function applyFilters(): void {
    if (!cy) return;
    ctrlApplyFilters(cy, {
      viewMode,
      folderFilter,
      tagFilter,
      searchFilter,
      workspaceRoot: workspace.info?.root ?? "",
      activeMarkdownPath: activeNotePath,
    });
  }

  // ─── Toolbar action handlers ───────────────────────────────────────────

  function resetFilters(): void {
    folderFilter = [];
    tagFilter = [];
    searchFilter = "";
  }

  function fitGraph(): void {
    cy?.fit(undefined, 60);
  }

  function centerViewport(): void {
    if (!cy) return;
    cy.resize();
    cy.fit(undefined, 60);
  }

  // Layout save scheduler — created once cy exists in initGraph.
  let layoutSaveScheduler: LayoutSaveScheduler | null = null;

  async function resetLayoutToDefault(): Promise<void> {
    await clearSavedLayout();
    if (cy) {
      cy.nodes().unlock();
      hardRebuild();
    }
  }

  onMount(() => {
    ensureCytoscapeExtensions();
    void initGraph();
  });

  // High-level orchestration. Each step is a small named helper below so
  // the flow reads top-to-bottom without scrolling through inlined detail.
  async function initGraph(): Promise<void> {
    const saved = await loadLayout();
    cy = createGraph(host);
    annotateDegree(cy);
    annotateColors(cy, colorMode);
    layoutSaveScheduler = createLayoutSaveScheduler(cy);
    seedTrackers();
    applySavedOrInitialPositions(saved);
    scheduleInitialFit();
    registerCyHandlers(cy);
    themeObserver = createThemeObserver();
    visibilityObserver = createVisibilityObserver(host);
  }

  // ─── Cytoscape event handlers (component scope) ────────────────────────
  // Defined here (not inline in initGraph) so they're easy to find and
  // reason about. They close over `cy` + reactive state + scheduler.

  function onNodeTap(e: cytoscape.EventObject): void {
    const path = e.target.id() as string;
    const evt = e.originalEvent as MouseEvent | undefined;
    // Clear any pending hover-preview timer so it doesn't flash after click.
    clearHoverPreview();

    // NOTE: previously animated the camera to center the clicked node.
    // Removed because it disrupted user-controlled panning — the user
    // would carefully pan to inspect a region, click any node, and the
    // viewport would slide away. Click-to-open should NOT move the
    // camera. If "locate active note in graph" is wanted later, expose
    // it as an explicit toolbar action instead of a click side-effect.

    // Click mapping aligned with FileTreeNode:
    //   tap             → openFile (current pane, focuses existing tab)
    //   cmd/ctrl+tap    → openInOtherPane (splits if only one pane)
    //   shift+tap       → replaceCurrentTab (reuse current tab id)
    if (evt?.shiftKey) {
      workspace.replaceCurrentTab(path);
    } else if (evt && (evt.metaKey || evt.ctrlKey)) {
      workspace.openInOtherPane(path);
    } else {
      workspace.openFile(path);
    }
  }

  function onNodeMouseOver(e: cytoscape.EventObject): void {
    if (!cy) return;
    // closedNeighborhood = node + its edges + the other endpoints of those edges.
    // That's the ego network.
    const focused = e.target.closedNeighborhood();
    cy.elements().addClass("dimmed");
    focused.removeClass("dimmed").addClass("focused");

    // Hover preview shows after a 1s delay (managed by the module).
    const path = e.target.id() as string;
    const evt = e.originalEvent as MouseEvent | undefined;
    scheduleHoverPreview(path, evt);
  }

  function onNodeMouseOut(): void {
    cy?.elements().removeClass("dimmed").removeClass("focused");
    clearHoverPreview();
  }

  function onNodeDragFree(): void {
    layoutSaveScheduler?.schedule();
  }

  // ─── initGraph helpers ─────────────────────────────────────────────────

  function createGraph(host: HTMLDivElement): cytoscape.Core {
    const { nodes, edges } = buildGraphData();
    return cytoscape({
      container: host,
      elements: [...nodes, ...edges],
      style: styleFromTheme(currentStyleInputs()),
      // Defer layout — we apply saved positions or run fcose pre-pass below.
      layout: { name: "preset" } as cytoscape.LayoutOptions,
      wheelSensitivity: 0.2,
      minZoom: 0.2,
      maxZoom: 3,
    });
  }

  // Seed the change-detection trackers so the corresponding $effect blocks
  // don't fire spuriously on the first tick after mount.
  function seedTrackers(): void {
    lastRoot = workspace.info?.root ?? null;
    // Key shape MUST match the lastRefitKey effect below.
    lastRefitKey = [
      viewMode,
      folderFilter.slice().sort().join("|"),
      tagFilter.slice().sort().join("|"),
    ].join("::");
    lastFileIndexLength = workspace.fileIndex.length;
    lastBacklinksBuilt = backlinks.lastBuilt;
  }

  // If saved positions exist, apply them and lock so the layout doesn't
  // move them; new (unsaved) nodes go through fcose to find a place. After
  // a short delay we unlock everything so live cola force can take over.
  // No saved positions → fcose for everyone, then cola.
  function applySavedOrInitialPositions(
    saved: Map<string, { x: number; y: number }>,
  ): void {
    if (!cy) return;
    if (saved.size > 0) {
      let lockedCount = 0;
      cy.nodes().forEach((n) => {
        const pos = saved.get(n.id());
        if (pos) {
          n.position(pos);
          n.lock();
          lockedCount++;
        }
      });
      const hasNewNodes = lockedCount < cy.nodes().length;
      if (hasNewNodes) runInitialLayout(cy);
      setTimeout(() => {
        cy?.nodes().unlock();
        startLayout();
      }, 500);
    } else {
      runInitialLayout(cy);
      startLayout();
    }
  }

  // Two rAFs is the belt-and-braces pattern — the flex container needs a
  // tick to settle its size, and cy.resize() (inside centerViewport) forces
  // cytoscape to re-read container bounds so cy.fit lands centered.
  function scheduleInitialFit(): void {
    requestAnimationFrame(() => {
      requestAnimationFrame(centerViewport);
    });
  }

  function registerCyHandlers(cy: cytoscape.Core): void {
    cy.on("tap", "node", onNodeTap);
    cy.on("mouseover", "node", onNodeMouseOver);
    cy.on("mouseout", "node", onNodeMouseOut);
    cy.on("dragfreeon", "node", onNodeDragFree);
  }

  // Re-apply stylesheet on theme switch. Default colorMode reads
  // --color-primary which changes per theme; re-annotate so node colors
  // match the new palette.
  function createThemeObserver(): MutationObserver {
    const observer = new MutationObserver(() => {
      if (!cy) return;
      cy.style(styleFromTheme(currentStyleInputs()));
      if (colorMode === "default") {
        annotateColors(cy, colorMode);
        cy.style().update();
      }
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return observer;
  }

  // Pause continuous force simulation when the tab is hidden (display:none).
  // Resumes on show. The activeTabId effect below is a belt-and-braces
  // backup for cases IO doesn't fire on (e.g. mount-while-hidden).
  function createVisibilityObserver(
    host: HTMLDivElement,
  ): IntersectionObserver {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!cy) return;
        if (entry.isIntersecting) {
          if (!currentLayout) startLayout();
        } else {
          stopLayout();
        }
      },
      { threshold: 0 },
    );
    observer.observe(host);
    return observer;
  }

  onDestroy(() => {
    void layoutSaveScheduler?.flush();
    layoutSaveScheduler = null;
    clearHoverPreview();
    visibilityObserver?.disconnect();
    visibilityObserver = null;
    themeObserver?.disconnect();
    themeObserver = null;
    stopLayout();
    cy?.destroy();
    cy = null;
  });

  // Reconcile graph when fileIndex changes or backlinks index gets rebuilt.
  // Uses the incremental path — existing node positions are preserved.
  // Full rebuild only happens on workspace switch (handled by the lastRoot
  // effect below).
  $effect(() => {
    const len = workspace.fileIndex.length;
    const built = backlinks.lastBuilt;
    if (!cy) return;
    if (len !== lastFileIndexLength || built !== lastBacklinksBuilt) {
      lastFileIndexLength = len;
      lastBacklinksBuilt = built;
      reconcileGraph();
    }
  });

  // Hard rebuild on workspace switch. Initial mount is handled by initGraph(),
  // which seeds lastRoot; subsequent root changes trigger a full re-layout.
  let lastRoot: string | null = null;
  $effect(() => {
    const root = workspace.info?.root ?? null;
    if (!cy) return;
    if (root !== lastRoot) {
      lastRoot = root;
      hardRebuild();
    }
  });

  // Belt-and-braces: in addition to IntersectionObserver, watch the owning
  // pane's activeTabId. If this tab isn't active, pause the cola simulation.
  // IO covers display:none correctly per spec, but the explicit check is
  // immediate and handles edge cases where IO doesn't fire (e.g. initial
  // mount while the tab is already hidden).
  $effect(() => {
    const pane = workspace.panes.find((p) =>
      p.tabs.some((t) => t.id === tab.id),
    );
    const isActive = pane?.activeTabId === tab.id;
    if (!cy) return;
    if (isActive) {
      if (!currentLayout) startLayout();
    } else {
      stopLayout();
    }
  });

  // Re-apply combined filters when any filter input or active tab changes
  // OR after a graph rebuild (which clears all classes).
  $effect(() => {
    void viewMode;
    void folderFilter;
    void tagFilter;
    void searchFilter;
    void workspace.activePane.activeTabId;
    void backlinks.lastBuilt;
    if (cy) applyFilters();
  });

  // Refit viewport only on deliberate user-driven scope changes — view mode
  // or folder/tag filter toggles. Search keystrokes are intentionally
  // excluded so typing doesn't jitter the viewport.
  //
  // NOTE: activeTabId used to be in this key (in local mode only) so the
  // camera would follow the active note when its neighborhood changed.
  // Removed because clicking any graph node calls openFile → activeTabId
  // changes → camera refits → user perceives "graph keeps jumping". Stable
  // camera wins over auto-follow; the Fit button (⤢) is always one click
  // away if the user wants to recenter manually.
  let lastRefitKey = "";
  $effect(() => {
    const key = [
      viewMode,
      folderFilter.slice().sort().join("|"),
      tagFilter.slice().sort().join("|"),
    ].join("::");
    if (!cy) return;
    if (key === lastRefitKey) return;
    lastRefitKey = key;
    refitToVisible(cy);
  });

  // Re-annotate node colors when colorMode changes.
  $effect(() => {
    void colorMode;
    if (!cy) return;
    annotateColors(cy, colorMode);
    cy.style().update();
  });

  // Keep the .active class in sync with the currently-open markdown note.
  // Runs whenever activeNotePath changes OR after a rebuild (backlinks
  // rebuild signal via lastBuilt covers reconcile too).
  $effect(() => {
    const p = activeNotePath;
    void backlinks.lastBuilt;
    if (!cy) return;
    cy.nodes(".active").removeClass("active");
    if (p) {
      const n = cy.getElementById(p);
      if (!n.empty()) n.addClass("active");
    }
  });

  // Persist display preferences whenever any of them changes.
  $effect(() => {
    saveDisplayPrefs({
      nodeSize,
      labelSize,
      labelMode,
      showEdgeArrows,
      edgeWidth,
    });
  });

  // Persist filter state (view / color / folder / tag) across sessions.
  // searchFilter is intentionally excluded.
  $effect(() => {
    saveFilters({
      viewMode,
      colorMode,
      folderFilter: [...folderFilter],
      tagFilter: [...tagFilter],
    });
  });

  // Hot-apply stylesheet when any visual display pref changes.
  $effect(() => {
    void nodeSize;
    void labelSize;
    void labelMode;
    void showEdgeArrows;
    void edgeWidth;
    if (!cy) return;
    cy.style(styleFromTheme(currentStyleInputs()));
  });
</script>

<div class="relative w-full h-full">
  <div bind:this={host} class="w-full h-full bg-base-200"></div>

  <GraphToolbar
    bind:viewMode
    bind:colorMode
    bind:folderFilter
    bind:tagFilter
    bind:searchFilter
    bind:nodeSize
    bind:labelSize
    bind:labelMode
    bind:showEdgeArrows
    bind:edgeWidth
    {folderOptions}
    {tagOptions}
    onReset={resetFilters}
    onFit={fitGraph}
    onResetLayout={resetLayoutToDefault}
  />
</div>

<HoverPreview />
