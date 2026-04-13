<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { tree } from "./treeState.svelte";
  import FileTreeNode from "./FileTreeNode.svelte";

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
</script>

{#if workspace.info}
  {@const root = workspace.info.root}
  {@const entries = tree.getChildren(root)}
  <div class="flex flex-col gap-0.5 py-1 overflow-y-auto">
    {#if entries}
      {#if entries.length === 0}
        <div class="px-3 py-2 text-base-content/40 text-xs italic">
          Empty folder
        </div>
      {:else}
        {#each entries as entry (entry.path)}
          <FileTreeNode {entry} depth={0} />
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
