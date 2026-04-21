<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { tree } from "./treeState.svelte";
  import FileTreeNode from "./FileTreeNode.svelte";
  import {
    openContextMenu,
    type ContextMenuItem,
  } from "$lib/components/ui/contextMenuState.svelte";
  import { openNamePrompt } from "./namePromptState.svelte";
  import { newFile, newFolder } from "./treeOps";

  let lastRoot: string | null = null;
  $effect(() => {
    const root = workspace.info?.root ?? null;
    if (root && root !== lastRoot) {
      lastRoot = root;
      tree.reset();
      void tree.load(root);
    } else if (!root && lastRoot) {
      lastRoot = null;
      tree.reset();
    }
  });

  function handleRootContextMenu(e: MouseEvent) {
    // Only fire when the right-click is on the empty area, not a child node.
    if (e.target !== e.currentTarget) return;
    const root = workspace.info?.root;
    if (!root) return;
    const items: ContextMenuItem[] = [
      {
        label: "New file",
        icon: "file-plus",
        onclick: () =>
          openNamePrompt({
            title: "New file at workspace root",
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
            title: "New folder at workspace root",
            placeholder: "folder name",
            confirmLabel: "Create",
            onConfirm: (name) => newFolder(root, name),
          }),
      },
    ];
    openContextMenu(e, items);
  }
</script>

{#if workspace.info}
  {@const root = workspace.info.root}
  {@const entries = tree.getChildren(root)}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex flex-col py-1 overflow-y-auto min-h-full"
    oncontextmenu={handleRootContextMenu}
  >
    {#if entries}
      {#if entries.length === 0}
        <div class="px-3 py-2 text-base-content/40 text-xs italic">
          Empty folder
        </div>
      {:else}
        {#each entries as entry, i (entry.path)}
          <FileTreeNode
            {entry}
            depth={0}
            isLast={i === entries.length - 1}
            ancestorLast={[]}
          />
        {/each}
      {/if}
    {:else}
      <div class="px-3 py-2 text-base-content/40 text-xs">Loading…</div>
    {/if}
  </div>
{:else}
  <div class="px-3 py-3 text-base-content/40 text-xs italic">
    No workspace open
  </div>
{/if}
