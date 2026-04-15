import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { DirEntry } from "../workspace/types";
import { listDirectory } from "../workspace/tauri";
import { workspace } from "../workspace/workspace.svelte";

const children = new SvelteMap<string, DirEntry[]>();
const expanded = new SvelteSet<string>();
const loading = new SvelteSet<string>();

function parentOf(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx > 0 ? path.slice(0, idx) : path;
}

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

  async expand(path: string): Promise<void> {
    if (expanded.has(path)) return;
    expanded.add(path);
    if (!children.has(path)) {
      await tree.load(path);
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

export async function revealInTree(absPath: string): Promise<void> {
  const root = workspace.info?.root;
  if (!root || !absPath.startsWith(root)) return;

  // Collect ancestor directories from workspace root down to (but not
  // including) the target itself. Expanding these guarantees the target
  // node is rendered in the DOM so querySelector can find it.
  const ancestors: string[] = [];
  let cur = parentOf(absPath);
  while (cur.length > root.length && cur.startsWith(root)) {
    ancestors.push(cur);
    cur = parentOf(cur);
  }
  ancestors.reverse();

  for (const dir of ancestors) {
    await tree.expand(dir);
  }

  // Two rAFs to let Svelte commit the expansion render before querying.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      const el = document.querySelector<HTMLElement>(
        `[data-marrow-tree-path="${CSS.escape(absPath)}"]`,
      );
      if (!el) return;
      el.scrollIntoView({ block: "nearest", behavior: "smooth" });
      el.classList.remove("reveal-flash");
      // Force reflow so re-adding the class restarts the animation.
      void el.offsetWidth;
      el.classList.add("reveal-flash");
      setTimeout(() => el.classList.remove("reveal-flash"), 900);
    });
  });
}
