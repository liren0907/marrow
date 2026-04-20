<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import type { CrossHit, SearchHit } from "$lib/workspace/tauri";
  import { peek } from "$lib/peek/peekState.svelte";
  import {
    search,
    scheduleSearch,
    closeSearch,
    setScope,
    type SearchScope,
  } from "./searchState.svelte";

  let inputEl: HTMLInputElement | undefined = $state();

  // Nested grouping: workspace → file → hits. When scope=current there is
  // exactly one workspace group and we hide its header.
  interface FileGroup {
    path: string;
    relPath: string;
    hits: CrossHit[];
  }
  interface WsGroup {
    id: string;
    name: string;
    files: FileGroup[];
  }

  const groups = $derived.by(() => {
    const out: WsGroup[] = [];
    for (const ch of search.results) {
      let ws = out[out.length - 1];
      if (!ws || ws.id !== ch.workspace_id) {
        ws = { id: ch.workspace_id, name: ch.workspace_name, files: [] };
        out.push(ws);
      }
      const last = ws.files[ws.files.length - 1];
      const root = ch.workspace_root;
      const rel =
        root && ch.hit.path.startsWith(root)
          ? ch.hit.path.slice(root.length).replace(/^[/\\]/, "")
          : ch.hit.path;
      if (last && last.path === ch.hit.path) {
        last.hits.push(ch);
      } else {
        ws.files.push({ path: ch.hit.path, relPath: rel, hits: [ch] });
      }
    }
    return out;
  });

  const flatHits = $derived(search.results);

  $effect(() => {
    if (search.isOpen) {
      queueMicrotask(() => inputEl?.focus());
      if (search.selectedIdx >= flatHits.length) search.selectedIdx = 0;
    }
  });

  function pick(idx: number, newTab: boolean): void {
    const ch = flatHits[idx];
    if (!ch) return;
    const currentRoot = workspace.info?.root ?? "";
    const sameWorkspace = ch.workspace_root === currentRoot;
    if (!sameWorkspace) {
      void peek.push(ch.hit.path, null, {
        root: ch.workspace_root,
        name: ch.workspace_name,
      });
      closeSearch();
      return;
    }
    if (newTab) {
      workspace.openFile(ch.hit.path);
    } else {
      workspace.replaceCurrentTab(ch.hit.path);
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

  function indexOfHit(target: CrossHit): number {
    return flatHits.indexOf(target);
  }

  function pickScope(s: SearchScope): void {
    setScope(s);
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
  .scope-toggle {
    display: inline-flex;
    gap: 0;
    border: 1px solid var(--mw-rule);
    border-radius: 4px;
    overflow: hidden;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--mw-ink-2);
  }
  .scope-btn {
    padding: 2px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
  }
  .scope-btn:hover {
    background: var(--color-base-300);
  }
  .scope-btn.active {
    background: var(--mw-accent);
    color: var(--color-base-100);
  }
</style>

{#if search.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Search">
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
          placeholder={search.scope === "all"
            ? "Search all workspaces…"
            : "Search workspace contents…"}
          bind:value={search.query}
          oninput={onQueryInput}
          onkeydown={onInputKeydown}
        />
        <div class="scope-toggle">
          <button
            type="button"
            class="scope-btn"
            class:active={search.scope === "current"}
            onclick={() => pickScope("current")}>This ws</button
          >
          <button
            type="button"
            class="scope-btn"
            class:active={search.scope === "all"}
            onclick={() => pickScope("all")}>All ws</button
          >
        </div>
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
            {#each groups as ws (ws.id)}
              <li class="mb-2">
                {#if search.scope === "all"}
                  <div
                    class="mw-meta px-3 pt-2 pb-1 sticky top-0 bg-base-100"
                  >
                    {ws.name}
                  </div>
                {/if}
                {#each ws.files as file (file.path)}
                  <div class="mb-1">
                    <div
                      class="px-3 py-1 text-[11px] text-base-content/50 font-semibold"
                    >
                      {file.relPath}
                    </div>
                    <ul>
                      {#each file.hits as ch (ch.hit.path + ":" + ch.hit.line)}
                        {@const idx = indexOfHit(ch)}
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
                              >{ch.hit.line}</span
                            >
                            <span class="font-mono truncate flex-1 search-line">
                              {#each renderContent(ch.hit) as part}
                                {#if part.on}
                                  <mark class="search-hit">{part.text}</mark>
                                {:else}{part.text}{/if}
                              {/each}
                            </span>
                          </button>
                        </li>
                      {/each}
                    </ul>
                  </div>
                {/each}
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
