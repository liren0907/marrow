const MAX_ENTRIES = 20;
const cache = new Map<string, string>();

export function getCached(path: string): string | null {
  return cache.get(path) ?? null;
}

export function setCached(path: string, markdown: string): void {
  cache.delete(path);
  cache.set(path, markdown);
  while (cache.size > MAX_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) break;
    cache.delete(oldest);
  }
}

export function invalidateCached(path: string): void {
  cache.delete(path);
}
