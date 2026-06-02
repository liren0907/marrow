import { workspace } from "$lib/workspace/workspace.svelte";
import { listRecentWorkspaces } from "$lib/workspace/tauri";

// Dev-only seed hook for the /gallery. Store-driven components read the global
// `workspace` singleton; the gallery lives OUTSIDE the (app) route group so the
// workspace runtime never initialises it. We seed it here by opening the
// devmock demo workspace (browser → isInTauri=false → all invoke() routes to
// devmock, so this performs NO real Tauri/file I/O) and opening one markdown
// file, so panes/tabs/fileIndex/backlinks/tags all have realistic data to
// render against. Idempotent.
let seeded = false;

export async function seedWorkspace(): Promise<void> {
  if (seeded || workspace.info) {
    seeded = true;
    return;
  }
  const recents = await listRecentWorkspaces(1).catch(() => []);
  const root = recents[0]?.last_path ?? "/demo";
  await workspace.open(root); // devmock-backed: populates info + fileIndex (+ backlinks/tags)

  // Open one markdown file so Tab/TabBar/Breadcrumb/OutlineTab have content.
  const md = workspace.fileIndex.find((f) => f.kind === "markdown");
  if (md) workspace.openFile(md.path);

  seeded = true;
}
