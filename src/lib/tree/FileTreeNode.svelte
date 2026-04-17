<script lang="ts">
  import type { DirEntry } from "$lib/workspace/types";
  import { tree } from "./treeState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { classifyFile } from "$lib/workspace/fileKind";
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

  let { entry, depth }: { entry: DirEntry; depth: number } = $props();

  function iconFor(e: DirEntry): string {
    if (e.is_dir) return tree.isExpanded(e.path) ? "folder_open" : "folder";
    const kind = classifyFile(e.path);
    switch (kind) {
      case "markdown":
        return "description";
      case "image":
        return "image";
      case "video":
        return "movie";
      case "audio":
        return "music_note";
      case "pdf":
        return "picture_as_pdf";
      case "text":
        return "article";
      default:
        return "draft";
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
          icon: "note_add",
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
          icon: "create_new_folder",
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
          icon: "drive_file_rename_outline",
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
          icon: "delete",
          danger: true,
          onclick: () => deleteEntry(entry.path, true),
        },
      );
    } else {
      const isMd = classifyFile(entry.path) === "markdown";
      items.push(
        {
          label: "Open",
          icon: "open_in_new",
          onclick: () => workspace.openFile(entry.path),
        },
        {
          label: "Open in other pane",
          icon: "vertical_split",
          onclick: () => workspace.openInOtherPane(entry.path),
        },
      );
      if (isMd) {
        items.push(
          { label: "", divider: true },
          {
            label: "Rename",
            icon: "drive_file_rename_outline",
            onclick: () => startFileRename(entry.path),
          },
        );
      }
      items.push(
        { label: "", divider: true },
        {
          label: "Delete",
          icon: "delete",
          danger: true,
          onclick: () => deleteEntry(entry.path, false),
        },
      );
    }
    openContextMenu(e, items);
  }

  const isActive = $derived(
    !entry.is_dir &&
      workspace.activePane.tabs.find(
        (t) => t.id === workspace.activePane.activeTabId,
      )?.path === entry.path,
  );
</script>

<button
  class="flex items-center gap-1.5 py-1 w-full text-left rounded transition-colors
    {isActive ? 'bg-primary/10 text-primary' : 'hover:bg-base-200'}"
  style:padding-left="{6 + depth * 14}px"
  style:padding-right="6px"
  data-marrow-tree-path={entry.path}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
>
  {#if entry.is_dir}
    <span class="material-symbols-rounded text-[14px] text-base-content/40 shrink-0">
      {tree.isExpanded(entry.path) ? "expand_more" : "chevron_right"}
    </span>
  {:else}
    <span class="w-[14px] shrink-0"></span>
  {/if}
  <span class="material-symbols-rounded text-[16px] text-base-content/60 shrink-0">
    {iconFor(entry)}
  </span>
  <span class="truncate text-xs">{entry.name}</span>
</button>

{#if entry.is_dir && tree.isExpanded(entry.path)}
  {@const kids = tree.getChildren(entry.path)}
  {#if kids}
    {#each kids as child (child.path)}
      <Self entry={child} depth={depth + 1} />
    {/each}
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
