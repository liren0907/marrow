<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { formatRelative } from "$lib/utils/formatRelative";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let lastClickAt = 0;

  function onTitlebarMouseDown(e: MouseEvent): void {
    if (e.button !== 0) return;
    const now = Date.now();
    const isDoubleClick = now - lastClickAt < 400;
    lastClickAt = isDoubleClick ? 0 : now;
    const win = getCurrentWindow();
    if (isDoubleClick) {
      void win.toggleMaximize();
    } else {
      void win.startDragging();
    }
  }

  const root = $derived(workspace.info?.root ?? "");

  const dirtyCount = $derived(
    workspace.panes
      .flatMap((p) => p.tabs)
      .filter((t) => t.isDirty).length,
  );

  const anyTabSaved = $derived(
    workspace.panes.some((p) => p.tabs.some((t) => t.lastSavedTs)),
  );

  const latestSavedSeconds = $derived.by(() => {
    const tabs = workspace.panes.flatMap((p) => p.tabs);
    const stamps = tabs
      .map((t) => t.lastSavedTs)
      .filter((s): s is number => typeof s === "number");
    if (stamps.length === 0) return 0;
    return Math.max(...stamps) / 1000;
  });

  // Re-evaluate the "Saved Xm ago" label every 30s by polling a tick counter.
  let tick = $state(0);
  $effect(() => {
    const id = setInterval(() => tick++, 30_000);
    return () => clearInterval(id);
  });

  const saveLabel = $derived.by(() => {
    void tick;
    if (dirtyCount > 0) return `${dirtyCount} unsaved`;
    if (latestSavedSeconds > 0) return `Saved ${formatRelative(latestSavedSeconds)}`;
    if (anyTabSaved) return "All saved";
    return "";
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="titlebar"
  onmousedown={onTitlebarMouseDown}
>
  <div class="titlebar-drag"></div>
  <div class="titlebar-left"></div>
  <div class="titlebar-center">
    {#if root}
      <span class="titlebar-workspace">{root}</span>
      <span class="titlebar-sep">—</span>
    {/if}
    <span class="titlebar-app">marrow</span>
  </div>
  <div class="titlebar-right">
    {#if saveLabel}
      <span class="titlebar-save" class:dirty={dirtyCount > 0}>{saveLabel}</span>
    {/if}
  </div>
</div>

<style>
  .titlebar {
    height: var(--mw-titlebar-h);
    display: grid;
    grid-template-columns: 80px 1fr 220px;
    align-items: center;
    background: var(--color-base-200);
    border-bottom: 1px solid var(--mw-rule);
    font-size: 11.5px;
    color: var(--mw-ink-2);
    user-select: none;
    -webkit-app-region: drag;
    position: relative;
    padding: 0 14px;
  }
  .titlebar-drag {
    position: absolute;
    inset: 0;
    -webkit-app-region: drag;
  }
  .titlebar-left {
    grid-column: 1;
  }
  .titlebar-center {
    grid-column: 2;
    text-align: center;
    letter-spacing: 0.02em;
    position: relative;
    z-index: 1;
    display: flex;
    justify-content: center;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
    -webkit-app-region: drag;
  }
  .titlebar-workspace {
    color: var(--mw-ink-1);
    font-family: var(--font-mono);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 60vw;
  }
  .titlebar-sep {
    color: var(--mw-ink-3);
  }
  .titlebar-app {
    font-family: var(--font-display);
    font-weight: 600;
    color: var(--color-base-content);
    letter-spacing: 0.08em;
    text-transform: lowercase;
  }
  .titlebar-right {
    grid-column: 3;
    justify-self: end;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--mw-ink-2);
    position: relative;
    z-index: 1;
    -webkit-app-region: drag;
  }
  .titlebar-save {
    color: var(--mw-ink-2);
  }
  .titlebar-save.dirty {
    color: var(--mw-accent);
  }
</style>
