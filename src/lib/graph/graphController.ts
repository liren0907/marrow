// All cy-coupled commands for the graph view.
//
// Every function takes `cy` (and any other deps) as parameters — there is
// NO module-level state. This keeps the cy reference firmly in GraphTab's
// `let` binding (Svelte 5 proxy would corrupt it if it ever entered $state).
//
// FilterState is a snapshot of the component's reactive state at the moment
// applyFilters runs — the controller doesn't reach into stores so the
// caller stays in charge of when filters are computed.

import type cytoscape from "cytoscape";
import { tagsForFile } from "$lib/workspace/tagIndex.svelte";
import { buildGraphData } from "./data";
import { colorForNode } from "./style";
import { fcoseInitOptions, colaLayoutOptions } from "./layoutConfig";
import type { ViewMode, ColorMode } from "./prefs";

// ─── Annotation helpers (per-node data writes) ───────────────────────────

// Write each node's connection count into `data.degree` so the stylesheet's
// mapData(degree, …) sizing works. Must run AFTER edges are added.
export function annotateDegree(cy: cytoscape.Core): void {
  cy.nodes().forEach((n) => {
    n.data("degree", n.connectedEdges().length);
  });
}

// Write each node's color into `data.color` based on the active colorMode.
// styleFromTheme reads `data(color)` for the background-color rule.
export function annotateColors(
  cy: cytoscape.Core,
  colorMode: ColorMode,
): void {
  cy.nodes().forEach((n) => {
    n.data("color", colorForNode(n.id(), colorMode));
  });
}

// ─── Layout commands ─────────────────────────────────────────────────────

// First-paint pass — fcose places nodes deterministically and cheaply.
export function runInitialLayout(cy: cytoscape.Core): void {
  cy.layout(fcoseInitOptions).run();
}

// Create + start a continuous cola simulation. Caller MUST hold the
// returned handle in a plain `let` (NOT $state) so it can call .stop()
// later. cytoscape's Layouts handle has internal state that Svelte's
// proxy would corrupt if observed.
export function runColaLayout(cy: cytoscape.Core): cytoscape.Layouts {
  const layout = cy.layout(colaLayoutOptions);
  layout.run();
  return layout;
}

// ─── Graph structure sync ────────────────────────────────────────────────

// Incremental rebuild: add new nodes, remove gone nodes, replace all edges.
// Preserves user-placed positions — a rename or new link does NOT scramble
// the layout. Returns `true` if new nodes were added (so the caller can
// kick off cola to find places for them); `false` if nothing changed.
//
// Intentionally does NOT trigger a layout save — only real drag events
// should mark positions as user-authored.
export function reconcileGraph(
  cy: cytoscape.Core,
  colorMode: ColorMode,
): boolean {
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
    for (const id of toRemove) cy.getElementById(id).remove();
    cy.edges().remove();
    if (toAdd.length > 0) cy.add(toAdd);
    cy.add(edges);
  });
  annotateDegree(cy);
  annotateColors(cy, colorMode);

  return toAdd.length > 0;
}

// Full rebuild: clear everything, re-add, run initial fcose. Used on
// workspace switch — rare. Caller is responsible for kicking off cola
// after this returns (the layout pipeline is fcose → cola).
export function hardRebuild(
  cy: cytoscape.Core,
  colorMode: ColorMode,
): void {
  const { nodes, edges } = buildGraphData();
  cy.batch(() => {
    cy.elements().remove();
    cy.add([...nodes, ...edges]);
  });
  annotateDegree(cy);
  annotateColors(cy, colorMode);
  runInitialLayout(cy);
}

// ─── Filter pipeline ─────────────────────────────────────────────────────

export interface FilterState {
  viewMode: ViewMode;
  folderFilter: string[];
  tagFilter: string[];
  searchFilter: string;
  workspaceRoot: string;
  // Path of the currently-active markdown note, or null. Required by
  // local-N modes which center their visible set on this path.
  activeMarkdownPath: string | null;
}

// Apply combined view-mode + folder + tag + search filters by toggling the
// `.hidden` class. Edges become hidden whenever either endpoint is hidden.
// Idempotent — clears everything first so it can be re-run on any state
// change.
export function applyFilters(
  cy: cytoscape.Core,
  state: FilterState,
): void {
  cy.elements().removeClass("hidden");

  // === View mode (local-N graph) ===
  if (state.viewMode !== "all") {
    if (state.activeMarkdownPath === null) {
      cy.elements().addClass("hidden");
    } else {
      const center = cy.getElementById(state.activeMarkdownPath);
      if (center.empty()) {
        cy.elements().addClass("hidden");
      } else {
        const depth = state.viewMode === "local-1" ? 1 : 2;
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

  // === Folder filter ===
  if (state.folderFilter.length > 0) {
    const root = state.workspaceRoot;
    const folderSet = new Set(state.folderFilter);
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

  // === Tag filter ===
  if (state.tagFilter.length > 0) {
    const tagSet = new Set(state.tagFilter);
    cy.nodes().forEach((n) => {
      if (n.hasClass("hidden")) return;
      const fileTags = tagsForFile(n.id());
      const hasMatch = fileTags.some((t) => tagSet.has(t));
      if (!hasMatch) n.addClass("hidden");
    });
  }

  // === Search filter ===
  const q = state.searchFilter.trim().toLowerCase();
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

// ─── Camera ──────────────────────────────────────────────────────────────

// Refit viewport to currently-visible elements with 60px padding. Used
// only on deliberate scope changes (view mode, folder/tag toggle, Fit
// button). Search keystrokes are intentionally NOT a trigger — that would
// jitter the viewport as the user types.
export function refitToVisible(cy: cytoscape.Core): void {
  const visible = cy.elements(":visible");
  if (visible.length > 0) cy.fit(visible, 60);
  else cy.fit(undefined, 60);
}
