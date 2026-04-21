<script lang="ts">
  import type { Tab as TabType } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    openContextMenu,
    type ContextMenuItem,
  } from "$lib/components/ui/contextMenuState.svelte";
  import { openFileHistory } from "$lib/history/fileHistoryState.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  let {
    tab,
    paneId,
    active,
  }: { tab: TabType; paneId: string; active: boolean } = $props();

  const kindLetter = $derived.by(() => {
    switch (tab.kind) {
      case "markdown":
        return "M";
      case "graph":
        return "◉";
      case "image":
        return "I";
      case "video":
        return "V";
      case "audio":
        return "A";
      case "pdf":
        return "P";
      case "text":
        return "T";
      default:
        return "·";
    }
  });

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
      icon: "x",
      onclick: () => workspace.closeTab(paneId, tab.id),
    });
    openContextMenu(e, items);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="mw-tab"
  class:active
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
  <span class="mw-tab-kind" data-kind={tab.kind}>{kindLetter}</span>
  <span class="mw-tab-title">{tab.title}</span>
  {#if tab.isDirty}
    <span class="mw-tab-dirty" aria-label="Modified"></span>
  {/if}
  <button
    class="mw-tab-close"
    onclick={close}
    aria-label="Close tab"
  >
    <Icon name="x" size={14} />
  </button>
</div>

<style>
  .mw-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 12px;
    font-size: 12px;
    color: var(--mw-ink-2);
    cursor: pointer;
    border-right: 1px solid var(--mw-rule);
    background: var(--mw-tab-inactive);
    max-width: 220px;
    min-width: 0;
    position: relative;
    user-select: none;
    white-space: nowrap;
  }
  .mw-tab:hover {
    color: var(--mw-ink-1);
  }
  .mw-tab.active {
    background: var(--color-base-100);
    color: var(--color-base-content);
  }
  .mw-tab.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 1px;
    background: var(--mw-accent);
  }
  .mw-tab-kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--mw-ink-3);
    width: 12px;
    text-align: center;
    flex-shrink: 0;
  }
  .mw-tab-kind[data-kind="graph"] {
    color: var(--mw-accent);
  }
  .mw-tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .mw-tab-dirty {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--mw-accent);
    flex-shrink: 0;
  }
  .mw-tab-close {
    color: var(--mw-ink-3);
    padding: 0 4px;
    border-radius: 2px;
    font-size: 14px;
    line-height: 1;
    background: transparent;
    border: none;
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .mw-tab-close:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
</style>
