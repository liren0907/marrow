import {
  searchWorkspace,
  searchAllWorkspaces,
  type CrossHit,
} from "$lib/workspace/tauri";
import { workspace } from "$lib/workspace/workspace.svelte";
import { advancedSettings } from "$lib/settings/advancedSettings.svelte";
import { editorSettings } from "$lib/settings/editorSettings.svelte";

export type SearchScope = "current" | "all";

const SCOPE_KEY = "marrow.search.scope";

function loadScope(): SearchScope {
  if (typeof localStorage === "undefined") return "current";
  const v = localStorage.getItem(SCOPE_KEY);
  return v === "all" ? "all" : "current";
}

export const search = $state<{
  isOpen: boolean;
  query: string;
  scope: SearchScope;
  results: CrossHit[];
  isSearching: boolean;
  selectedIdx: number;
  error: string | null;
}>({
  isOpen: false,
  query: "",
  scope: loadScope(),
  results: [],
  isSearching: false,
  selectedIdx: 0,
  error: null,
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let queryToken = 0;

export function scheduleSearch(): void {
  if (debounceTimer) clearTimeout(debounceTimer);
  // Read live so changing the setting takes effect on the very next
  // keystroke without restart.
  debounceTimer = setTimeout(() => {
    debounceTimer = null;
    void runSearch();
  }, editorSettings.searchDebounceMs);
}

export function setScope(scope: SearchScope): void {
  if (search.scope === scope) return;
  search.scope = scope;
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(SCOPE_KEY, scope);
  }
  scheduleSearch();
}

async function runSearch(): Promise<void> {
  const q = search.query.trim();
  if (q.length < 2) {
    search.results = [];
    search.error = null;
    search.isSearching = false;
    return;
  }
  const myToken = ++queryToken;
  search.isSearching = true;
  search.error = null;
  try {
    // Read once per query so a mid-flight setting change doesn't
    // produce mismatched scope vs current-workspace result counts.
    const limit = advancedSettings.searchResultLimit;
    let out: CrossHit[];
    if (search.scope === "all") {
      out = await searchAllWorkspaces(q, limit);
    } else {
      const info = workspace.info;
      if (!info) {
        if (myToken === queryToken) {
          search.results = [];
          search.isSearching = false;
        }
        return;
      }
      const hits = await searchWorkspace(info.root, q, limit);
      out = hits.map((hit) => ({
        workspace_id: info.root,
        workspace_name: info.name,
        workspace_root: info.root,
        hit,
      }));
    }
    if (myToken !== queryToken) return;
    search.results = out;
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
