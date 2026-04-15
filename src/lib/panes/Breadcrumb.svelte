<script lang="ts">
  import type { Pane } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { revealInTree } from "$lib/tree/treeState.svelte";

  let { pane }: { pane: Pane } = $props();

  const activeTab = $derived(
    pane.tabs.find((t) => t.id === pane.activeTabId) ?? null,
  );

  type Segment =
    | { kind: "folder"; label: string; absPath: string }
    | { kind: "file"; label: string }
    | { kind: "virtual"; label: string };

  const segments = $derived.by<Segment[]>(() => {
    if (!activeTab) return [];
    if (activeTab.path.startsWith("marrow://")) {
      return [{ kind: "virtual", label: activeTab.title }];
    }
    const root = workspace.info?.root ?? "";
    const rel = activeTab.path.startsWith(root)
      ? activeTab.path.slice(root.length).replace(/^[/\\]/, "")
      : activeTab.path;
    const parts = rel.split(/[/\\]/).filter((p) => p.length > 0);
    if (parts.length === 0) return [];

    const sep = root.includes("\\") ? "\\" : "/";
    const out: Segment[] = [];
    let cursor = root;
    for (let i = 0; i < parts.length; i++) {
      const isLast = i === parts.length - 1;
      cursor = cursor ? `${cursor}${sep}${parts[i]}` : parts[i];
      if (isLast) {
        out.push({ kind: "file", label: parts[i] });
      } else {
        out.push({ kind: "folder", label: parts[i], absPath: cursor });
      }
    }
    return out;
  });

  function handleFolderClick(absPath: string): void {
    void revealInTree(absPath);
  }
</script>

{#if segments.length > 0}
  <nav
    class="flex items-center gap-0.5 px-3 h-6 shrink-0 border-b border-base-200 overflow-x-auto text-[11px] text-base-content/60"
    aria-label="Editor breadcrumb"
  >
    {#each segments as seg, i (i)}
      {#if i > 0}
        <span class="text-base-content/30 px-0.5 select-none">›</span>
      {/if}
      {#if seg.kind === "folder"}
        <button
          type="button"
          class="px-1 py-0.5 rounded hover:bg-base-200 whitespace-nowrap"
          onclick={() => handleFolderClick(seg.absPath)}
          title="Reveal in file tree"
        >
          {seg.label}
        </button>
      {:else if seg.kind === "file"}
        <span
          class="px-1 py-0.5 font-medium text-base-content/90 whitespace-nowrap"
        >
          {seg.label}
        </span>
      {:else}
        <span
          class="px-1 py-0.5 font-medium text-base-content/90 whitespace-nowrap"
        >
          {seg.label}
        </span>
      {/if}
    {/each}
  </nav>
{/if}
