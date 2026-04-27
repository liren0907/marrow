// Bundles the `demo/` folder at build time into a flat list of mock files
// the FS can seed. Vite's `import.meta.glob` with `?raw` reads each file's
// source as a string, eager so we don't have to await N micro-imports at
// startup.
//
// We rewrite paths from the build-time relative form ("./demo/notes/foo.md")
// to the workspace-style absolute form ("/demo/notes/foo.md") since the
// mock workspace is rooted at "/demo". This keeps every code path that
// already assumes absolute paths (workspace store, file index, etc.) honest
// — the FS doesn't get a special second flavour of paths.

import { mockFs } from "./fs";

const DEMO_ROOT = "/demo";

const rawFiles = import.meta.glob("./demo/**/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

let seeded = false;

export function ensureDemoSeeded(): void {
  if (seeded) return;
  seeded = true;

  const baseMtime = Date.parse("2025-04-28T10:00:00Z");

  const files = Object.entries(rawFiles).map(([key, content]) => {
    // key looks like "./demo/notes/quick-start.md" — strip the leading "."
    // and prepend "/demo"-ness via the slice that drops "./demo".
    const path = DEMO_ROOT + key.slice("./demo".length);
    return {
      path,
      isDir: false,
      content,
      // Stable mtime so reload doesn't constantly bump "edited just now".
      mtime: baseMtime,
    };
  });

  // Also seed the root directory entry so `list_directory("/demo")` works
  // even before any file has materialised the parent via ensureDir.
  mockFs.seed([
    { path: DEMO_ROOT, isDir: true, content: "", mtime: baseMtime },
    ...files,
  ]);
}

export const DEMO_WORKSPACE_ROOT = DEMO_ROOT;
export const DEMO_WORKSPACE_NAME = "Demo Workspace";
