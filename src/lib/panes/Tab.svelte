<script lang="ts">
  import type { Tab as TabType } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    openContextMenu,
    type ContextMenuItem,
  } from "$lib/components/ui/contextMenuState.svelte";
  import { openFileHistory } from "$lib/history/fileHistoryState.svelte";

  let {
    tab,
    paneId,
    active,
  }: { tab: TabType; paneId: string; active: boolean } = $props();

  function activate() {
    workspace.setActiveTab(paneId, tab.id);
  }

  function close(e: MouseEvent) {
    e.stopPropagation();
    workspace.closeTab(paneId, tab.id);
  }

  function onDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData(
      "application/x-marrow-tab",
      JSON.stringify({ srcPaneId: paneId, tabId: tab.id }),
    );
  }

  function onContextMenu(e: MouseEvent) {
    const isVirtual = tab.path.startsWith("marrow://");
    const isMarkdown = /\.(md|markdown|mdx)$/i.test(tab.path);
    const items: ContextMenuItem[] = [];
    if (!isVirtual && isMarkdown) {
      items.push({
        label: "View history",
        icon: "history",
        onclick: () => openFileHistory(tab.path),
      });
      items.push({ label: "", divider: true });
    }
    items.push({
      label: "Close tab",
      icon: "close",
      onclick: () => workspace.closeTab(paneId, tab.id),
    });
    openContextMenu(e, items);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="flex items-center gap-1.5 px-3 border-r border-base-200 cursor-pointer text-xs whitespace-nowrap select-none
    {active
    ? 'bg-base-200 text-base-content'
    : 'text-base-content/60 hover:bg-base-200/50'}"
  onclick={activate}
  oncontextmenu={onContextMenu}
  role="tab"
  tabindex="0"
  aria-selected={active}
  draggable="true"
  ondragstart={onDragStart}
  data-tab-id={tab.id}
  data-pane-id={paneId}
>
  <span class="truncate max-w-[180px]">{tab.title}</span>
  {#if tab.isDirty}
    <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
  {/if}
  <button
    class="btn btn-ghost btn-xs btn-circle h-5 min-h-0 w-5 p-0 shrink-0"
    onclick={close}
    aria-label="Close tab"
  >
    <span class="material-symbols-rounded text-[14px]">close</span>
  </button>
</div>
