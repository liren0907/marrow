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

// Frozen empty sentinel returned by Pane.svelte's `collapsedSet` $derived
// when a tab has no entry yet. Critical: the derived MUST stay pure (read
// only), so we cannot lazy-init the per-tab SvelteSet inside it — Svelte 5
// throws `state_unsafe_mutation` on writes inside $derived, and that throw
// aborts the whole reactive flush mid-tick (visible symptom: clicking a
// tab while GraphTab is mounted leaves the UI frozen because TabBar never
// re-renders). Reading `byTab.size` in the derived keeps the subscription
// live, so when `getOrCreateCollapsedSet` below first creates a real set
// for this tab the derived re-runs and consumers swap from sentinel to
// the real set automatically.
//
// DO NOT mutate this sentinel — every consumer treats it as read-only.
const EMPTY_COLLAPSED_SET: SvelteSet<number> = new SvelteSet<number>();
export function emptyCollapsedSet(): SvelteSet<number> {
  return EMPTY_COLLAPSED_SET;
}

// Imperative get-or-create for the collapsed-headings map. Call this from
// event handlers (toggle clicks, etc.) — never from inside a $derived /
// $effect that returns the result. Read paths should use the raw
// `collapsedHeadings.byTab.get(tabId)` and fall back to the sentinel.
export function getOrCreateCollapsedSet(tabId: string): SvelteSet<number> {
  let set = collapsedHeadings.byTab.get(tabId);
  if (!set) {
    set = new SvelteSet<number>();
    collapsedHeadings.byTab.set(tabId, set);
  }
  return set;
}

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
