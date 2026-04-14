import { readTextFile } from "$lib/workspace/tauri";
import { basename } from "$lib/workspace/fileKind";
import { bottomPanel } from "$lib/panels/bottomPanelState.svelte";
import { showWarning, showError } from "$lib/stores/toastStore.svelte";

export interface PeekLayer {
  path: string;
  section: string | null;
  content: string;
  scrollY: number;
  label: string;
}

export const MAX_PEEK_DEPTH = 5;

const state = $state<{ layers: PeekLayer[] }>({ layers: [] });

function sameKey(a: PeekLayer, path: string, section: string | null): boolean {
  return a.path === path && a.section === section;
}

export const peek = {
  get layers(): PeekLayer[] {
    return state.layers;
  },
  get current(): PeekLayer | null {
    return state.layers[state.layers.length - 1] ?? null;
  },
  get isOpen(): boolean {
    return state.layers.length > 0;
  },
  get depth(): number {
    return state.layers.length;
  },

  async push(path: string, section: string | null): Promise<void> {
    if (state.layers.length >= MAX_PEEK_DEPTH) {
      showWarning(`Peek depth limit (${MAX_PEEK_DEPTH}) reached`);
      return;
    }
    const top = state.layers[state.layers.length - 1];
    if (top && sameKey(top, path, section)) return;

    let content: string;
    try {
      const result = await readTextFile(path);
      content = result.content;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(`Peek failed: ${msg}`);
      return;
    }

    const base = basename(path);
    const label = section ? `${base} \u203A ${section}` : base;
    state.layers.push({
      path,
      section,
      content,
      scrollY: 0,
      label,
    });

    if (!bottomPanel.isOpen) bottomPanel.isOpen = true;
    bottomPanel.activeTab = "peek";
  },

  pop(): void {
    state.layers.pop();
  },

  popTo(targetDepth: number): void {
    while (state.layers.length > targetDepth) state.layers.pop();
  },

  clear(): void {
    state.layers = [];
  },

  rememberScroll(y: number): void {
    const top = state.layers[state.layers.length - 1];
    if (top) top.scrollY = y;
  },
};
