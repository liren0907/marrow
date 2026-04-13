import * as cmd from "$lib/workspace/tauri";
import { workspace } from "$lib/workspace/workspace.svelte";
import { tree } from "./treeState.svelte";
import { rebuildBacklinks } from "$lib/workspace/backlinkIndex.svelte";
import { showError, showSuccess } from "$lib/stores/toastStore.svelte";

function joinPath(parent: string, name: string): string {
  // Normalize trailing separators on the parent, use forward slash on macOS.
  const sep = parent.includes("\\") ? "\\" : "/";
  return parent.endsWith(sep) ? `${parent}${name}` : `${parent}${sep}${name}`;
}

function ensureMdExtension(name: string): string {
  if (/\.[a-z0-9]+$/i.test(name)) return name;
  return `${name}.md`;
}

function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

function dirname(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx > 0 ? path.slice(0, idx) : path;
}

async function refreshAfterMutation(parent: string): Promise<void> {
  await tree.load(parent);
  await workspace.refreshFileIndex();
  void rebuildBacklinks();
}

export async function newFile(parentDir: string, name: string): Promise<void> {
  if (!name.trim()) return;
  const finalName = ensureMdExtension(name.trim());
  const path = joinPath(parentDir, finalName);
  try {
    await cmd.createFile(path);
    await refreshAfterMutation(parentDir);
    workspace.openFile(path);
    showSuccess(`Created ${finalName}`);
  } catch (e) {
    showError(`Failed to create file: ${e instanceof Error ? e.message : String(e)}`);
  }
}

export async function newFolder(parentDir: string, name: string): Promise<void> {
  if (!name.trim()) return;
  const path = joinPath(parentDir, name.trim());
  try {
    await cmd.createDirectory(path);
    await refreshAfterMutation(parentDir);
    showSuccess(`Created ${name}/`);
  } catch (e) {
    showError(`Failed to create folder: ${e instanceof Error ? e.message : String(e)}`);
  }
}

function closeTabsUnder(prefix: string): void {
  for (const pane of workspace.panes) {
    const toClose = pane.tabs.filter((t) => t.path === prefix || t.path.startsWith(prefix + "/") || t.path.startsWith(prefix + "\\"));
    for (const tab of toClose) {
      workspace.closeTab(pane.id, tab.id);
    }
  }
}

export async function deleteEntry(path: string, isDir: boolean): Promise<void> {
  try {
    await cmd.deletePath(path);
    closeTabsUnder(path);
    await refreshAfterMutation(dirname(path));
    showSuccess(`Deleted ${basename(path)}${isDir ? "/" : ""}`);
  } catch (e) {
    showError(`Failed to delete: ${e instanceof Error ? e.message : String(e)}`);
  }
}

export async function renameFolder(oldPath: string, newName: string): Promise<void> {
  if (!newName.trim()) return;
  const parent = dirname(oldPath);
  const newPath = joinPath(parent, newName.trim());
  try {
    await cmd.renamePath(oldPath, newPath);
    // Repoint any open tabs whose path is inside the renamed folder.
    for (const pane of workspace.panes) {
      for (const tab of pane.tabs) {
        if (tab.path === oldPath || tab.path.startsWith(oldPath + "/") || tab.path.startsWith(oldPath + "\\")) {
          tab.path = newPath + tab.path.slice(oldPath.length);
        }
      }
    }
    await refreshAfterMutation(parent);
    showSuccess(`Renamed to ${newName.trim()}`);
  } catch (e) {
    showError(`Failed to rename: ${e instanceof Error ? e.message : String(e)}`);
  }
}
