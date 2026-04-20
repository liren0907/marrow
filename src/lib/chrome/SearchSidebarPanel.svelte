<script lang="ts">
  import {
    search,
    scheduleSearch,
    setScope,
    type SearchScope,
  } from "$lib/search/searchState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import type { CrossHit, SearchHit } from "$lib/workspace/tauri";
  import { peek } from "$lib/peek/peekState.svelte";

  function relPath(path: string, root: string): string {
    if (root && path.startsWith(root)) {
      return path.slice(root.length).replace(/^[/\\]/, "");
    }
    return path;
  }

  function open(ch: CrossHit): void {
    const currentRoot = workspace.info?.root ?? "";
    if (ch.workspace_root !== currentRoot) {
      void peek.push(ch.hit.path, null, {
        root: ch.workspace_root,
        name: ch.workspace_name,
      });
      return;
    }
    workspace.replaceCurrentTab(ch.hit.path);
  }

  function onInput(e: Event): void {
    search.query = (e.currentTarget as HTMLInputElement).value;
    scheduleSearch();
  }

  function pickScope(s: SearchScope): void {
    setScope(s);
  }

  interface ContentSpan {
    text: string;
    on: boolean;
  }

  function renderHit(hit: SearchHit): ContentSpan[] {
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
</script>

<div class="panel">
  <div class="panel-header mw-meta">Search</div>
  <div class="search-input-wrap">
    <input
      type="text"
      class="search-input"
      placeholder={search.scope === "all"
        ? "Search all workspaces…"
        : "Search all notes…"}
      value={search.query}
      oninput={onInput}
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
  </div>
  <div class="search-meta mw-meta">
    {#if search.isSearching}
      Searching…
    {:else if search.error}
      {search.error}
    {:else}
      {search.results.length} result{search.results.length === 1 ? "" : "s"}
    {/if}
  </div>
  <div class="search-results">
    {#if search.results.length === 0 && !search.isSearching && search.query.trim().length >= 2}
      <p class="panel-empty">No matches</p>
    {:else}
      {#each search.results as ch, i (i + ":" + ch.hit.path + ":" + ch.hit.line)}
        {#if search.scope === "all" && (i === 0 || search.results[i - 1].workspace_id !== ch.workspace_id)}
          <div class="ws-header mw-meta">{ch.workspace_name}</div>
        {/if}
        <button
          type="button"
          class="search-result"
          onclick={() => open(ch)}
        >
          <div class="search-result-title">
            {relPath(ch.hit.path, ch.workspace_root)}
          </div>
          {#if ch.hit.content}
            <div class="search-result-snippet">
              <span class="search-result-line">L{ch.hit.line}:</span>
              {#each renderHit(ch.hit) as part}
                {#if part.on}<mark class="search-hit">{part.text}</mark
                  >{:else}{part.text}{/if}
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }
  .panel-header {
    padding: 10px 14px 8px;
    position: sticky;
    top: 0;
    background: var(--color-base-200);
    z-index: 2;
  }
  .search-input-wrap {
    padding: 4px 12px 6px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .search-input {
    width: 100%;
    background: var(--color-base-300);
    border: 1px solid var(--mw-rule-strong);
    border-radius: 4px;
    padding: 6px 10px;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-base-content);
    outline: none;
  }
  .search-input:focus {
    border-color: var(--mw-accent);
  }
  .scope-toggle {
    display: inline-flex;
    align-self: flex-start;
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
  .search-meta {
    padding: 2px 14px 8px;
  }
  .search-results {
    flex: 1;
    overflow-y: auto;
    padding: 0 6px 20px;
  }
  .ws-header {
    padding: 10px 10px 4px;
  }
  .search-result {
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    cursor: pointer;
    border-radius: 3px;
    margin-bottom: 2px;
    background: transparent;
    border: none;
    color: inherit;
  }
  .search-result:hover {
    background: var(--color-base-300);
  }
  .search-result-title {
    font-size: 12px;
    color: var(--color-base-content);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .search-result-snippet {
    font-size: 11px;
    color: var(--mw-ink-2);
    margin: 3px 0;
    font-family: var(--font-mono);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .panel-empty {
    padding: 20px 14px;
    color: var(--mw-ink-3);
    font-size: 12px;
    font-style: italic;
  }
  .search-result-line {
    color: var(--mw-ink-3);
    margin-right: 3px;
  }
  :global(.search-hit) {
    background: color-mix(in oklch, var(--mw-accent) 25%, transparent);
    color: var(--color-base-content);
    padding: 0 1px;
    border-radius: 2px;
    font-weight: 600;
  }
</style>
