import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { DirEntry } from "../workspace/types";
import { listDirectory } from "../workspace/tauri";

const children = new SvelteMap<string, DirEntry[]>();
const expanded = new SvelteSet<string>();
const loading = new SvelteSet<string>();

export const tree = {
  isExpanded(path: string): boolean {
    return expanded.has(path);
  },
  isLoading(path: string): boolean {
    return loading.has(path);
  },
  getChildren(path: string): DirEntry[] | undefined {
    return children.get(path);
  },

  async load(path: string): Promise<void> {
    if (loading.has(path)) return;
    loading.add(path);
    try {
      const entries = await listDirectory(path);
      children.set(path, entries);
    } finally {
      loading.delete(path);
    }
  },

  async toggle(path: string): Promise<void> {
    if (expanded.has(path)) {
      expanded.delete(path);
      return;
    }
    expanded.add(path);
    if (!children.has(path)) {
      await tree.load(path);
    }
  },

  reset(): void {
    children.clear();
    expanded.clear();
    loading.clear();
  },
};
