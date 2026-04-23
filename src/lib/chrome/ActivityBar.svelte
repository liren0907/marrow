<script lang="ts">
  import {
    activityBar,
    setActivity,
    type Activity,
  } from "./activityBarState.svelte";
  import { toggleCommandPalette } from "$lib/command/commandPaletteState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";

  interface Item {
    id: Activity;
    icon: IconName;
    label: string;
  }

  const items: Item[] = [
    { id: "files", icon: "folder", label: "Files" },
    { id: "search", icon: "search", label: "Search" },
    { id: "tags", icon: "tag", label: "Tags" },
    { id: "graph", icon: "network", label: "Graph" },
    { id: "backlinks", icon: "arrow-left", label: "Backlinks" },
  ];
</script>

<nav class="activity-bar" aria-label="Activity">
  <div class="activity-group">
    {#each items as item (item.id)}
      <button
        type="button"
        class="activity-btn tooltip tooltip-right"
        class:active={activityBar.current === item.id}
        onclick={() => setActivity(item.id)}
        data-tip={item.label}
        aria-label={item.label}
        aria-pressed={activityBar.current === item.id}
      >
        <Icon name={item.icon} size={20} />
      </button>
    {/each}
  </div>
  <div class="activity-group">
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      onclick={() => workspace.openConvertView()}
      data-tip="Convert to Markdown"
      aria-label="Convert to Markdown"
    >
      <Icon name="file-code" size={20} />
    </button>
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      onclick={toggleCommandPalette}
      data-tip="Command palette · ⇧⌘P"
      aria-label="Command palette"
    >
      <Icon name="terminal" size={20} />
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
</style>
