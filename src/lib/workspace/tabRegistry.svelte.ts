import { SvelteMap } from "svelte/reactivity";

export interface Heading {
  level: 1 | 2 | 3 | 4 | 5 | 6;
  text: string;
  pos: number;
}

export const outlines = $state<{ byTab: SvelteMap<string, Heading[]> }>({
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
