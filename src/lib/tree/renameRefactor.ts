import * as cmd from "$lib/workspace/tauri";
import { workspace } from "$lib/workspace/workspace.svelte";
import {
  backlinks,
  rebuildBacklinks,
} from "$lib/workspace/backlinkIndex.svelte";
import { tabSaveRegistry } from "$lib/workspace/shortcuts.svelte";

export interface RefactorPreview {
  oldPath: string;
  newPath: string;
  oldBasename: string; // without .md
  newBasename: string;
  affectedFiles: string[];
}

export interface RefactorResult {
  failures: { path: string; error: string }[];
}

function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

function dirname(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx > 0 ? path.slice(0, idx) : path;
}

function joinPath(parent: string, name: string): string {
  const sep = parent.includes("\\") ? "\\" : "/";
  return parent.endsWith(sep) ? `${parent}${name}` : `${parent}${sep}${name}`;
}

function stripMd(name: string): string {
  return name.replace(/\.md$/i, "");
}

function ensureMd(name: string): string {
  if (/\.md$/i.test(name)) return name;
  return `${name}.md`;
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function previewRename(
  oldPath: string,
  newName: string,
): RefactorPreview {
  const newNameMd = ensureMd(newName.trim());
  const newPath = joinPath(dirname(oldPath), newNameMd);
  const oldBasename = stripMd(basename(oldPath));
  const newBasename = stripMd(newNameMd);

  // Look up the reverse index by basename key (lowercase, stripped).
  const key = oldBasename.toLowerCase();
  const entries = backlinks.byTarget.get(key) ?? [];
  // Unique source paths, excluding the file being renamed (we still update
  // its own self-references separately).
  const seen = new Set<string>();
  const affectedFiles: string[] = [];
  for (const entry of entries) {
    if (entry.sourcePath === oldPath) continue;
    if (seen.has(entry.sourcePath)) continue;
    seen.add(entry.sourcePath);
    affectedFiles.push(entry.sourcePath);
  }
  // Always include the file itself so its own self-references get rewritten.
  affectedFiles.unshift(oldPath);

  return { oldPath, newPath, oldBasename, newBasename, affectedFiles };
}

/**
 * Runs the rename. Order of operations:
 * 1. Force-save dirty open tabs whose path is in the affected list.
 * 2. fs::rename the target file (so the new basename exists on disk).
 * 3. Re-resolve any open tabs that pointed at oldPath → newPath.
 * 4. For each affected file (including the renamed file at its new path),
 *    load → regex replace → write. Best-effort, errors collected.
 * 5. Refresh fileIndex + rebuild backlinks.
 */
export async function executeRename(
  preview: RefactorPreview,
): Promise<RefactorResult> {
  const failures: { path: string; error: string }[] = [];

  // Step 1: force-save dirty affected tabs.
  for (const pane of workspace.panes) {
    for (const tab of pane.tabs) {
      if (!tab.isDirty) continue;
      if (!preview.affectedFiles.includes(tab.path)) continue;
      const saveFn = tabSaveRegistry.get(tab.id);
      if (saveFn) {
        try {
          await saveFn();
        } catch (e) {
          failures.push({
            path: tab.path,
            error: `force-save failed: ${e instanceof Error ? e.message : String(e)}`,
          });
          // Abort if we can't save a dirty file we're about to rewrite.
          return { failures };
        }
      }
    }
  }

  // Step 2: rename the file itself.
  try {
    await cmd.renamePath(preview.oldPath, preview.newPath);
  } catch (e) {
    failures.push({
      path: preview.oldPath,
      error: `rename failed: ${e instanceof Error ? e.message : String(e)}`,
    });
    return { failures };
  }

  // Step 3: repoint open tabs.
  const oldName = basename(preview.oldPath);
  const newName = basename(preview.newPath);
  for (const pane of workspace.panes) {
    for (const tab of pane.tabs) {
      if (tab.path === preview.oldPath) {
        tab.path = preview.newPath;
        tab.title = newName;
        tab.lastKnownMtime = undefined;
        tab.reloadToken = (tab.reloadToken ?? 0) + 1;
      }
    }
  }

  // Step 4: rewrite refs in affected files.
  // The pattern matches `[[oldBasename]]` or `[[oldBasename.md]]` with an
  // optional leading `!` (preserved on output). It does NOT match
  // `[[oldBasenameButLonger]]` because the closing `]]` is anchored.
  const pat = new RegExp(
    "(!?)\\[\\[" + escapeRegex(preview.oldBasename) + "(\\.md)?\\]\\]",
    "gi",
  );

  // The file we just renamed lives at its new path now.
  const filesToRewrite = preview.affectedFiles.map((p) =>
    p === preview.oldPath ? preview.newPath : p,
  );

  for (const file of filesToRewrite) {
    try {
      const result = await cmd.readTextFile(file);
      const replaced = result.content.replace(
        pat,
        (_match, bang: string, ext: string | undefined) =>
          `${bang}[[${preview.newBasename}${ext ?? ""}]]`,
      );
      if (replaced !== result.content) {
        // Skip mtime check — the user already confirmed the rename.
        await cmd.writeTextFile(file, replaced, undefined);
      }
    } catch (e) {
      failures.push({
        path: file,
        error: e instanceof Error ? e.message : String(e),
      });
    }
  }

  // Step 5: refresh derived state.
  await workspace.refreshFileIndex();
  void rebuildBacklinks();

  return { failures };
}

// Convenience export for the modal — keeps the other helpers private to this file.
export { ensureMd as ensureMdExtension };
