// Convert the workspace's flat fileIndex + backlinks reverse index into the
// node + edge arrays cytoscape consumes.
//
// Edge dedup logic: multiple [[ref]]s from A to B → ONE edge. If both
// A→B and B→A exist, they collapse into a single edge marked `bi`
// (rendered with arrows on both ends in styleFromTheme). Self-links
// (A → A) are filtered out — cytoscape can render them but they're
// usually noise from typing your own filename in your own note.

import type cytoscape from "cytoscape";
import { workspace } from "$lib/workspace/workspace.svelte";
import { backlinks } from "$lib/workspace/backlinkIndex.svelte";

export function buildGraphData(): {
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
