<script lang="ts">
  import { slide } from "svelte/transition";
  import type { DirEntry } from "$lib/workspace/types";
  import { tree } from "./treeState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { classifyFile, isConvertible } from "$lib/workspace/fileKind";
  import Self from "./FileTreeNode.svelte";
  import {
    openContextMenu,
    type ContextMenuItem,
  } from "$lib/components/ui/contextMenuState.svelte";
  import { openNamePrompt } from "./namePromptState.svelte";
  import {
    newFile,
    newFolder,
    deleteEntry,
    renameFolder,
  } from "./treeOps";
  import { startFileRename } from "./renameModalState.svelte";
  import { openFileHistory } from "$lib/history/fileHistoryState.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";

  let {
    entry,
    depth,
    isLast = false,
    ancestorLast = [],
  }: {
    entry: DirEntry;
    depth: number;
    isLast?: boolean;
    ancestorLast?: boolean[];
  } = $props();

  function iconFor(e: DirEntry): IconName {
    if (e.is_dir) return tree.isExpanded(e.path) ? "folder-open" : "folder";
    const kind = classifyFile(e.path);
    switch (kind) {
      case "markdown":
        return "file-text";
      case "image":
        return "image";
      case "video":
        return "film";
      case "audio":
        return "music";
      case "pdf":
        return "file-text";
      case "text":
        return "file-code";
      default:
        return "file";
    }
  }

  function colorFor(e: DirEntry): string {
    if (e.is_dir) {
      return tree.isExpanded(e.path)
        ? "color-mix(in oklch, var(--mw-accent) 70%, var(--mw-ink-2))"
        : "var(--mw-ink-1)";
    }
    const kind = classifyFile(e.path);
    switch (kind) {
      case "markdown":
        return "color-mix(in oklch, var(--mw-accent) 75%, var(--mw-ink-2))";
      case "image":
        return "color-mix(in oklch, oklch(0.62 0.15 300) 60%, var(--mw-ink-2))";
      case "video":
        return "color-mix(in oklch, oklch(0.62 0.18 25) 60%, var(--mw-ink-2))";
      case "audio":
        return "color-mix(in oklch, oklch(0.62 0.15 150) 60%, var(--mw-ink-2))";
      case "pdf":
        return "color-mix(in oklch, oklch(0.62 0.18 45) 60%, var(--mw-ink-2))";
      case "text":
        return "var(--mw-ink-2)";
      default:
        return "var(--mw-ink-3)";
    }
  }

  function handleClick() {
    if (entry.is_dir) {
      void tree.toggle(entry.path);
    } else {
      workspace.openFile(entry.path);
    }
  }

  function handleContextMenu(e: MouseEvent) {
    const items: ContextMenuItem[] = [];
    if (entry.is_dir) {
      items.push(
        {
          label: "New file",
          icon: "file-plus",
          onclick: () =>
            openNamePrompt({
              title: `New file in ${entry.name}`,
              placeholder: "filename.md",
              confirmLabel: "Create",
              onConfirm: (name) => newFile(entry.path, name),
            }),
        },
        {
          label: "New folder",
          icon: "folder-plus",
          onclick: () =>
            openNamePrompt({
              title: `New folder in ${entry.name}`,
              placeholder: "folder name",
              confirmLabel: "Create",
              onConfirm: (name) => newFolder(entry.path, name),
            }),
        },
        { label: "", divider: true },
        {
          label: "Rename folder",
          icon: "pencil",
          onclick: () =>
            openNamePrompt({
              title: `Rename ${entry.name}`,
              initial: entry.name,
              confirmLabel: "Rename",
              onConfirm: (name) => renameFolder(entry.path, name),
            }),
        },
        {
          label: "Delete folder",
          icon: "trash-2",
          danger: true,
          onclick: () => deleteEntry(entry.path, true),
        },
      );
    } else {
      const isMd = classifyFile(entry.path) === "markdown";
      items.push(
        {
          label: "Open",
          icon: "external-link",
          onclick: () => workspace.openFile(entry.path),
        },
        {
          label: "Open in other pane",
          icon: "columns-2",
          onclick: () => workspace.openInOtherPane(entry.path),
        },
      );
      if (isMd) {
        items.push(
          { label: "", divider: true },
          {
            label: "Rename",
            icon: "pencil",
            onclick: () => startFileRename(entry.path),
          },
          {
            label: "View history",
            icon: "history",
            onclick: () => openFileHistory(entry.path),
          },
        );
      } else if (isConvertible(entry.path)) {
        items.push(
          { label: "", divider: true },
          {
            label: "Convert to Markdown",
            icon: "file-code",
            onclick: () => workspace.openConvert(entry.path),
          },
        );
      }
      items.push(
        { label: "", divider: true },
        {
          label: "Delete",
          icon: "trash-2",
          danger: true,
          onclick: () => deleteEntry(entry.path, false),
        },
      );
    }
    openContextMenu(e, items);
  }

  function handleHoverAction(e: MouseEvent) {
    e.stopPropagation();
    if (entry.is_dir) {
      const root = entry.path;
      const items: ContextMenuItem[] = [
        {
          label: "New file",
          icon: "file-plus",
          onclick: () =>
            openNamePrompt({
              title: `New file in ${entry.name}`,
              placeholder: "filename.md",
              confirmLabel: "Create",
              onConfirm: (name) => newFile(root, name),
            }),
        },
        {
          label: "New folder",
          icon: "folder-plus",
          onclick: () =>
            openNamePrompt({
              title: `New folder in ${entry.name}`,
              placeholder: "folder name",
              confirmLabel: "Create",
              onConfirm: (name) => newFolder(root, name),
            }),
        },
      ];
      openContextMenu(e, items);
    } else {
      workspace.openInOtherPane(entry.path);
    }
  }

  const isActive = $derived(
    !entry.is_dir &&
      workspace.activePane.tabs.find(
        (t) => t.id === workspace.activePane.activeTabId,
      )?.path === entry.path,
  );

  const folderCount = $derived.by(() => {
    if (!entry.is_dir || !tree.isExpanded(entry.path)) return null;
    const kids = tree.getChildren(entry.path);
    return kids ? kids.length : null;
  });
