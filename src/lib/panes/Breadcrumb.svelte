<script lang="ts">
  import type { Pane } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { revealInTree } from "$lib/tree/treeState.svelte";
  import MetaIconButton from "$lib/components/ui/MetaIconButton.svelte";
  import { relativeTime, formatFullDateTime } from "$lib/utils/relativeTime";
  import {
    uiSettings,
    togglePaneOutline,
  } from "$lib/settings/uiSettings.svelte";

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

  // ─── Right-side meta + controls (merged from former EditorMetaHeader) ───
  // Breadcrumb is now the single chrome row above the editor: path on the
  // left, "edited X ago" + view-mode toggle on the right. Anything that's
  // tab-scoped chrome belongs here so we don't grow another horizontal
  // layer above the content.

  const modifiedRel = $derived.by(() => {
    const ms = activeTab?.lastKnownMtime;
    return ms ? relativeTime(ms) : "";
  });
  const modifiedFull = $derived.by(() => {
    const ms = activeTab?.lastKnownMtime;
    return ms ? formatFullDateTime(ms) : "";
  });

  // Pretty/Raw view-mode toggle — markdown tabs only. Cmd+/ does the same
  // thing globally (handled in shortcuts.svelte.ts), so even if the user
  // hides the breadcrumb via settings the keyboard path still works.
  const viewMode = $derived(activeTab?.viewMode ?? "pretty");
  function toggleViewMode(): void {
    if (!activeTab) return;
    workspace.patchTab(activeTab.id, {
      viewMode: viewMode === "pretty" ? "raw" : "pretty",
    });
  }
</script>

{#if segments.length > 0}
  <nav
    class="flex items-center gap-2 px-3 h-7 shrink-0 border-b border-[color:var(--mw-rule)] text-[11px] text-base-content/60"
    aria-label="Editor breadcrumb"
  >
    <!-- Path segments — flex-1 + min-w-0 + overflow-x-auto so very deep
         paths scroll horizontally without pushing the right-side controls
         off the row. -->
    <div class="flex items-center gap-0.5 flex-1 min-w-0 overflow-x-auto">
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
    </div>

    <!-- Right-side controls cluster — relative-time meta + view-mode
         toggle. Only renders the toggle for markdown tabs; the time hides
         when there's no mtime (virtual tabs like graph view). -->
    <div class="flex items-center gap-1.5 shrink-0">
      {#if modifiedRel}
        <span
          class="breadcrumb-edited whitespace-nowrap"
          title={modifiedFull}
        >
          edited {modifiedRel}
        </span>
      {/if}
      {#if activeTab?.kind === "markdown"}
        {#if modifiedRel}
          <span class="text-base-content/30 select-none">·</span>
        {/if}
        <!-- Outline toggle. Tied to the global uiSettings preference, so
             persists across sessions; auto-collapse based on pane width
             still applies on top (handled in Pane.svelte). Sits left of
             the view-mode toggle so the editor-mode controls stay
             grouped together at the far right. -->
        <MetaIconButton
          icon={uiSettings.showPaneOutline
            ? "panel-right-close"
            : "panel-right-open"}
          tooltip={uiSettings.showPaneOutline
            ? "Hide outline"
            : "Show outline"}
          onclick={togglePaneOutline}
        />
        <span class="text-base-content/30 select-none">·</span>
        <MetaIconButton
          icon={viewMode === "pretty" ? "code" : "eye"}
          tooltip={viewMode === "pretty"
            ? "Switch to source (⌘/)"
            : "Switch to preview (⌘/)"}
          active={viewMode === "raw"}
          onclick={toggleViewMode}
        />
      {/if}
    </div>
  </nav>
{/if}

<style>
  .breadcrumb-edited {
    color: var(--mw-ink-3);
    font-variant-numeric: tabular-nums;
    cursor: default;
  }
</style>
