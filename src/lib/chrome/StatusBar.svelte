<script lang="ts">
  import { onMount } from "svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinksFor } from "$lib/workspace/backlinkIndex.svelte";
  import { gitState } from "$lib/workspace/gitState.svelte";
  import { readTextFile, getWatcherStatus } from "$lib/workspace/tauri";

  const activeTab = $derived(
    workspace.activePane.tabs.find(
      (t) => t.id === workspace.activePane.activeTabId,
    ),
  );
  const noteCount = $derived(
    workspace.fileIndex.filter((f) => f.kind === "markdown").length,
  );
  const paneCount = $derived(workspace.panes.length);
  const tabCount = $derived(
    workspace.panes.reduce((sum, p) => sum + p.tabs.length, 0),
  );
  const dirtyCount = $derived(
    workspace.panes
      .flatMap((p) => p.tabs)
      .filter((t) => t.isDirty).length,
  );
  const backlinkCount = $derived.by(() => {
    if (!activeTab || activeTab.kind !== "markdown") return 0;
    return backlinksFor(activeTab.path).length;
  });

  let watcherRunning = $state(true);
  onMount(() => {
    let cancelled = false;
    const tick = async () => {
      try {
        const s = await getWatcherStatus();
        if (!cancelled) watcherRunning = s.running;
      } catch {
        // ignore
      }
    };
    void tick();
    const id = setInterval(tick, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  });

  let wordCount = $state(0);
  let wordCountKey = "";
  $effect(() => {
    const tab = activeTab;
    if (!tab || tab.kind !== "markdown") {
      wordCount = 0;
      wordCountKey = "";
      return;
    }
    const key = `${tab.path}:${tab.reloadToken ?? 0}`;
    if (key === wordCountKey) return;
    wordCountKey = key;
    let cancelled = false;
    void (async () => {
      try {
        const res = await readTextFile(tab.path);
        if (cancelled) return;
        const words = res.content
          .replace(/[#*_`>\-\[\]()!]/g, " ")
          .split(/\s+/)
          .filter(Boolean).length;
        wordCount = words;
      } catch {
        if (!cancelled) wordCount = 0;
      }
    })();
    return () => {
      cancelled = true;
    };
  });

  const showNoteMeta = $derived(!!activeTab && activeTab.kind === "markdown");
</script>

<div class="statusbar">
  <div class="statusbar-left">
    <span class="watcher" class:off={!watcherRunning}>
      {watcherRunning ? "●" : "○"} Watcher
    </span>
    <span class="sep">·</span>
    <span>{gitState.branch ?? "no git"}</span>
    <span class="sep">·</span>
    <span>{noteCount} notes</span>
    {#if dirtyCount > 0}
      <span class="sep">·</span>
      <span class="dirty">{dirtyCount} unsaved</span>
    {/if}
  </div>
  <div class="statusbar-right">
    {#if showNoteMeta}
      <span>{wordCount} words</span>
      <span class="sep">·</span>
      <span>{backlinkCount} backlinks</span>
      <span class="sep">·</span>
    {/if}
    <span>{paneCount} pane{paneCount === 1 ? "" : "s"}</span>
    <span class="sep">·</span>
    <span>{tabCount} tabs</span>
    <span class="sep">·</span>
    <span>Markdown</span>
    <span class="sep">·</span>
    <span>UTF-8</span>
  </div>
</div>

<style>
  .statusbar {
    height: var(--mw-statusbar-h);
    background: var(--color-base-200);
    color: var(--mw-ink-2);
    font-family: var(--font-mono);
    font-size: 10.5px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 14px;
    user-select: none;
    border-top: 1px solid var(--mw-rule);
  }
  .statusbar-left,
  .statusbar-right {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .sep {
    color: var(--mw-ink-3);
  }
  .watcher {
    color: var(--mw-accent);
  }
  .watcher.off {
    color: var(--mw-ink-3);
  }
  .dirty {
    color: var(--mw-accent);
  }
</style>
