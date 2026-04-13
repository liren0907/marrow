import { searchWorkspace, type SearchHit } from "$lib/workspace/tauri";
import { workspace } from "$lib/workspace/workspace.svelte";

export const search = $state<{
  isOpen: boolean;
  query: string;
  results: SearchHit[];
  isSearching: boolean;
  selectedIdx: number;
  error: string | null;
}>({
  isOpen: false,
  query: "",
  results: [],
  isSearching: false,
  selectedIdx: 0,
  error: null,
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let queryToken = 0;

export function scheduleSearch(): void {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    debounceTimer = null;
    void runSearch();
  }, 300);
}

async function runSearch(): Promise<void> {
  const root = workspace.info?.root;
  const q = search.query.trim();
  if (!root || q.length < 2) {
    search.results = [];
    search.error = null;
    search.isSearching = false;
    return;
  }
  const myToken = ++queryToken;
  search.isSearching = true;
  search.error = null;
  try {
    const hits = await searchWorkspace(root, q, 200);
    if (myToken !== queryToken) return; // a newer query superseded us
    search.results = hits;
    search.selectedIdx = 0;
  } catch (e) {
    if (myToken !== queryToken) return;
    search.error = e instanceof Error ? e.message : String(e);
    search.results = [];
  } finally {
    if (myToken === queryToken) {
      search.isSearching = false;
    }
  }
}

export function openSearch(): void {
  search.isOpen = true;
}

export function closeSearch(): void {
  search.isOpen = false;
}

export function toggleSearch(): void {
  search.isOpen = !search.isOpen;
}