</script>

<div class="tree-row" class:active={isActive}>
  <button
    class="row-btn"
    data-marrow-tree-path={entry.path}
    onclick={handleClick}
    oncontextmenu={handleContextMenu}
  >
    {#each { length: depth } as _, i (i)}
      <span
        class="indent-rail"
        class:rail-empty={i < depth - 1 && ancestorLast[i]}
        class:rail-branch={i === depth - 1}
        class:rail-leaf={i === depth - 1 && isLast}
        aria-hidden="true"
      ></span>
    {/each}
    <span class="chev-slot" aria-hidden="true">
      {#if entry.is_dir}
        <span
          class="chev-icon"
          style:transform="rotate({tree.isExpanded(entry.path) ? 90 : 0}deg)"
        >
          <Icon name="chevron-right" size={14} />
        </span>
      {/if}
    </span>
    <span class="icon" style:color={colorFor(entry)}>
      <Icon name={iconFor(entry)} size={16} />
    </span>
    <span class="name truncate text-xs flex-1">{entry.name}</span>
    {#if folderCount !== null}
      <span class="folder-count">{folderCount}</span>
    {/if}
  </button>
  <button
    class="hover-action"
    onclick={handleHoverAction}
    tabindex="-1"
    aria-label={entry.is_dir ? "New in folder" : "Open in other pane"}
    title={entry.is_dir ? "New in folder" : "Open in other pane"}
  >
    <Icon name={entry.is_dir ? "plus" : "columns-2"} size={14} />
  </button>
</div>

{#if entry.is_dir && tree.isExpanded(entry.path)}
  {@const kids = tree.getChildren(entry.path)}
  {#if kids}
    <div transition:slide={{ duration: 120 }}>
      {#each kids as child, i (child.path)}
        <Self
          entry={child}
          depth={depth + 1}
          isLast={i === kids.length - 1}
          ancestorLast={[...ancestorLast, isLast]}
        />
      {/each}
    </div>
  {:else}
    <div
      class="text-[10px] text-base-content/30 italic py-0.5"
      style:padding-left="{6 + (depth + 1) * 14 + 14 + 16 + 6}px"
    >
      loading…
    </div>
  {/if}
{/if}

<style>
  .tree-row {
    position: relative;
    height: var(--mw-row-h, 22px);
  }
  .row-btn {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    padding-left: 6px;
    padding-right: 6px;
    text-align: left;
    background: transparent;
    border-radius: var(--mw-radius-xs, 2px);
    transition: background-color 80ms ease;
    position: relative;
  }
  .row-btn:hover {
    background: color-mix(in oklch, var(--mw-accent) 6%, transparent);
  }
  .tree-row.active .row-btn {
    background: color-mix(in oklch, var(--mw-accent) 10%, transparent);
  }
  .tree-row.active .row-btn::before {
    content: "";
    position: absolute;
    left: 0;
    top: 3px;
    bottom: 3px;
    width: 2px;
    background: var(--mw-accent);
    border-radius: 1px;
  }
  .indent-rail {
    position: relative;
    width: 14px;
    height: 100%;
    flex-shrink: 0;
  }
  .indent-rail::before {
    content: "";
    position: absolute;
    left: 7px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--mw-rule);
  }
  .indent-rail.rail-empty::before {
    display: none;
  }
  .indent-rail.rail-branch::after {
    content: "";
    position: absolute;
    left: 7px;
    right: 0;
    top: 50%;
    height: 1px;
    background: var(--mw-rule);
  }
  .indent-rail.rail-leaf::before {
    bottom: 50%;
  }
  .row-btn:hover .indent-rail::before,
  .row-btn:hover .indent-rail.rail-branch::after {
    background: var(--mw-rule-strong);
  }
  .chev-slot {
    width: 14px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--mw-ink-3);
    margin-right: 4px;
  }
  .icon {
    margin-right: 6px;
  }
  .name {
    min-width: 0;
  }
  .chev-icon {
    transition: transform 120ms ease;
  }
  .folder-count {
    font-size: 10px;
    color: var(--mw-ink-3);
    margin-left: auto;
    padding-left: 6px;
    flex-shrink: 0;
    transition: opacity 80ms ease;
  }
  .tree-row:hover .folder-count {
    opacity: 0;
  }
  .hover-action {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--mw-radius-xs, 2px);
    color: var(--mw-ink-2);
    background: transparent;
    opacity: 0;
    pointer-events: none;
    transition: opacity 80ms ease, background-color 80ms ease;
  }
  .tree-row:hover .hover-action {
    opacity: 1;
    pointer-events: auto;
  }
  .hover-action:hover {
    background: color-mix(in oklch, var(--mw-accent) 18%, transparent);
    color: var(--mw-ink-1);
  }

  :global(button[data-marrow-tree-path].reveal-flash) {
    animation: reveal-flash 800ms ease-out;
  }
  @keyframes reveal-flash {
    0% {
      background-color: color-mix(in oklch, var(--color-primary) 35%, transparent);
    }
    100% {
      background-color: transparent;
    }
  }
</style>
