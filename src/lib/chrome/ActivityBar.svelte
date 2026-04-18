<script lang="ts">
  import {
    activityBar,
    setActivity,
    type Activity,
  } from "./activityBarState.svelte";
  import { quickOpen } from "$lib/quickopen/quickOpenState.svelte";
  import { toggleCommandPalette } from "$lib/command/commandPaletteState.svelte";

  interface Item {
    id: Activity;
    icon: string;
    label: string;
  }

  const items: Item[] = [
    { id: "files", icon: "folder", label: "Files" },
    { id: "search", icon: "search", label: "Search" },
    { id: "tags", icon: "sell", label: "Tags" },
    { id: "graph", icon: "hub", label: "Graph" },
    { id: "backlinks", icon: "arrow_back", label: "Backlinks" },
  ];

  function openQuick(): void {
    quickOpen.isOpen = true;
    quickOpen.query = "";
    quickOpen.selectedIdx = 0;
  }
</script>

<nav class="activity-bar" aria-label="Activity">
  <div class="activity-group">
    {#each items as item (item.id)}
      <button
        type="button"
        class="activity-btn"
        class:active={activityBar.current === item.id}
        onclick={() => setActivity(item.id)}
        title={item.label}
        aria-label={item.label}
        aria-pressed={activityBar.current === item.id}
      >
        <span class="material-symbols-rounded">{item.icon}</span>
      </button>
    {/each}
  </div>
  <div class="activity-group">
    <button
      type="button"
      class="activity-btn"
      onclick={openQuick}
      title="Quick open (⌘P)"
      aria-label="Quick open"
    >
      <span class="material-symbols-rounded">subject</span>
    </button>
    <button
      type="button"
      class="activity-btn"
      onclick={toggleCommandPalette}
      title="Command palette (⇧⌘P)"
      aria-label="Command palette"
    >
      <span class="material-symbols-rounded">terminal</span>
    </button>
  </div>
</nav>

<style>
  .activity-bar {
    width: var(--mw-activitybar-w);
    background: var(--color-base-200);
    border-right: 1px solid var(--mw-rule);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    height: 100%;
    flex-shrink: 0;
  }
  .activity-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: center;
  }
  .activity-btn {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    color: var(--mw-ink-2);
    border-radius: 4px;
    position: relative;
    transition: color 0.1s, background 0.1s;
    cursor: pointer;
  }
  .activity-btn:hover {
    color: var(--color-base-content);
    background: var(--color-base-300);
  }
  .activity-btn.active {
    color: var(--color-base-content);
  }
  .activity-btn.active::before {
    content: "";
    position: absolute;
    left: -6px;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: var(--mw-accent);
    border-radius: 2px;
  }
  .activity-btn :global(.material-symbols-rounded) {
    font-size: 20px;
  }
</style>
