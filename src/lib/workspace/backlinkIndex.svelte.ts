import { SvelteMap } from "svelte/reactivity";
import { WIKI_LINK_RE } from "$lib/editor/milkdown/wikiLink/regex";
import { workspace } from "./workspace.svelte";
import { readTextFile } from "./tauri";

export interface BacklinkEntry {
  sourcePath: string;
  target: string; // raw target string from [[target]]
}

// Public reactive state. Maps survive Svelte reactivity via SvelteMap.
export const backlinks = $state<{
  byTarget: SvelteMap<string, BacklinkEntry[]>;
  unresolvedBySource: SvelteMap<string, string[]>;
  isBuilding: boolean;
  lastBuilt: number;
}>({
  byTarget: new SvelteMap(),
  unresolvedBySource: new SvelteMap(),
  isBuilding: false,
  lastBuilt: 0,
});

function targetKey(rawTarget: string): string {
  return rawTarget.replace(/\.md$/i, "").trim().toLowerCase();
}

function scanContent(
  content: string,
): { resolved: { key: string; raw: string }[]; unresolved: string[] } {
  const resolved: { key: string; raw: string }[] = [];
  const unresolved: string[] = [];
  WIKI_LINK_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = WIKI_LINK_RE.exec(content)) !== null) {
    // Skip transclusion syntax `![[...]]` — handled separately.
    if (m.index > 0 && content[m.index - 1] === "!") continue;
    const raw = m[1].trim();
    if (!raw) continue;
    const key = targetKey(raw);
    resolved.push({ key, raw });
    if (workspace.resolveBasename(raw) === null) {
      unresolved.push(raw);
    }
  }
  return { resolved, unresolved };
}

/**
 * Drop all entries that originate from `sourcePath`.
 * Called before re-scanning a file's content.
 */
export function dropEntriesForFile(sourcePath: string): void {
  for (const [key, entries] of backlinks.byTarget) {
    const next = entries.filter((e) => e.sourcePath !== sourcePath);
    if (next.length === 0) {
      backlinks.byTarget.delete(key);
    } else if (next.length !== entries.length) {
      backlinks.byTarget.set(key, next);
    }
  }
  backlinks.unresolvedBySource.delete(sourcePath);
}

/**
 * Add entries from a file's current content to the index.
 * Caller is responsible for dropping prior entries first.
 */
export function addEntriesForFile(sourcePath: string, content: string): void {
  const { resolved, unresolved } = scanContent(content);
  for (const { key, raw } of resolved) {
    const list = backlinks.byTarget.get(key) ?? [];
    list.push({ sourcePath, target: raw });
    backlinks.byTarget.set(key, list);
  }
  if (unresolved.length > 0) {
    backlinks.unresolvedBySource.set(sourcePath, unresolved);
  }
}

/**
 * Incremental update: re-scan one file. Used by fs-event handler.
 * Pass `null` content for removed files (just drops entries).
 */
export async function updateBacklinksForFile(
  sourcePath: string,
  removed: boolean,
): Promise<void> {
  dropEntriesForFile(sourcePath);
  if (removed) return;
  try {
    const result = await readTextFile(sourcePath);
    addEntriesForFile(sourcePath, result.content);
  } catch {
    // File may have vanished between event and read — leave dropped.
  }
}

/**
 * Full rebuild from `workspace.fileIndex`. Called on workspace open.
 */
export async function rebuildBacklinks(): Promise<void> {
  if (backlinks.isBuilding) return;
  backlinks.isBuilding = true;
  try {
    backlinks.byTarget.clear();
    backlinks.unresolvedBySource.clear();
    const mdFiles = workspace.fileIndex.filter((f) => f.kind === "markdown");
    // Read in parallel batches to avoid IPC saturation.
    const BATCH = 16;
    for (let i = 0; i < mdFiles.length; i += BATCH) {
      const batch = mdFiles.slice(i, i + BATCH);
      await Promise.all(
        batch.map(async (f) => {
          try {
            const result = await readTextFile(f.path);
            addEntriesForFile(f.path, result.content);
          } catch {
            // skip unreadable files
          }
        }),
      );
    }
    backlinks.lastBuilt = Date.now();
  } finally {
    backlinks.isBuilding = false;
  }
}

/**
 * Lookup helper: backlinks pointing at the given file.
 */
export function backlinksFor(targetPath: string): BacklinkEntry[] {
  // The file's basename (without .md) is the lookup key.
  const name = targetPath.split(/[/\\]/).pop() ?? targetPath;
  const key = targetKey(name);
  const all = backlinks.byTarget.get(key) ?? [];
  // Exclude self-references.
  return all.filter((e) => e.sourcePath !== targetPath);
}

/**
 * Snapshot of all unresolved links across the workspace, grouped by source.
 */
export function unresolvedAll(): { source: string; targets: string[] }[] {
  const out: { source: string; targets: string[] }[] = [];
  for (const [source, targets] of backlinks.unresolvedBySource) {
    if (targets.length > 0) out.push({ source, targets });
  }
  out.sort((a, b) => a.source.localeCompare(b.source));
  return out;
}
