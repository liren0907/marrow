<script lang="ts">
  import type { DirEntry } from "$lib/workspace/types";
  import { tree } from "./treeState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { classifyFile } from "$lib/workspace/fileKind";
  import Self from "./FileTreeNode.svelte";

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
  onclick={handleClick}
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
