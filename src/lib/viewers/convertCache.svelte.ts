import type { ConvertAsset } from "$lib/workspace/tauri";

/** Which converter produced a cache entry. The same source file yields
 * different Markdown per engine, so the cache key must include it. */
export type ConvertEngine = "native" | "markitdown";

interface CacheEntry {
  markdown: string;
  /** Sidecar assets (DOCX/PPTX pictures today). Empty for converters that
   * don't produce any. */
  assets: ConvertAsset[];
}

const MAX_ENTRIES = 20;
const cache = new Map<string, CacheEntry>();

/** Composite cache key. The same source file yields different Markdown per
 * engine, so both engine and path are folded in. JSON.stringify of the
 * pair is collision-free regardless of what characters appear in the path. */
function keyFor(engine: ConvertEngine, path: string): string {
  return JSON.stringify([engine, path]);
}

export function getCached(
  engine: ConvertEngine,
  path: string,
): CacheEntry | null {
  return cache.get(keyFor(engine, path)) ?? null;
}

export function setCached(
  engine: ConvertEngine,
  path: string,
  markdown: string,
  assets: ConvertAsset[] = [],
): void {
  const key = keyFor(engine, path);
  cache.delete(key);
  cache.set(key, { markdown, assets });
  while (cache.size > MAX_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) break;
    cache.delete(oldest);
  }
}

/** Drop every cached conversion of `path`, across all engines. Called from
 * the fs-watcher when the source file changes — hence the single-path
 * signature. Keep the engine list in sync with the ConvertEngine union. */
export function invalidateCached(path: string): void {
  cache.delete(keyFor("native", path));
  cache.delete(keyFor("markitdown", path));
}
