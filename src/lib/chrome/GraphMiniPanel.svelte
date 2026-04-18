<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinks, backlinksFor } from "$lib/workspace/backlinkIndex.svelte";
  import { miniGraph, reloadMiniGraph } from "$lib/graph/miniGraphState.svelte";

  const activeTab = $derived.by(() => {
    const pane = workspace.activePane;
    if (!pane.activeTabId) return null;
    return pane.tabs.find((t) => t.id === pane.activeTabId) ?? null;
  });

  const activeNotePath = $derived(
    activeTab?.kind === "markdown" ? activeTab.path : null,
  );

  // Mount-time: if we haven't loaded yet and there's a workspace open,
  // refresh — covers the case where the sidebar graph panel is opened
  // before the workspace.open() import chain has settled.
  $effect(() => {
    if (!miniGraph.loaded && workspace.info) void reloadMiniGraph();
  });

  const nodes = $derived(miniGraph.nodes);
  const hasLayout = $derived(nodes.length > 0);

  // Neighborhood — same as before.
  const neighborhood = $derived.by(() => {
    if (!activeTab || activeTab.kind !== "markdown") return [];
    void backlinks.lastBuilt;
    const incoming = backlinksFor(activeTab.path).map((e) => ({
      path: e.sourcePath,
      direction: "in" as const,
    }));
    const outgoing: { path: string; direction: "out" }[] = [];
    for (const [, entries] of backlinks.byTarget) {
      for (const e of entries) {
        if (e.sourcePath === activeTab.path) {
          const resolved = workspace.resolveBasename(e.target);
          if (resolved && resolved !== activeTab.path) {
            outgoing.push({ path: resolved, direction: "out" });
          }
        }
      }
    }
    const seen = new Set<string>();
    const merged: { path: string; direction: "in" | "out" | "both" }[] = [];
    for (const item of [...outgoing, ...incoming]) {
      const existing = merged.find((m) => m.path === item.path);
      if (existing) {
        if (existing.direction !== item.direction) existing.direction = "both";
      } else if (!seen.has(item.path)) {
        merged.push({ ...item });
        seen.add(item.path);
      }
    }
    return merged.slice(0, 40);
  });

  // Edge list derived from backlinks — direction-agnostic (we render undirected lines).
  const edges = $derived.by(() => {
    const out: { source: string; target: string }[] = [];
    void backlinks.lastBuilt;
    const pathSet = new Set(nodes.map((n) => n.path));
    for (const [, entries] of backlinks.byTarget) {
      for (const e of entries) {
        const resolved = workspace.resolveBasename(e.target);
        if (!resolved) continue;
        if (pathSet.has(e.sourcePath) && pathSet.has(resolved)) {
          out.push({ source: e.sourcePath, target: resolved });
        }
      }
    }
    return out;
  });

  function basename(path: string): string {
    const n = path.split(/[/\\]/).pop() ?? path;
    return n.replace(/\.md$/i, "");
  }

  function open(path: string): void {
    workspace.replaceCurrentTab(path);
  }

  function openGraph(): void {
    workspace.openGraph();
  }

  // ─── Canvas rendering ─────────────────────────────────────────
  let canvasEl = $state<HTMLCanvasElement | undefined>();
  let wrapEl = $state<HTMLDivElement | undefined>();

  interface ScreenNode {
    path: string;
    sx: number;
    sy: number;
  }
  let screenNodes: ScreenNode[] = [];

  function cssColor(varName: string, fallback: string): string {
    if (typeof getComputedStyle === "undefined") return fallback;
    const el = document.documentElement;
    const v = getComputedStyle(el).getPropertyValue(varName).trim();
    return v || fallback;
  }

  function draw(): void {
    const canvas = canvasEl;
    const wrap = wrapEl;
    if (!canvas || !wrap) return;

    const rect = wrap.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = rect.width + "px";
    canvas.style.height = rect.height + "px";
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, rect.width, rect.height);

    if (nodes.length === 0) {
      screenNodes = [];
      return;
    }

    // Compute bounds.
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const n of nodes) {
      if (n.x < minX) minX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.x > maxX) maxX = n.x;
      if (n.y > maxY) maxY = n.y;
    }
    const pad = 10;
    const srcW = Math.max(1, maxX - minX);
    const srcH = Math.max(1, maxY - minY);
    const scale = Math.min(
      (rect.width - pad * 2) / srcW,
      (rect.height - pad * 2) / srcH,
    );
    const offX = (rect.width - srcW * scale) / 2 - minX * scale;
    const offY = (rect.height - srcH * scale) / 2 - minY * scale;

    // Build screen-space node lookup.
    const lookup = new Map<string, ScreenNode>();
    screenNodes = nodes.map((n) => {
      const sn = { path: n.path, sx: n.x * scale + offX, sy: n.y * scale + offY };
      lookup.set(n.path, sn);
      return sn;
    });

    const accent = cssColor("--mw-accent", "oklch(0.72 0.13 55)");
    const ink3 = cssColor("--mw-ink-3", "#6b6358");
    const rule = cssColor("--mw-rule-strong", "rgba(230,215,190,0.14)");

    // Edges.
    ctx.strokeStyle = rule;
    ctx.lineWidth = 1;
    for (const e of edges) {
      const a = lookup.get(e.source);
      const b = lookup.get(e.target);
      if (!a || !b) continue;
      const isActive =
        activeNotePath && (e.source === activeNotePath || e.target === activeNotePath);
      ctx.strokeStyle = isActive ? accent : rule;
      ctx.globalAlpha = isActive ? 0.8 : 0.6;
      ctx.beginPath();
      ctx.moveTo(a.sx, a.sy);
      ctx.lineTo(b.sx, b.sy);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;

    // Nodes.
    for (const sn of screenNodes) {
      const isActive = sn.path === activeNotePath;
      ctx.fillStyle = isActive ? accent : ink3;
      ctx.beginPath();
      ctx.arc(sn.sx, sn.sy, isActive ? 3.5 : 2, 0, Math.PI * 2);
      ctx.fill();
      if (isActive) {
        ctx.strokeStyle = accent;
        ctx.lineWidth = 1;
        ctx.globalAlpha = 0.4;
        ctx.beginPath();
        ctx.arc(sn.sx, sn.sy, 6, 0, Math.PI * 2);
        ctx.stroke();
        ctx.globalAlpha = 1;
      }
    }
  }

  $effect(() => {
    // Redraw whenever the inputs change.
    void nodes;
    void edges;
    void activeNotePath;
    queueMicrotask(draw);
  });

  $effect(() => {
    const wrap = wrapEl;
    if (!wrap) return;
    const ro = new ResizeObserver(() => draw());
    ro.observe(wrap);
    return () => ro.disconnect();
  });

  function onCanvasClick(e: MouseEvent): void {
    const canvas = canvasEl;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    let best: { path: string; d2: number } | null = null;
    for (const sn of screenNodes) {
      const dx = sx - sn.sx;
      const dy = sy - sn.sy;
      const d2 = dx * dx + dy * dy;
      if (!best || d2 < best.d2) best = { path: sn.path, d2 };
    }
    if (best && best.d2 < 100) open(best.path);
  }
