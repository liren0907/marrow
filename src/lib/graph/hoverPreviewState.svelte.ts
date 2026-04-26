// Self-contained hover-preview feature for the graph view.
//
// `hoverState` is a module-level reactive store; <HoverPreview /> reads it
// to render the floating card. GraphTab calls `scheduleHoverPreview()` on
// node mouseover and `clearHoverPreview()` on mouseout / tap / destroy.
//
// LRU cache + 1s delay timer live as module-private state so the entire
// feature owns its lifecycle. Single-instance assumption — there is only
// ever one graph view visible at a time, so a shared store is fine.

import { readTextFile } from "$lib/workspace/tauri";

interface HoverState {
  path: string | null;
  content: string;
  pos: { x: number; y: number };
}

export const hoverState = $state<HoverState>({
  path: null,
  content: "",
  pos: { x: 0, y: 0 },
});

// LRU cache of file body previews (first ~200 chars after frontmatter).
const previewCache = new Map<string, string>();
const PREVIEW_CACHE_MAX = 32;

const HOVER_DELAY_MS = 1000;
const PREVIEW_BODY_CHARS = 200;

let hoverDelayTimer: ReturnType<typeof setTimeout> | null = null;

// Map preserves insertion order — touch-to-back keeps most-recent at the
// end, oldest at the front for eviction.
function setPreview(path: string, body: string): void {
  if (previewCache.has(path)) previewCache.delete(path);
  else if (previewCache.size >= PREVIEW_CACHE_MAX) {
    const oldest = previewCache.keys().next().value;
    if (oldest !== undefined) previewCache.delete(oldest);
  }
  previewCache.set(path, body);
}

function getPreview(path: string): string | undefined {
  const v = previewCache.get(path);
  if (v !== undefined) {
    previewCache.delete(path);
    previewCache.set(path, v);
  }
  return v;
}

function stripFrontmatterAndTrim(raw: string): string {
  let body = raw;
  if (body.startsWith("---")) {
    const end = body.indexOf("\n---\n", 4);
    if (end >= 0) body = body.slice(end + 5);
  }
  return body.trim().slice(0, PREVIEW_BODY_CHARS);
}

export function scheduleHoverPreview(path: string, evt?: MouseEvent): void {
  if (hoverDelayTimer) clearTimeout(hoverDelayTimer);
  hoverDelayTimer = setTimeout(async () => {
    hoverDelayTimer = null;
    let cached = getPreview(path);
    if (cached === undefined) {
      try {
        const result = await readTextFile(path);
        cached = stripFrontmatterAndTrim(result.content);
      } catch {
        cached = "(unreadable)";
      }
      setPreview(path, cached);
    }
    hoverState.path = path;
    hoverState.content = cached;
    if (evt) {
      hoverState.pos = { x: evt.clientX + 16, y: evt.clientY + 16 };
    }
  }, HOVER_DELAY_MS);
}

export function clearHoverPreview(): void {
  if (hoverDelayTimer) {
    clearTimeout(hoverDelayTimer);
    hoverDelayTimer = null;
  }
  hoverState.path = null;
}
