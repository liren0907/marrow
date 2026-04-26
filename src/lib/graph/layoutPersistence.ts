// Disk persistence for graph node positions.
//
// Layout is saved per-workspace via Tauri commands; the actual file lives
// next to the .marrow workspace metadata. After a successful save we kick
// the mini-graph reloader so the sidebar overview stays in sync.
//
// `createLayoutSaveScheduler` returns a debouncer bound to a specific
// `cy` handle — its internal timer is closure-private, avoiding any
// module-level mutable state.

import type cytoscape from "cytoscape";
import { workspace } from "$lib/workspace/workspace.svelte";
import {
  loadGraphLayout,
  saveGraphLayout,
} from "$lib/workspace/tauri";

export interface PersistedLayout {
  version: 1;
  savedAt: number;
  nodes: Record<string, { x: number; y: number }>;
}

const DEFAULT_DEBOUNCE_MS = 1500;

export async function loadLayout(): Promise<
  Map<string, { x: number; y: number }>
> {
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

export async function saveLayout(cy: cytoscape.Core): Promise<void> {
  if (!workspace.info?.root) return;
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
    // Refresh the mini-graph in the sidebar so it reflects the saved
    // positions immediately.
    const { reloadMiniGraph } = await import("./miniGraphState.svelte");
    void reloadMiniGraph();
  } catch (e) {
    console.warn("[graph] save layout failed", e);
  }
}

// Wipe persisted positions back to "no saved layout". Caller is responsible
// for unlocking nodes and re-running the layout pipeline afterwards (this
// function only touches disk, not the live cy).
export async function clearSavedLayout(): Promise<void> {
  try {
    await saveGraphLayout({
      version: 1,
      savedAt: Date.now(),
      nodes: {},
    } as unknown as Record<string, unknown>);
  } catch (e) {
    console.warn("[graph] reset layout failed", e);
  }
}

export interface LayoutSaveScheduler {
  /** Debounced save — coalesces bursts of drag-finish events. */
  schedule(): void;
  /** Cancel pending timer + immediately save. Use on tab destroy. */
  flush(): Promise<void>;
  /** Cancel pending timer without saving. */
  cancel(): void;
}

// Bind a debounced save to a specific cy handle. The returned scheduler
// is one-shot per GraphTab mount — destroy by calling flush() on cleanup.
export function createLayoutSaveScheduler(
  cy: cytoscape.Core,
  delayMs: number = DEFAULT_DEBOUNCE_MS,
): LayoutSaveScheduler {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return {
    schedule(): void {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        void saveLayout(cy);
      }, delayMs);
    },
    async flush(): Promise<void> {
      if (timer) {
        clearTimeout(timer);
        timer = null;
        await saveLayout(cy);
      }
    },
    cancel(): void {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    },
  };
}