</script>

<div class="panel">
  <div class="panel-header mw-meta">Graph</div>
  {#if hasLayout}
    <div
      bind:this={wrapEl}
      class="mini-graph-live"
      onclick={onCanvasClick}
      role="button"
      tabindex="0"
      title="Click a node to open · ⇧⌘G for full graph"
      onkeydown={(e) => { if (e.key === "Enter") openGraph(); }}
    >
      <canvas bind:this={canvasEl} class="mini-graph-canvas"></canvas>
    </div>
    <button type="button" class="mini-graph-expand mw-meta" onclick={openGraph}>
      Open full graph · ⇧⌘G
    </button>
  {:else}
    <div
      class="mini-graph-placeholder"
      role="button"
      tabindex="0"
      onclick={openGraph}
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); openGraph(); } }}
      title="Open full graph view (⇧⌘G)"
    >
      <span class="material-symbols-rounded placeholder-icon">hub</span>
      <span class="placeholder-text">Open full graph view</span>
      <span class="placeholder-kbd">⇧⌘G</span>
    </div>
  {/if}

  <div class="panel-header mw-meta">Neighborhood</div>
  <div class="neighborhood">
    {#if !activeTab || activeTab.kind !== "markdown"}
      <p class="panel-empty">Open a note to see its neighborhood</p>
    {:else if neighborhood.length === 0}
      <p class="panel-empty">No linked notes</p>
    {:else}
      {#each neighborhood as n (n.path)}
        <button
          type="button"
          class="neighborhood-row"
          onclick={() => open(n.path)}
          title={n.path}
        >
          <span
            class="neighborhood-dot"
            class:in={n.direction === "in"}
            class:out={n.direction === "out"}
            class:both={n.direction === "both"}
          ></span>
          <span class="neighborhood-name">{basename(n.path)}</span>
          <span class="neighborhood-dir">
            {n.direction === "in" ? "←" : n.direction === "out" ? "→" : "↔"}
          </span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }
  .panel-header {
    padding: 10px 14px 4px;
  }
  .mini-graph-live {
    margin: 4px 10px 4px;
    height: 180px;
    border: 1px solid var(--mw-rule);
    border-radius: 4px;
    overflow: hidden;
    background: var(--color-base-100);
    cursor: pointer;
    position: relative;
  }
  .mini-graph-canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
  .mini-graph-expand {
    margin: 0 12px 12px;
    padding: 4px 8px;
    background: transparent;
    border: none;
    text-align: left;
    color: var(--mw-ink-2);
    cursor: pointer;
    border-radius: 3px;
  }
  .mini-graph-expand:hover {
    color: var(--color-base-content);
    background: var(--color-base-300);
  }
  .mini-graph-placeholder {
    margin: 4px 10px 12px;
    padding: 28px 12px;
    border: 1px dashed var(--mw-rule-strong);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    color: var(--mw-ink-2);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    text-align: center;
  }
  .mini-graph-placeholder:hover {
    color: var(--color-base-content);
    background: var(--color-base-300);
  }
  .placeholder-icon {
    font-size: 32px;
    color: var(--mw-accent);
  }
  .placeholder-text {
    font-size: 12px;
  }
  .placeholder-kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--mw-ink-3);
  }
  .neighborhood {
    flex: 1;
    overflow-y: auto;
    padding: 0 6px 20px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .neighborhood-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 8px;
    font-size: 12px;
    color: var(--mw-ink-1);
    cursor: pointer;
    border-radius: 3px;
    background: transparent;
    border: none;
    text-align: left;
    width: 100%;
  }
  .neighborhood-row:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  .neighborhood-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--mw-ink-3);
    flex-shrink: 0;
  }
  .neighborhood-dot.in {
    background: var(--mw-ink-2);
  }
  .neighborhood-dot.out {
    background: var(--mw-accent);
  }
  .neighborhood-dot.both {
    background: var(--mw-accent);
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--mw-accent) 25%, transparent);
  }
  .neighborhood-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .neighborhood-dir {
    color: var(--mw-ink-3);
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .panel-empty {
    padding: 12px 14px;
    color: var(--mw-ink-3);
    font-size: 12px;
    font-style: italic;
  }
</style>
