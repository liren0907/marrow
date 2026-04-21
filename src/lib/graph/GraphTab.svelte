<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import fcose from "cytoscape-fcose";
  import cola from "cytoscape-cola";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinks } from "$lib/workspace/backlinkIndex.svelte";
  import { tags, tagList, tagsForFile } from "$lib/workspace/tagIndex.svelte";
  import {
    readTextFile,
    loadGraphLayout,
    saveGraphLayout,
  } from "$lib/workspace/tauri";
  import type { Tab } from "$lib/workspace/types";
  import GraphToolbar from "./GraphToolbar.svelte";

  type NodeSizePreset = "xs" | "sm" | "md" | "lg" | "xl";
  type LabelSizePreset = "hidden" | "xs" | "sm" | "md" | "lg";
  type LabelMode = "always" | "hover";

  interface DisplayPrefs {
    nodeSize: NodeSizePreset;
    labelSize: LabelSizePreset;
    labelMode: LabelMode;
    showEdgeArrows: boolean;
    edgeWidth: number;
  }

  const DEFAULT_PREFS: DisplayPrefs = {
    nodeSize: "sm",
    labelSize: "sm",
    labelMode: "always",
    showEdgeArrows: false,
    edgeWidth: 1,
  };

  const NODE_SIZE_MAP: Record<NodeSizePreset, [number, number]> = {
    xs: [2, 6],
    sm: [4, 12],
    md: [6, 18],
    lg: [8, 24],
    xl: [10, 30],
  };

  const LABEL_SIZE_MAP: Record<LabelSizePreset, number> = {
    hidden: 0,
    xs: 8,
    sm: 9,
    md: 10,
    lg: 12,
  };

  const DISPLAY_STORAGE_KEY = "marrow.graph.display";

  function loadDisplayPrefs(): DisplayPrefs {
    if (typeof localStorage === "undefined") return { ...DEFAULT_PREFS };
    try {
      const raw = localStorage.getItem(DISPLAY_STORAGE_KEY);
      if (!raw) return { ...DEFAULT_PREFS };
      return { ...DEFAULT_PREFS, ...JSON.parse(raw) };
    } catch {
      return { ...DEFAULT_PREFS };
    }
  }

  function saveDisplayPrefs(p: DisplayPrefs): void {
    try {
      localStorage.setItem(DISPLAY_STORAGE_KEY, JSON.stringify(p));
    } catch {
      // storage full / disabled — silently skip
    }
  }

  // Filter state persistence — view mode / color mode / folder / tag filters
  // carry over between sessions. searchFilter is intentionally NOT persisted:
  // a stale search string across sessions is confusing.
  interface SavedFilters {
    viewMode: "all" | "local-1" | "local-2";
    colorMode: "default" | "folder" | "tag";
    folderFilter: string[];
    tagFilter: string[];
  }
  const DEFAULT_FILTERS: SavedFilters = {
    viewMode: "all",
    colorMode: "default",
    folderFilter: [],
    tagFilter: [],
  };
  const FILTERS_STORAGE_KEY = "marrow.graph.filters";

  function loadSavedFilters(): SavedFilters {
    if (typeof localStorage === "undefined") return { ...DEFAULT_FILTERS };
    try {
      const raw = localStorage.getItem(FILTERS_STORAGE_KEY);
      if (!raw) return { ...DEFAULT_FILTERS };
      const parsed = JSON.parse(raw) as Partial<SavedFilters>;
      return {
        viewMode: parsed.viewMode ?? DEFAULT_FILTERS.viewMode,
        colorMode: parsed.colorMode ?? DEFAULT_FILTERS.colorMode,
        folderFilter: Array.isArray(parsed.folderFilter) ? parsed.folderFilter : [],
        tagFilter: Array.isArray(parsed.tagFilter) ? parsed.tagFilter : [],
      };
    } catch {
      return { ...DEFAULT_FILTERS };
    }
  }

  function saveFilters(f: SavedFilters): void {
    try {
      localStorage.setItem(FILTERS_STORAGE_KEY, JSON.stringify(f));
    } catch {
      // storage full / disabled — silently skip
    }
  }

  let { tab }: { tab: Tab } = $props();

  // Register cytoscape extensions once at module load. Idempotent guard
  // ensures HMR / multiple GraphTab mounts don't double-register.
  let registered = false;
  function ensureExtensions(): void {
    if (registered) return;
    cytoscape.use(fcose);
    cytoscape.use(cola);
    registered = true;
  }

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

  // Phase 8: hover preview
  let hoverPath: string | null = $state(null);
  let hoverContent = $state("");
  let hoverPos: { x: number; y: number } = $state({ x: 0, y: 0 });
  const previewCache = new Map<string, string>();
  const PREVIEW_CACHE_MAX = 32;
  let hoverDelayTimer: ReturnType<typeof setTimeout> | null = null;

  // LRU helpers — Map preserves insertion order, so touch-to-back keeps
  // most-recent at the end and oldest at the front for eviction.
  function setPreview(path: string, body: string): void {
    if (previewCache.has(path)) previewCache.delete(path);
    else if (previewCache.size >= PREVIEW_CACHE_MAX) {
      const oldest = previewCache.keys().next().value;
      if (oldest !== undefined) previewCache.delete(oldest);
    }
    previewCache.set(path, body);
  }
  function getPreview(path: string): string | undefined {
    const v = previewCache.get(path);
    if (v !== undefined) {
      previewCache.delete(path);
      previewCache.set(path, v);
    }
    return v;
  }

  type ViewMode = "all" | "local-1" | "local-2";
  type ColorMode = "default" | "folder" | "tag";

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

  function buildGraphData(): {
    nodes: cytoscape.ElementDefinition[];
    edges: cytoscape.ElementDefinition[];
  } {
    const mdFiles = workspace.fileIndex.filter((f) => f.kind === "markdown");
    const nodes: cytoscape.ElementDefinition[] = mdFiles.map((f) => ({
      data: {
        id: f.path,
        label: f.name.replace(/\.md$/i, ""),
      },
    }));
    // Dedup edges by (source, target). Multi-link → single edge.
    // Bidirectional pairs collapse into one edge with `bi` class.
    const edgeMap = new Map<
      string,
      { source: string; target: string; bidirectional: boolean }
    >();
    for (const [, entries] of backlinks.byTarget) {
      for (const entry of entries) {
        const targetPath = workspace.resolveBasename(entry.target);
        if (!targetPath || targetPath === entry.sourcePath) continue;

        const key = `${entry.sourcePath}→${targetPath}`;
        const reverseKey = `${targetPath}→${entry.sourcePath}`;

        const reverse = edgeMap.get(reverseKey);
        if (reverse) {
          reverse.bidirectional = true;
          continue;
        }
        if (!edgeMap.has(key)) {
          edgeMap.set(key, {
            source: entry.sourcePath,
            target: targetPath,
            bidirectional: false,
          });
        }
      }
    }
    const edges: cytoscape.ElementDefinition[] = Array.from(
      edgeMap.values(),
    ).map((e, i) => ({
      data: {
        id: `e${i}`,
        source: e.source,
        target: e.target,
      },
      classes: e.bidirectional ? "bi" : "uni",
    }));
    return { nodes, edges };
  }

  function styleFromTheme(): cytoscape.StylesheetStyle[] {
    const root = getComputedStyle(document.documentElement);
    // DaisyUI v5 returns full oklch(...) strings (not unitless coords), so the
    // values flow through as-is into cytoscape stylesheet color fields.
    const bc = root.getPropertyValue("--color-base-content").trim() || "oklch(0 0 0)";
    const p = root.getPropertyValue("--color-primary").trim() || "oklch(0.6 0.2 250)";
    const b3 = root.getPropertyValue("--color-base-300").trim() || "oklch(0.85 0 0)";
    const accent = root.getPropertyValue("--mw-accent").trim() || "oklch(0.72 0.13 55)";

    const [nMin, nMax] = NODE_SIZE_MAP[nodeSize];
    const fontPx = LABEL_SIZE_MAP[labelSize];
    const labelHidden = fontPx === 0;
    const baseTextOpacity = labelHidden || labelMode === "hover" ? 0 : 1;
    const arrowShape = showEdgeArrows ? "triangle" : "none";
    void p; // keep referenced for legacy symmetry; default color still uses --color-primary via colorForNode

    return [
      {
        selector: "node",
        style: {
          // Per-node color computed by annotateColors() from colorMode.
          "background-color": "data(color)" as unknown as string,
          label: labelHidden ? "" : "data(label)",
          color: bc,
          "font-size": `${fontPx || 9}px`,
          "text-opacity": baseTextOpacity,
          "text-valign": "bottom",
          "text-halign": "center",
          "text-margin-y": 6,
          width: `mapData(degree, 0, 30, ${nMin}, ${nMax})` as unknown as number,
          height: `mapData(degree, 0, 30, ${nMin}, ${nMax})` as unknown as number,
          "border-width": 0,
        },
      },
      {
        selector: "node.highlighted",
        style: {
          "border-width": 2,
          "border-color": bc,
        },
      },
      // Active note — the markdown file currently open in the active pane.
      // Rendered as an accent ring on top of whatever fill colorMode gave it.
      // Listed AFTER .highlighted so specificity ties break in .active's favor.
      {
        selector: "node.active",
        style: {
          "border-width": 2.5,
          "border-color": accent,
          "border-opacity": 1,
        },
      },
      {
        selector: "edge",
        style: {
          width: edgeWidth,
          "line-color": `color-mix(in oklch, ${b3} 40%, transparent)`,
          "curve-style": "bezier",
          "arrow-scale": 0.5,
          opacity: 0.7,
        },
      },
      {
        selector: "edge.uni",
        style: {
          "target-arrow-shape": arrowShape,
          "target-arrow-color": `color-mix(in oklch, ${b3} 60%, transparent)`,
          "source-arrow-shape": "none",
        },
      },
      {
        selector: "edge.bi",
        style: {
          "target-arrow-shape": arrowShape,
          "target-arrow-color": `color-mix(in oklch, ${b3} 60%, transparent)`,
          "source-arrow-shape": arrowShape,
          "source-arrow-color": `color-mix(in oklch, ${b3} 60%, transparent)`,
        },
      },
      // Hover ego-network: dim everything, then bring focused subgraph back.
      {
        selector: ".dimmed",
        style: {
          opacity: 0.12,
          "text-opacity": 0.12,
          "transition-property": "opacity, text-opacity",
          "transition-duration": 180,
        },
      },
      {
        selector: ".focused",
        style: {
          opacity: 1,
          "text-opacity": 1,
          "transition-property": "opacity, text-opacity",
          "transition-duration": 180,
        },
      },
      // Local-graph mode hides nodes outside the ±N-hop neighborhood.
      // Using display:none keeps elements in the cola simulation set
      // (so toggling back is instant), but they don't render.
      {
        selector: ".hidden",
        style: {
          display: "none",
        },
      },
    ];
  }

  // Annotate each node with its connection count so styleFromTheme's
  // mapData(degree, ...) sizing can take effect. Must be called AFTER
  // edges are added.
  function annotateDegree(): void {
    cy?.nodes().forEach((n) => {
      n.data("degree", n.connectedEdges().length);
    });
  }

  // Stable string-to-hue hash. Same name → same hue across sessions.
  function hashHue(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) {
      h = (h * 31 + s.charCodeAt(i)) >>> 0;
    }
    return h % 360;
  }

  function colorForNode(path: string): string {
    if (colorMode === "default") {
      return (
        getComputedStyle(document.documentElement)
          .getPropertyValue("--color-primary")
          .trim() || "oklch(0.6 0.2 250)"
      );
    }
    if (colorMode === "folder") {
      const root = workspace.info?.root ?? "";
      const rel = path.startsWith(root)
        ? path.slice(root.length).replace(/^[/\\]/, "")
        : path;
      const segs = rel.split(/[/\\]/);
      const firstSeg = segs.length > 1 ? segs[0] : "(root)";
      return `oklch(0.7 0.18 ${hashHue(firstSeg)})`;
    }
    if (colorMode === "tag") {
      const fileTags = tagsForFile(path);
      if (fileTags.length === 0) return `oklch(0.55 0.02 250)`;
      return `oklch(0.7 0.18 ${hashHue(fileTags[0])})`;
    }
    return `oklch(0.6 0.2 250)`;
  }

  function annotateColors(): void {
    cy?.nodes().forEach((n) => {
      n.data("color", colorForNode(n.id()));
    });
  }

  // First-paint layout: fcose gives nice initial positions cheap.
  // cola then takes over with continuous physics for the live feel.
  const fcoseInitOptions = {
    name: "fcose",
    quality: "default",
    randomize: true,
    animate: false,
    nodeRepulsion: () => 4500,
    idealEdgeLength: () => 80,
    nodeSeparation: 75,
    numIter: 1500,
    tile: true,
  } as unknown as cytoscape.LayoutOptions;

  // cola in `infinite: true` mode runs continuously — drag a node and the
  // rest of the graph reacts. This is the "live force" feel.
  const colaLayoutOptions = {
    name: "cola",
    infinite: true,
    fit: false,
    animate: true,
    refresh: 1,
    maxSimulationTime: 4000,
    ungrabifyWhileSimulating: false,
    nodeSpacing: () => 16,
    edgeLength: () => 80,
    randomize: false,
    avoidOverlap: true,
    handleDisconnected: true,
  } as unknown as cytoscape.LayoutOptions;

  function runInitialLayout(): void {
    if (!cy) return;
    cy.layout(fcoseInitOptions).run();
  }

  function startLayout(): void {
    if (!cy) return;
    currentLayout?.stop();
    currentLayout = cy.layout(colaLayoutOptions);
    currentLayout.run();
  }

  function stopLayout(): void {
    currentLayout?.stop();
    currentLayout = null;
  }

  // Incremental rebuild: only add new nodes, remove gone nodes, replace
  // edges (cheap; no layout state). Preserves user-placed positions —
  // a rename or a new link does NOT scramble the layout. Intentionally
  // does NOT call scheduleSaveLayout() — only real drag events persist.
  function reconcileGraph(): void {
    if (!cy) return;
    const { nodes, edges } = buildGraphData();
    const newIds = new Set(nodes.map((n) => n.data!.id as string));
    const oldIds = new Set<string>();
    cy.nodes().forEach((n) => {
      oldIds.add(n.id());
    });

    const toRemove: string[] = [];
    for (const id of oldIds) if (!newIds.has(id)) toRemove.push(id);
    const toAdd = nodes.filter((n) => !oldIds.has(n.data!.id as string));

    cy.batch(() => {
      for (const id of toRemove) cy!.getElementById(id).remove();
      cy!.edges().remove();
      if (toAdd.length > 0) cy!.add(toAdd);
      cy!.add(edges);
    });
    annotateDegree();
    annotateColors();

    // Only run layout for genuinely new nodes — existing positions stay put.
    if (toAdd.length > 0) {
      // Keep cola running so new nodes find a place; don't re-run fcose.
      startLayout();
    }
  }

  // Full rebuild: used on workspace switch. Clears everything and re-runs
  // the initial layout pass. Called rarely.
  function hardRebuild(): void {
    if (!cy) return;
    stopLayout();
    const { nodes, edges } = buildGraphData();
    cy.batch(() => {
      cy!.elements().remove();
      cy!.add([...nodes, ...edges]);
    });
    annotateDegree();
    annotateColors();
    runInitialLayout();
    startLayout();
  }

  function applyFilters(): void {
    if (!cy) return;

    cy.elements().removeClass("hidden");

    // === View mode (Phase 4) ===
    if (viewMode !== "all") {
      const active = workspace.activePane.tabs.find(
        (t) => t.id === workspace.activePane.activeTabId,
      );
      if (!active || active.kind !== "markdown") {
        cy.elements().addClass("hidden");
      } else {
        const center = cy.getElementById(active.path);
        if (center.empty()) {
          cy.elements().addClass("hidden");
        } else {
          const depth = viewMode === "local-1" ? 1 : 2;
          let visible = center as cytoscape.CollectionReturnValue;
          for (let i = 0; i < depth; i++) {
            visible = visible.union(visible.connectedNodes());
          }
          visible = visible.union(visible.connectedEdges());
          cy.elements().addClass("hidden");
          visible.removeClass("hidden");
        }
      }
    }

    // === Folder filter (Phase 6) ===
    if (folderFilter.length > 0) {
      const root = workspace.info?.root ?? "";
      const folderSet = new Set(folderFilter);
      cy.nodes().forEach((n) => {
        if (n.hasClass("hidden")) return;
        const path = n.id();
        const rel = path.startsWith(root)
          ? path.slice(root.length).replace(/^[/\\]/, "")
          : path;
        const segs = rel.split(/[/\\]/);
        const firstSeg = segs.length > 1 ? segs[0] : "(root)";
        if (!folderSet.has(firstSeg)) n.addClass("hidden");
      });
    }

    // === Tag filter (Phase 6) ===
    if (tagFilter.length > 0) {
      const tagSet = new Set(tagFilter);
      cy.nodes().forEach((n) => {
        if (n.hasClass("hidden")) return;
        const fileTags = tagsForFile(n.id());
        const hasMatch = fileTags.some((t) => tagSet.has(t));
        if (!hasMatch) n.addClass("hidden");
      });
    }

    // === Search filter (Phase 6) ===
    const q = searchFilter.trim().toLowerCase();
    if (q) {
      cy.nodes().forEach((n) => {
        if (n.hasClass("hidden")) return;
        const label = (n.data("label") as string).toLowerCase();
        if (!label.includes(q)) n.addClass("hidden");
      });
    }

    // Hide edges with any hidden endpoint
    cy.edges().forEach((e) => {
      if (e.source().hasClass("hidden") || e.target().hasClass("hidden")) {
        e.addClass("hidden");
      }
    });
  }

  // Refit viewport to currently-visible elements. Called only for deliberate
  // scope changes (view mode, folder/tag toggle, Fit button) — NOT on every
  // search keystroke, which would jitter the viewport as the user types.
  function refitToVisible(): void {
    if (!cy) return;
    const visible = cy.elements(":visible");
    if (visible.length > 0) cy.fit(visible, 60);
    else cy.fit(undefined, 60);
  }

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

  // ===== Phase 7: layout persistence =====

  interface PersistedLayout {
    version: 1;
    savedAt: number;
    nodes: Record<string, { x: number; y: number }>;
  }

  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadLayout(): Promise<Map<string, { x: number; y: number }>> {
    if (!workspace.info?.root) return new Map();
    try {
      const data = (await loadGraphLayout()) as PersistedLayout | null;
      if (!data) return new Map();
      return new Map(Object.entries(data.nodes ?? {}));
    } catch (e) {
      console.warn("[graph] load layout failed", e);
      return new Map();
    }
  }

  function scheduleSaveLayout(): void {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void saveLayout();
    }, 1500);
  }

  async function saveLayout(): Promise<void> {
    if (!workspace.info?.root || !cy) return;
    const nodes: Record<string, { x: number; y: number }> = {};
    cy.nodes().forEach((n) => {
      const p = n.position();
      nodes[n.id()] = { x: p.x, y: p.y };
    });
    const data: PersistedLayout = {
      version: 1,
      savedAt: Date.now(),
      nodes,
    };
    try {
      await saveGraphLayout(data as unknown as Record<string, unknown>);
      const { reloadMiniGraph } = await import("./miniGraphState.svelte");
      void reloadMiniGraph();
    } catch (e) {
      console.warn("[graph] save layout failed", e);
    }
  }

  async function resetLayoutToDefault(): Promise<void> {
    try {
      // Clear saved positions by writing an empty-nodes layout.
      await saveGraphLayout({
        version: 1,
        savedAt: Date.now(),
        nodes: {},
      } as unknown as Record<string, unknown>);
    } catch (e) {
      console.warn("[graph] reset layout failed", e);
    }
    if (cy) {
      cy.nodes().unlock();
      hardRebuild();
    }
  }

  onMount(() => {
    ensureExtensions();
    void initGraph();
  });

  async function initGraph(): Promise<void> {
    const saved = await loadLayout();
    const { nodes, edges } = buildGraphData();
    cy = cytoscape({
      container: host,
      elements: [...nodes, ...edges],
      style: styleFromTheme(),
      // Defer layout — we apply saved positions or run fcose pre-pass below.
      layout: { name: "preset" } as cytoscape.LayoutOptions,
      wheelSensitivity: 0.2,
      minZoom: 0.2,
      maxZoom: 3,
    });
    annotateDegree();
    annotateColors();
    // Seed workspace-root tracker so the hard-rebuild effect doesn't
    // fire spuriously on the first tick after mount.
    lastRoot = workspace.info?.root ?? null;
    // Seed the refit tracker so the filter-refit effect doesn't double-fit
    // on mount (centerViewport below already handles the initial fit).
    lastRefitKey = [
      viewMode,
      folderFilter.slice().sort().join("|"),
      tagFilter.slice().sort().join("|"),
      viewMode !== "all" ? (workspace.activePane.activeTabId ?? "") : "",
    ].join("::");

    if (saved.size > 0) {
      // Apply saved positions, lock those nodes so layout doesn't move them
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
      if (hasNewNodes) {
        runInitialLayout();
      }
      // Unlock after a short delay so live force can take over
      setTimeout(() => {
        cy?.nodes().unlock();
        startLayout();
      }, 500);
    } else {
      runInitialLayout();
      startLayout();
    }

    // Defer the initial fit until the flex container has settled its size.
    // Two rAFs is the belt-and-braces pattern; cy.resize() forces cytoscape
    // to re-read container bounds so cy.fit lands the graph centered.
    requestAnimationFrame(() => {
      requestAnimationFrame(centerViewport);
    });

    cy.on("tap", "node", (e) => {
      const path = e.target.id() as string;
      const evt = e.originalEvent as MouseEvent | undefined;
      // Clear any pending hover-preview timer so it doesn't flash after click.
      if (hoverDelayTimer) {
        clearTimeout(hoverDelayTimer);
        hoverDelayTimer = null;
      }
      hoverPath = null;

      // Smooth center-on-selected animation
      cy?.animate({
        center: { eles: e.target },
        duration: 350,
        easing: "ease-in-out-cubic",
      });

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
    });
    cy.on("mouseover", "node", (e) => {
      if (!cy) return;
      // closedNeighborhood = node + its edges + the other endpoints of those edges.
      // That's the ego network.
      const focused = e.target.closedNeighborhood();
      cy.elements().addClass("dimmed");
      focused.removeClass("dimmed").addClass("focused");

      // Schedule hover preview after 1s
      const path = e.target.id() as string;
      const evt = e.originalEvent as MouseEvent | undefined;
      if (hoverDelayTimer) clearTimeout(hoverDelayTimer);
      hoverDelayTimer = setTimeout(async () => {
        hoverDelayTimer = null;
        let cached = getPreview(path);
        if (cached === undefined) {
          try {
            const result = await readTextFile(path);
            // Strip leading frontmatter and take first 200 chars of body
            let body = result.content;
            if (body.startsWith("---")) {
              const end = body.indexOf("\n---\n", 4);
              if (end >= 0) body = body.slice(end + 5);
            }
            cached = body.trim().slice(0, 200);
          } catch {
            cached = "(unreadable)";
          }
          setPreview(path, cached);
        }
        hoverPath = path;
        hoverContent = cached;
        if (evt) {
          hoverPos = { x: evt.clientX + 16, y: evt.clientY + 16 };
        }
      }, 1000);
    });
    cy.on("mouseout", "node", () => {
      cy?.elements().removeClass("dimmed").removeClass("focused");
      if (hoverDelayTimer) {
        clearTimeout(hoverDelayTimer);
        hoverDelayTimer = null;
      }
      hoverPath = null;
    });
    // Save layout when user drops a node after dragging
    cy.on("dragfreeon", "node", () => {
      scheduleSaveLayout();
    });

    lastFileIndexLength = workspace.fileIndex.length;
    lastBacklinksBuilt = backlinks.lastBuilt;

    themeObserver = new MutationObserver(() => {
      if (!cy) return;
      cy.style(styleFromTheme());
      // Default colorMode reads --p, which changes on theme switch; re-annotate.
      if (colorMode === "default") {
        annotateColors();
        cy.style().update();
      }
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    // Pause continuous force simulation when the tab is hidden.
    visibilityObserver = new IntersectionObserver(
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
    visibilityObserver.observe(host);
  }

  onDestroy(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      void saveLayout();
    }
    if (hoverDelayTimer) {
      clearTimeout(hoverDelayTimer);
      hoverDelayTimer = null;
    }
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

  // Refit viewport only on deliberate scope changes — view mode, folder/tag
  // filters, or local-mode center swap. Search keystrokes are intentionally
  // excluded so typing doesn't jitter the viewport.
  let lastRefitKey = "";
  $effect(() => {
    const key = [
      viewMode,
      folderFilter.slice().sort().join("|"),
      tagFilter.slice().sort().join("|"),
      viewMode !== "all" ? (workspace.activePane.activeTabId ?? "") : "",
    ].join("::");
    if (!cy) return;
    if (key === lastRefitKey) return;
    lastRefitKey = key;
    refitToVisible();
  });

  // Re-annotate node colors when colorMode changes.
  $effect(() => {
    void colorMode;
    if (!cy) return;
    annotateColors();
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
    cy.style(styleFromTheme());
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

{#if hoverPath}
  <div
    class="fixed pointer-events-none z-30 max-w-[320px] bg-base-100/95 border border-base-300 rounded-md shadow-lg p-3 text-[11px] text-base-content/80 leading-relaxed backdrop-blur"
    style:left="{hoverPos.x}px"
    style:top="{hoverPos.y}px"
  >
    <div class="font-semibold text-base-content/60 text-[10px] mb-1 truncate">
      {hoverPath.split(/[/\\]/).pop()}
    </div>
    <div class="line-clamp-6 whitespace-pre-wrap break-words">
      {hoverContent || "(empty)"}
    </div>
  </div>
{/if}
