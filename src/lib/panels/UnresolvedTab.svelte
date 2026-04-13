<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    backlinks,
    unresolvedAll,
  } from "$lib/workspace/backlinkIndex.svelte";

  const groups = $derived.by(() => {
    // Touch the reactive map so this re-runs when entries change.
    void backlinks.unresolvedBySource.size;
    return unresolvedAll();
  });

  function relPath(path: string): string {
    const root = workspace.info?.root ?? "";
    if (root && path.startsWith(root)) {
      return path.slice(root.length).replace(/^[/\\]/, "");
    }
    return path;
  }

  function open(path: string) {
    workspace.replaceCurrentTab(path);
  }
</script>

<div class="flex-1 overflow-y-auto p-2">
  {#if groups.length === 0}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      No unresolved links — nice
    </p>
  {:else}
    <ul class="flex flex-col gap-2">
      {#each groups as group (group.source)}
        <li>
          <button
            type="button"
            class="w-full flex items-center gap-2 px-2 py-1 text-left text-xs font-semibold text-base-content/70 hover:bg-base-200 rounded"
            onclick={() => open(group.source)}
          >
            <span
              class="material-symbols-rounded text-[14px] text-base-content/40 shrink-0"
              >description</span
            >
            <span class="truncate">{relPath(group.source)}</span>
          </button>
          <ul class="ml-6 mt-0.5 flex flex-col gap-0.5">
            {#each group.targets as target}
              <li
                class="px-2 py-0.5 text-xs text-base-content/50 font-mono truncate"
              >
                [[{target}]]
              </li>
            {/each}
          </ul>
        </li>
      {/each}
    </ul>
  {/if}
</div>
