<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinksFor } from "$lib/workspace/backlinkIndex.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

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
      {#each entries as entry (entry.sourcePath + ":" + entry.matchIndex)}
        <li>
          <button
            type="button"
            class="sidebar-row sidebar-row--multiline"
            onclick={() => open(entry.sourcePath)}
          >
            <!-- 首行對齊：第一行（檔名）佔一個 row-h 高度並垂直置中，
                 跟其他單行 sidebar row 的視覺基準對齊。context 行自然往下排。 -->
            <span class="flex items-center gap-2 min-h-[var(--mw-row-h)]">
              <Icon name="file-text" size={16} class="text-base-content/40" />
              <span class="truncate">{relPath(entry.sourcePath)}</span>
            </span>
            {#if entry.context}
              <span
                class="block pl-6 text-xs text-base-content/50 truncate font-mono"
                >{entry.context}</span
              >
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
