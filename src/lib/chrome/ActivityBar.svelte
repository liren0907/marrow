<script lang="ts">
  import {
    activityBar,
    setActivity,
    type Activity,
  } from "./activityBarState.svelte";
  import { toggleCommandPalette } from "$lib/command/commandPaletteState.svelte";
  import { toggleSettings } from "$lib/settings/settingsModalState.svelte";
  import { uiSettings, toggleSidebar } from "$lib/settings/uiSettings.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { newExcalidrawCanvas } from "$lib/tree/treeOps";
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

  // Clicking a panel while the Sidebar is collapsed re-opens it — so a
  // collapsed Sidebar is never a dead end. A panel icon only ever shows a
  // panel; collapsing is the dedicated toggle button's job.
  function onActivityClick(id: Activity) {
    setActivity(id);
    if (!uiSettings.showSidebar) toggleSidebar();
  }

  // Distill is an activity whose sidebar panel is the DistillExplorer (a file
  // picker). Clicking the flask swaps the sidebar to it AND lays out the panes
  // (editor | Distill panel) so you land in the full three-zone Distill view.
  function onDistillClick() {
    setActivity("distill");
    if (!uiSettings.showSidebar) toggleSidebar();
    workspace.openDistillView();
  }
</script>

<nav class="activity-bar" aria-label="Activity">
  <div class="activity-group">
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      onclick={toggleSidebar}
      data-tip="Toggle sidebar · ⇧⌘B"
      aria-label="Toggle sidebar"
    >
      <Icon
        name={uiSettings.showSidebar ? "panel-left-close" : "panel-left-open"}
        size={20}
      />
    </button>
    <div class="activity-divider" aria-hidden="true"></div>
    {#each items as item (item.id)}
      <button
        type="button"
        class="activity-btn tooltip tooltip-right"
        class:active={activityBar.current === item.id}
        onclick={() => onActivityClick(item.id)}
        data-tip={item.label}
        aria-label={item.label}
        aria-pressed={activityBar.current === item.id}
      >
        <Icon name={item.icon} size={20} />
      </button>
    {/each}
  </div>
  <div class="activity-group">
    <!-- Distill is an activity (swaps the sidebar to the DistillExplorer, which
         lets you pick a note to distill). The divider sets it apart from the
         one-shot launchers below so its persistent active state reads as
         intentional. -->
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      class:active={activityBar.current === "distill"}
      onclick={onDistillClick}
      data-tip="Distill"
      aria-label="Distill"
      aria-pressed={activityBar.current === "distill"}
    >
      <Icon name="flask-conical" size={20} />
    </button>
    <div class="activity-divider" aria-hidden="true"></div>
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      onclick={() => newExcalidrawCanvas()}
      data-tip="New canvas"
      aria-label="New Excalidraw canvas"
    >
      <Icon name="brush" size={20} />
    </button>
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
    <button
      type="button"
      class="activity-btn tooltip tooltip-right"
      onclick={toggleSettings}
      data-tip="Settings · ⇧⌘,"
      aria-label="Settings"
    >
      <Icon name="settings" size={20} />
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
  .activity-divider {
    width: 18px;
    height: 1px;
    background: var(--mw-rule);
    margin: 4px 0;
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
    box-shadow: inset 2px 0 0 var(--mw-accent);
  }
</style>
