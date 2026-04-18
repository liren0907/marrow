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
    showEdgeArrows: true,
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

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
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
  let hoverDelayTimer: ReturnType<typeof setTimeout> | null = null;

  type ViewMode = "all" | "local-1" | "local-2";
  let viewMode: ViewMode = $state("all");

  type ColorMode = "default" | "folder" | "tag";
  let colorMode: ColorMode = $state("default");

  // Display preferences (persisted to localStorage, exposed via GraphToolbar)
  const initialDisplay = loadDisplayPrefs();
  let nodeSize = $state<NodeSizePreset>(initialDisplay.nodeSize);
  let labelSize = $state<LabelSizePreset>(initialDisplay.labelSize);
  let labelMode = $state<LabelMode>(initialDisplay.labelMode);
  let showEdgeArrows = $state(initialDisplay.showEdgeArrows);
  let edgeWidth = $state(initialDisplay.edgeWidth);

  // Phase 6 filters
  let folderFilter: string[] = $state([]);
  let tagFilter: string[] = $state([]);
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

  function rebuild(): void {
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
    scheduleSaveLayout();
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

    // Fit viewport on visible subgraph if any filter is active
    const anyFilter =
      viewMode !== "all" ||
      folderFilter.length > 0 ||
      tagFilter.length > 0 ||
      q.length > 0;
    if (anyFilter) {
      const visible = cy.elements(":visible");
      if (visible.length > 0) cy.fit(visible, 60);
    }
  }

  function resetFilters(): void {
    folderFilter = [];
    tagFilter = [];
    searchFilter = "";
  }

  function fitGraph(): void {
    cy?.fit(undefined, 60);
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
      rebuild();
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
      cy.fit(undefined, 60);
    } else {
      runInitialLayout();
      startLayout();
    }

    cy.on("tap", "node", (e) => {
      const path = e.target.id() as string;
      const evt = e.originalEvent as MouseEvent | undefined;
      const wantsReplace = !!(evt && (evt.metaKey || evt.ctrlKey));

      // Smooth center-on-selected animation
      cy?.animate({
        center: { eles: e.target },
        duration: 350,
        easing: "ease-in-out-cubic",
      });

      if (wantsReplace) {
        workspace.replaceCurrentTab(path);
      } else {
        workspace.openInOtherPane(path);
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
        if (!previewCache.has(path)) {
          try {
            const result = await readTextFile(path);
            // Strip leading frontmatter and take first 200 chars of body
            let body = result.content;
            if (body.startsWith("---")) {
              const end = body.indexOf("\n---\n", 4);
              if (end >= 0) body = body.slice(end + 5);
            }
            previewCache.set(path, body.trim().slice(0, 200));
          } catch {
            previewCache.set(path, "(unreadable)");
          }
        }
        hoverPath = path;
        hoverContent = previewCache.get(path) ?? "";
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

  // Rebuild graph when fileIndex changes or backlinks index gets rebuilt.
  // Touch the reactive deps explicitly so Svelte tracks them.
  $effect(() => {
    const len = workspace.fileIndex.length;
    const built = backlinks.lastBuilt;
    if (!cy) return;
    if (len !== lastFileIndexLength || built !== lastBacklinksBuilt) {
      lastFileIndexLength = len;
      lastBacklinksBuilt = built;
      rebuild();
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

  // Re-annotate node colors when colorMode changes.
  $effect(() => {
    void colorMode;
    if (!cy) return;
    annotateColors();
    cy.style().update();
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
