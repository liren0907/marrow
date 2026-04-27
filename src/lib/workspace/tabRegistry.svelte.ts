import { SvelteMap, SvelteSet } from "svelte/reactivity";

export interface Heading {
  level: 1 | 2 | 3 | 4 | 5 | 6;
  text: string;
  pos: number;
}

export const outlines = $state<{ byTab: SvelteMap<string, Heading[]> }>({
  byTab: new SvelteMap(),
});

// Currently-active heading position per tab — pushed by MilkdownEditor's
// scroll listener so the outline aside can highlight "where am I right
// now" as the user reads. `null` = scrolled above any heading (or no
// heading yet detected).
export const activeHeading = $state<{ byTab: SvelteMap<string, number | null> }>({
  byTab: new SvelteMap(),
});

// Per-tab in-memory set of heading positions whose subtree the user has
// collapsed in the outline. Lives outside the tab object so it survives
// outline rebuilds without forcing a Tab patch on every keystroke. Stale
// entries (positions that no longer exist after edits) are pruned by
// Pane.svelte when the outline updates.
export const collapsedHeadings = $state<{ byTab: SvelteMap<string, SvelteSet<number>> }>({
  byTab: new SvelteMap(),
});

export const tabScrollRegistry = new Map<string, (pos: number) => void>();

export function registerTabScroll(
  tabId: string,
  fn: (pos: number) => void,
): void {
  tabScrollRegistry.set(tabId, fn);
}

export function unregisterTabScroll(tabId: string): void {
  tabScrollRegistry.delete(tabId);
}

export const tabPeekRegistry = new Map<string, () => void>();

export function registerTabPeek(tabId: string, fn: () => void): void {
  tabPeekRegistry.set(tabId, fn);
}

export function unregisterTabPeek(tabId: string): void {
  tabPeekRegistry.delete(tabId);
}
