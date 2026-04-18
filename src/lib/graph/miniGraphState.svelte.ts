import { loadGraphLayout } from "$lib/workspace/tauri";

export interface NodePos {
  path: string;
  x: number;
  y: number;
}

export const miniGraph = $state<{
  nodes: NodePos[];
  loaded: boolean;
  loading: boolean;
}>({
  nodes: [],
  loaded: false,
  loading: false,
});

/**
 * Read the saved graph layout (as written by GraphTab) and cache it
 * for the sidebar mini-panel. Schema is owned by GraphTab:
 *   { version: 1, savedAt: number, nodes: Record<nodeId, {x, y}> }
 * where `nodeId` is the file path.
 */
export async function reloadMiniGraph(): Promise<void> {
  if (miniGraph.loading) return;
  miniGraph.loading = true;
  try {
    const raw = (await loadGraphLayout()) as {
      nodes?: Record<string, { x: number; y: number }>;
    } | null;
    const out: NodePos[] = [];
    if (raw?.nodes) {
      for (const [path, pos] of Object.entries(raw.nodes)) {
        if (
          pos &&
          typeof pos.x === "number" &&
          typeof pos.y === "number" &&
          Number.isFinite(pos.x) &&
          Number.isFinite(pos.y)
        ) {
          out.push({ path, x: pos.x, y: pos.y });
        }
      }
    }
    miniGraph.nodes = out;
    miniGraph.loaded = true;
  } catch {
    miniGraph.nodes = [];
    miniGraph.loaded = true;
  } finally {
    miniGraph.loading = false;
  }
}

export function clearMiniGraph(): void {
  miniGraph.nodes = [];
  miniGraph.loaded = false;
}
