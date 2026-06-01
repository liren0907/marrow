// Distill's own source lister — the read-side mirror of `extractAnnotations`
// in tauri.ts. The Distill page brings its OWN file explorer (it does NOT reuse
// the global FileTree / workspace store), so it needs a transport-aware way to
// list the `.md` files under an arbitrary root:
//   - http   → the dev-only :7080 bridge (`GET /distill/tree`), so a browser
//              can list REAL files on disk.
//   - invoke → the generic `list_workspace_files` command over an arbitrary
//              root (real files, native app). This is a read-only listing
//              command, not the global FileTree's UI/state.
//   - mock   → `list_workspace_files` again, which tauri.ts routes to the
//              in-memory devmock when not running under Tauri (offline sample).
//
// Nothing here touches the global workspace open/list/read seam beyond REUSING
// the existing `listWorkspaceFiles` wrapper; it adds no backend command.

import { listWorkspaceFiles, type DistillTransport } from "$lib/workspace/tauri";
import { listDistillTreeHttp } from "$lib/workspace/distillHttp";

/** One selectable source `.md`. `path` is absolute (sent to extraction);
 *  `rel` is relative to the queried root (shown in the explorer). */
export interface DistillSource {
  path: string;
  rel: string;
}

/** Path relative to `root` for display; falls back to the full path when the
 *  listing returns something outside the root (shouldn't happen, but safe). */
function relOf(root: string, path: string): string {
  if (root && path.startsWith(root)) {
    return path.slice(root.length).replace(/^[/\\]+/, "");
  }
  return path;
}

/** List the `.md` files under `root` using whichever backend `transport`
 *  points at. Sorted by relative path. Throws on a bad root / transport error
 *  so the caller can surface it. */
export async function listDistillSources(
  root: string,
  transport: DistillTransport,
): Promise<DistillSource[]> {
  if (transport === "http") {
    // Already returns {path, rel}, sorted by rel on the Rust side.
    return listDistillTreeHttp(root);
  }
  // invoke (native real files) and mock (offline sample) both go through the
  // generic listing wrapper; mock routing happens inside tauri.ts (!isInTauri).
  const files = await listWorkspaceFiles(root);
  return files
    .filter((f) => f.kind === "markdown")
    .map((f) => ({ path: f.path, rel: relOf(root, f.path) }))
    .sort((a, b) => a.rel.localeCompare(b.rel));
}
