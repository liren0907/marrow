<script lang="ts">
  import type { Tab } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";

  let { tab }: { tab: Tab } = $props();

  const folder = $derived.by(() => {
    const root = workspace.info?.root ?? "";
    const idx = Math.max(tab.path.lastIndexOf("/"), tab.path.lastIndexOf("\\"));
    const dir = idx > 0 ? tab.path.slice(0, idx) : "";
    if (root && dir.startsWith(root)) {
      const rel = dir.slice(root.length).replace(/^[/\\]/, "");
      return rel || "(root)";
    }
    return dir || "(root)";
  });

  const modified = $derived.by(() => {
    const ms = tab.lastKnownMtime;
    if (!ms) return "";
    // mtime is in seconds per Rust fs metadata; guard both scales just in case.
    const d = new Date(ms < 1e12 ? ms * 1000 : ms);
    if (Number.isNaN(d.getTime())) return "";
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  });
</script>

<div class="editor-meta mw-meta">
  <span class="editor-meta-folder">{folder}</span>
  {#if modified}
    <span class="editor-meta-sep">·</span>
    <span class="editor-meta-date">Modified {modified}</span>
  {/if}
</div>

<style>
  .editor-meta {
    max-width: 46rem;
    margin: 0 auto;
    padding: 20px 48px 0 48px;
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-shrink: 0;
  }
  .editor-meta-folder {
    color: var(--mw-ink-2);
  }
  .editor-meta-sep {
    color: var(--mw-ink-3);
  }
  .editor-meta-date {
    color: var(--mw-ink-3);
  }
  @media (max-width: 720px) {
    .editor-meta {
      padding: 16px 24px 0 24px;
    }
  }
</style>
