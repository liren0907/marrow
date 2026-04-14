<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    outlines,
    tabScrollRegistry,
  } from "$lib/workspace/tabRegistry.svelte";

  const activeTab = $derived.by(() => {
    const pane = workspace.activePane;
    if (!pane.activeTabId) return null;
    return pane.tabs.find((t) => t.id === pane.activeTabId) ?? null;
  });

  const headings = $derived.by(() => {
    if (!activeTab || activeTab.kind !== "markdown") return [];
    return outlines.byTab.get(activeTab.id) ?? [];
  });

  function jumpTo(pos: number) {
    if (!activeTab) return;
    const fn = tabScrollRegistry.get(activeTab.id);
    fn?.(pos);
  }
</script>

<div class="flex-1 overflow-y-auto p-2">
  {#if !activeTab}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      Open a file to see its outline
    </p>
  {:else if activeTab.kind !== "markdown"}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      Outline is only tracked for markdown files
    </p>
  {:else if headings.length === 0}
    <p class="px-3 py-2 text-xs text-base-content/40 italic">
      No headings in {activeTab.title}
    </p>
  {:else}
    <ul class="flex flex-col gap-0.5">
      {#each headings as h, i (i + ":" + h.pos)}
        <li>
          <button
            type="button"
            class="w-full flex items-baseline gap-2 px-2 py-1 text-left text-sm hover:bg-base-200 rounded"
            style:padding-left="{0.5 + (h.level - 1) * 0.75}rem"
            onclick={() => jumpTo(h.pos)}
          >
            <span class="font-mono text-[11px] text-base-content/40 shrink-0"
              >{"#".repeat(h.level)}</span
            >
            <span class="truncate">{h.text || "(empty heading)"}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
