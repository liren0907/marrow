<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import type { SearchHit } from "$lib/workspace/tauri";
  import { search, scheduleSearch, closeSearch } from "./searchState.svelte";

  let inputEl: HTMLInputElement | undefined = $state();

  // Group consecutive hits in the same file so we render a single file header
  // with multiple match rows underneath.
  interface HitGroup {
    path: string;
    relPath: string;
    hits: SearchHit[];
  }

  const groups = $derived.by(() => {
    const out: HitGroup[] = [];
    const root = workspace.info?.root ?? "";
    for (const hit of search.results) {
      const last = out[out.length - 1];
      if (last && last.path === hit.path) {
        last.hits.push(hit);
      } else {
        let rel = hit.path;
        if (root && hit.path.startsWith(root)) {
          rel = hit.path.slice(root.length).replace(/^[/\\]/, "");
        }
        out.push({ path: hit.path, relPath: rel, hits: [hit] });
      }
    }
    return out;
  });

  // Flat index of selectable rows (each hit is a row).
  const flatHits = $derived(search.results);

  $effect(() => {
    if (search.isOpen) {
      queueMicrotask(() => inputEl?.focus());
      if (search.selectedIdx >= flatHits.length) search.selectedIdx = 0;
    }
  });

  function pick(idx: number, newTab: boolean): void {
    const hit = flatHits[idx];
    if (!hit) return;
    if (newTab) {
      workspace.openFile(hit.path);
    } else {
      workspace.replaceCurrentTab(hit.path);
    }
    closeSearch();
  }

  function onInputKeydown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      closeSearch();
      return;
    }
    if (flatHits.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      search.selectedIdx = (search.selectedIdx + 1) % flatHits.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      search.selectedIdx =
        (search.selectedIdx - 1 + flatHits.length) % flatHits.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(search.selectedIdx, e.metaKey || e.ctrlKey);
    }
  }

  function onQueryInput(): void {
    scheduleSearch();
  }

  // Render a hit's content with the matched portion highlighted. Defensive:
  // if match_start === match_end (no match position), fall back to indexOf.
  interface ContentSpan {
    text: string;
    on: boolean;
  }
  function renderContent(hit: SearchHit): ContentSpan[] {
    const c = hit.content;
    let s = hit.match_start;
    let e = hit.match_end;
    if (s === e) {
      const q = search.query.trim().toLowerCase();
      if (q.length > 0) {
        const found = c.toLowerCase().indexOf(q);
        if (found >= 0) {
          s = found;
          e = found + q.length;
        }
      }
    }
    if (s >= e) return [{ text: c, on: false }];
    return [
      { text: c.slice(0, s), on: false },
      { text: c.slice(s, e), on: true },
      { text: c.slice(e), on: false },
    ];
  }

  // Compute global selectedIdx that maps to a position within a group's hits
  // so we can highlight the right row.
  function indexOfHit(targetHit: SearchHit): number {
    return flatHits.indexOf(targetHit);
  }
</script>

<style>
  :global(.search-hit) {
    background: color-mix(in oklch, var(--mw-accent) 25%, transparent);
    color: var(--color-base-content);
    padding: 0 1px;
    border-radius: 2px;
    font-weight: 600;
  }
</style>

{#if search.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Search workspace">
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={closeSearch}
      aria-label="Close"
    ></button>
    <div
      class="modal-box max-w-3xl bg-base-100 border border-base-300 shadow-2xl flex flex-col p-0 max-h-[80vh] overflow-hidden"
    >
      <div class="px-3 py-2 border-b border-base-200 flex items-center gap-2">
        <span
          class="material-symbols-rounded text-[18px] text-base-content/50 shrink-0"
          >search</span
        >
        <input
          bind:this={inputEl}
          type="text"
          class="flex-1 bg-transparent outline-none text-base px-1 py-1"
          placeholder="Search workspace contents…"
          bind:value={search.query}
          oninput={onQueryInput}
          onkeydown={onInputKeydown}
        />
        {#if search.isSearching}
          <span class="loading loading-spinner loading-xs text-base-content/40"
          ></span>
        {/if}
      </div>
      <div class="flex-1 overflow-y-auto py-1">
        {#if search.error}
          <div class="px-4 py-3 text-sm text-error">Error: {search.error}</div>
        {:else if !search.query.trim() || search.query.trim().length < 2}
          <div class="px-4 py-3 text-sm text-base-content/40 italic">
            Type at least 2 characters to search…
          </div>
        {:else if !search.isSearching && flatHits.length === 0}
          <div class="px-4 py-3 text-sm text-base-content/40 italic">
            No matches
          </div>
        {:else}
          <ul class="flex flex-col">
            {#each groups as group (group.path)}
              <li class="mb-1">
                <div
                  class="px-3 py-1 text-[11px] text-base-content/50 font-semibold sticky top-0 bg-base-100"
                >
                  {group.relPath}
                </div>
                <ul>
                  {#each group.hits as hit (hit.path + ":" + hit.line)}
                    {@const idx = indexOfHit(hit)}
                    <li>
                      <button
                        type="button"
                        class="w-full flex items-baseline gap-3 px-3 py-1 text-left text-xs hover:bg-base-200"
                        class:bg-base-200={idx === search.selectedIdx}
                        onmousemove={() => (search.selectedIdx = idx)}
                        onclick={(e) => pick(idx, e.metaKey || e.ctrlKey)}
                      >
                        <span
                          class="text-[10px] text-base-content/40 font-mono shrink-0 w-8 text-right"
                          >{hit.line}</span
                        >
                        <span class="font-mono truncate flex-1 search-line">
                          {#each renderContent(hit) as part}
                            {#if part.on}
                              <mark class="search-hit">{part.text}</mark>
                            {:else}{part.text}{/if}
                          {/each}
                        </span>
                      </button>
                    </li>
                  {/each}
                </ul>
              </li>
            {/each}
          </ul>
          {#if flatHits.length >= 200}
            <div
              class="px-4 py-2 text-[10px] text-base-content/40 italic border-t border-base-200"
            >
              200+ results — refine your search
            </div>
          {/if}
        {/if}
      </div>
      <div
        class="px-3 py-1.5 border-t border-base-200 text-[10px] text-base-content/40 flex gap-3"
      >
        <span>↑↓ select</span>
        <span>Enter open</span>
        <span>⌘Enter new tab</span>
        <span>Esc close</span>
      </div>
    </div>
  </div>
{/if}
