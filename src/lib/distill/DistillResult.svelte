<script lang="ts">
  // The Distill result panel — a PURE presentational view of an extraction
  // result (the right half of DistillTab, minus the header). Given a status
  // (+ annotations / error message) it renders the term list or one of the
  // empty / loading / error / no-terms states. No store, no localStorage, no
  // backend calls: DistillTab owns all of that and feeds this via props — which
  // also makes every state trivially showable in the dev-only /gallery.
  import type { Annotation } from "$lib/workspace/tauri";
  import Badge from "$lib/components/ui/Badge.svelte";

  let {
    status,
    annotations = [],
    errorMessage = "",
  }: {
    status: "empty" | "loading" | "ready" | "error";
    annotations?: Annotation[];
    errorMessage?: string;
  } = $props();
</script>

<div class="distill-body">
  {#if status === "empty"}
    <p class="distill-hint">在左側選一個資料夾，再挑一篇 <code>.md</code> 來提煉它的名詞。</p>
  {:else if status === "loading"}
    <p class="distill-hint">提煉中…</p>
  {:else if status === "error"}
    <p class="distill-hint distill-error">提煉失敗：{errorMessage}</p>
  {:else if annotations.length === 0}
    <p class="distill-hint">沒有抽到名詞。</p>
  {:else}
    <ul class="distill-list">
      {#each annotations as a (a.text)}
        <li class="distill-row">
          <span class="distill-term">{a.text}</span>
          <span class="distill-meta">
            <Badge variant="ghost" size="xs">{a.pos}</Badge>
            <span class="distill-count">{a.count}</span>
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .distill-body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 8px;
    min-height: 0;
  }
  .distill-hint {
    color: var(--mw-ink-3);
    font-size: var(--text-xs);
    padding: 12px 8px;
    line-height: 1.5;
  }
  .distill-error {
    color: var(--color-error);
  }
  .distill-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .distill-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 8px;
    border-radius: var(--mw-radius-xs);
  }
  .distill-row:hover {
    background: color-mix(in oklch, var(--mw-accent) 6%, transparent);
  }
  .distill-term {
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .distill-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
  .distill-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--mw-ink-2);
    min-width: 1.5em;
    text-align: right;
  }
</style>
