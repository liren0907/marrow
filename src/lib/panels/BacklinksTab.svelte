<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinksFor } from "$lib/workspace/backlinkIndex.svelte";

  const activeTab = $derived.by(() => {
    const pane = workspace.activePane;
    if (!pane.activeTabId) return null;
    return pane.tabs.find((t) => t.id === pane.activeTabId) ?? null;
  });

  const entries = $derived.by(() => {
    if (!activeTab || activeTab.kind !== "markdown") return [];
    return backlinksFor(activeTab.path);
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
  {#if !activeTab}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      Open a file to see its backlinks
    </p>
  {:else if activeTab.kind !== "markdown"}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      Backlinks are only tracked for markdown files
    </p>
  {:else if entries.length === 0}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      No backlinks to {activeTab.title}
    </p>
  {:else}
    <ul class="flex flex-col gap-0.5">
      {#each entries as entry (entry.sourcePath + entry.target)}
        <li>
          <button
            type="button"
            class="w-full flex items-center gap-2 px-2 py-1 text-left text-sm hover:bg-base-200 rounded"
            onclick={() => open(entry.sourcePath)}
          >
            <span
              class="material-symbols-rounded text-[16px] text-base-content/40 shrink-0"
              >description</span
            >
            <span class="truncate">{relPath(entry.sourcePath)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
