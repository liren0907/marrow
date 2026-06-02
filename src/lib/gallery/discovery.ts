import type { Component } from "svelte";

// Lazy component loaders for the dev-only /gallery. Keyed by absolute-from-root
// path (e.g. "/src/lib/components/ui/Button.svelte"). Invoked only when a
// manifest item is actually previewed — store-coupled/heavy modules that are
// marked "note" in the manifest are never loaded. (Mirrors the existing
// import.meta.glob usage in src/lib/devmock/demoData.ts.)
export const LOADERS = import.meta.glob("/src/lib/**/*.svelte") as Record<
  string,
  () => Promise<{ default: Component }>
>;

/** "$lib/components/ui/Button.svelte" -> "/src/lib/components/ui/Button.svelte" */
export function pathForImport(importPath: string): string {
  return importPath.replace(/^\$lib\//, "/src/lib/");
}
