<script lang="ts">
  // The Distill activity's sidebar panel: a markdown source picker. It hosts the
  // DistillExplorer (workspace root, transport-aware listing) and, on pick, opens
  // that file as a normal MarkdownTab in the EDITING pane — the pane that isn't
  // showing Distill. The panel-only DistillTab in the other pane follows that
  // markdown and re-extracts. So this panel just feeds the neighbouring pane;
  // the explorer / editor / extractor stay fully decoupled.
  import { untrack } from "svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { distillTransport } from "$lib/workspace/tauri";
  import DistillExplorer from "./DistillExplorer.svelte";

  const transport = distillTransport();

  // Root = the open workspace; the explorer lists .md beneath it. Kept in sync
  // if the workspace changes while the panel is mounted, but still overridable
  // via the explorer's own root input (dev/http).
  let root = $state(untrack(() => workspace.info?.root ?? ""));
  $effect(() => {
    const r = workspace.info?.root ?? "";
    if (r && r !== untrack(() => root)) root = r;
  });

  // Highlight whichever markdown is currently open (= what Distill is showing):
  // the active markdown tab in any pane.
  const selected = $derived.by(() => {
    for (const pane of workspace.panes) {
      const active = pane.tabs.find((t) => t.id === pane.activeTabId);
      if (active?.kind === "markdown") return active.path;
    }
    return null;
  });

  function onpick(path: string): void {
    // Open into the editing pane (the one not showing Distill); if there isn't a
    // separate one yet, openFile falls back to the active pane.
    const editing = workspace.panes.find((p) => {
      const active = p.tabs.find((t) => t.id === p.activeTabId);
      return active?.kind !== "distill";
    });
    workspace.openFile(path, editing?.id);
  }

  function onroot(next: string): void {
    root = next;
  }
</script>

<DistillExplorer {root} {transport} {selected} {onpick} {onroot} />
