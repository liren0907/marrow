// Cytoscape stylesheet generator + per-node color resolver.
//
// Both functions read CSS custom properties at call time, so they MUST run
// in browser context (post-mount). The hashHue + colorForNode pair gives
// stable, deterministic colors keyed on folder name or first tag — same
// name → same hue across sessions, no localStorage needed.

import type cytoscape from "cytoscape";
import { workspace } from "$lib/workspace/workspace.svelte";
import { tagsForFile } from "$lib/workspace/tagIndex.svelte";
import {
  NODE_SIZE_MAP,
  LABEL_SIZE_MAP,
  type NodeSizePreset,
  type LabelSizePreset,
  type LabelMode,
  type ColorMode,
} from "./prefs";

// Stable string-to-hue hash. Same input → same hue across sessions.
// Public (vs internal helper) so other features can reuse it for keeping
// colors in sync — e.g. tag chips matching graph node colors.
export function hashHue(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  return h % 360;
}

export function colorForNode(path: string, mode: ColorMode): string {
  if (mode === "default") {
    return (
      getComputedStyle(document.documentElement)
        .getPropertyValue("--color-primary")
        .trim() || "oklch(0.6 0.2 250)"
    );
  }
  if (mode === "folder") {
    const root = workspace.info?.root ?? "";
    const rel = path.startsWith(root)
      ? path.slice(root.length).replace(/^[/\\]/, "")
      : path;
    const segs = rel.split(/[/\\]/);
    const firstSeg = segs.length > 1 ? segs[0] : "(root)";
    return `oklch(0.7 0.18 ${hashHue(firstSeg)})`;
  }
  if (mode === "tag") {
    const fileTags = tagsForFile(path);
    if (fileTags.length === 0) return `oklch(0.55 0.02 250)`;
    return `oklch(0.7 0.18 ${hashHue(fileTags[0])})`;
  }
  return `oklch(0.6 0.2 250)`;
}

export interface StyleInputs {
  nodeSize: NodeSizePreset;
  labelSize: LabelSizePreset;
  labelMode: LabelMode;
  showEdgeArrows: boolean;
  edgeWidth: number;
}

export function styleFromTheme(
  inputs: StyleInputs,
): cytoscape.StylesheetStyle[] {
  const root = getComputedStyle(document.documentElement);
  // DaisyUI v5 returns full oklch(...) strings (not unitless coords), so the
  // values flow through as-is into cytoscape stylesheet color fields.
  const bc =
    root.getPropertyValue("--color-base-content").trim() || "oklch(0 0 0)";
  const p =
    root.getPropertyValue("--color-primary").trim() || "oklch(0.6 0.2 250)";
  const b3 =
    root.getPropertyValue("--color-base-300").trim() || "oklch(0.85 0 0)";
  const accent =
    root.getPropertyValue("--mw-accent").trim() || "oklch(0.72 0.13 55)";

  const [nMin, nMax] = NODE_SIZE_MAP[inputs.nodeSize];
  const fontPx = LABEL_SIZE_MAP[inputs.labelSize];
  const labelHidden = fontPx === 0;
  const baseTextOpacity =
    labelHidden || inputs.labelMode === "hover" ? 0 : 1;
  const arrowShape = inputs.showEdgeArrows ? "triangle" : "none";
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
        width: inputs.edgeWidth,
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
