import { SvelteMap } from "svelte/reactivity";
import { workspace } from "./workspace.svelte";
import { readTextFile } from "./tauri";

const TAG_RE = /(?<![\w/])#([\w][\w/-]*)/g;

// Internal bookkeeping: which tags each source file contributed.
// Lets dropEntriesForFile run in O(tags-in-file) instead of O(total-tags).
const bySource = new Map<string, Set<string>>();

export const tags = $state<{
  byTag: SvelteMap<string, string[]>;
  isBuilding: boolean;
  lastBuilt: number;
}>({
  byTag: new SvelteMap(),
  isBuilding: false,
  lastBuilt: 0,
});

function scanContent(content: string): Set<string> {
  const found = new Set<string>();
  let inFence = false;
  for (const rawLine of content.split(/\r?\n/)) {
    const stripped = rawLine.trimStart();
    if (stripped.startsWith("```") || stripped.startsWith("~~~")) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    TAG_RE.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = TAG_RE.exec(rawLine)) !== null) {
      found.add(m[1].toLowerCase());
    }
  }
  return found;
}

export function dropEntriesForFile(sourcePath: string): void {
  const oldTags = bySource.get(sourcePath);
  if (!oldTags) return;
  for (const tag of oldTags) {
    const sources = tags.byTag.get(tag);
    if (!sources) continue;
    const next = sources.filter((s) => s !== sourcePath);
    if (next.length === 0) {
      tags.byTag.delete(tag);
    } else {
      tags.byTag.set(tag, next);
    }
  }
  bySource.delete(sourcePath);
}

export function addEntriesForFile(sourcePath: string, content: string): void {
  const found = scanContent(content);
  if (found.size === 0) return;
  bySource.set(sourcePath, found);
  for (const tag of found) {
    const sources = tags.byTag.get(tag) ?? [];
    if (!sources.includes(sourcePath)) {
      sources.push(sourcePath);
      sources.sort((a, b) => a.localeCompare(b));
      tags.byTag.set(tag, sources);
    }
  }
}

export async function updateTagsForFile(
  sourcePath: string,
  removed: boolean,
): Promise<void> {
  dropEntriesForFile(sourcePath);
  if (removed) return;
  try {
    const result = await readTextFile(sourcePath);
    addEntriesForFile(sourcePath, result.content);
  } catch {
    // file may have vanished between event and read
  }
}

export async function rebuildTags(): Promise<void> {
  if (tags.isBuilding) return;
  tags.isBuilding = true;
  try {
    tags.byTag.clear();
    bySource.clear();
    const mdFiles = workspace.fileIndex.filter((f) => f.kind === "markdown");
    const BATCH = 16;
    for (let i = 0; i < mdFiles.length; i += BATCH) {
      const batch = mdFiles.slice(i, i + BATCH);
      await Promise.all(
        batch.map(async (f) => {
          try {
            const result = await readTextFile(f.path);
            addEntriesForFile(f.path, result.content);
          } catch {
            // skip unreadable
          }
        }),
      );
    }
    tags.lastBuilt = Date.now();
  } finally {
    tags.isBuilding = false;
  }
}

export function tagList(): { tag: string; count: number }[] {
  const out: { tag: string; count: number }[] = [];
  for (const [tag, sources] of tags.byTag) {
    out.push({ tag, count: sources.length });
  }
  out.sort((a, b) => b.count - a.count || a.tag.localeCompare(b.tag));
  return out;
}

export function filesForTag(tag: string): string[] {
  return tags.byTag.get(tag) ?? [];
}
