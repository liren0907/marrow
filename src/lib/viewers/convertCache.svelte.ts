import type { ConvertAsset } from "$lib/workspace/tauri";

interface CacheEntry {
  markdown: string;
  /** Sidecar assets (PPTX pictures today). Empty for converters that
   * don't produce any. */
  assets: ConvertAsset[];
}

const MAX_ENTRIES = 20;
const cache = new Map<string, CacheEntry>();

export function getCached(path: string): CacheEntry | null {
  return cache.get(path) ?? null;
}

export function setCached(
  path: string,
  markdown: string,
  assets: ConvertAsset[] = [],
): void {
  cache.delete(path);
  cache.set(path, { markdown, assets });
  while (cache.size > MAX_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) break;
    cache.delete(oldest);
  }
}

export function invalidateCached(path: string): void {
  cache.delete(path);
}
