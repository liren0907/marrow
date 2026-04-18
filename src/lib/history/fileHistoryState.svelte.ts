import {
  listFileHistory,
  readSnapshot,
  readTextFile,
  restoreSnapshot,
  type SnapshotMeta,
} from "$lib/workspace/tauri";
import { workspace } from "$lib/workspace/workspace.svelte";
import { showError, showSuccess } from "$lib/stores/toastStore.svelte";

type ViewMode = "preview" | "diff";

interface FileHistoryState {
  isOpen: boolean;
  targetPath: string | null;
  targetTitle: string | null;
  entries: SnapshotMeta[];
  selectedIdx: number;
  isLoading: boolean;
  viewMode: ViewMode;
  currentContent: string;
  selectedContent: string | null;
  isSelectedLoading: boolean;
  isRestoring: boolean;
}

export const fileHistory = $state<FileHistoryState>({
  isOpen: false,
  targetPath: null,
  targetTitle: null,
  entries: [],
  selectedIdx: 0,
  isLoading: false,
  viewMode: "preview",
  currentContent: "",
  selectedContent: null,
  isSelectedLoading: false,
  isRestoring: false,
});

function basename(p: string): string {
  const idx = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
  return idx >= 0 ? p.slice(idx + 1) : p;
}

export async function openFileHistory(path: string): Promise<void> {
  fileHistory.isOpen = true;
  fileHistory.targetPath = path;
  fileHistory.targetTitle = basename(path);
  fileHistory.entries = [];
  fileHistory.selectedIdx = 0;
  fileHistory.isLoading = true;
  fileHistory.viewMode = "preview";
  fileHistory.currentContent = "";
  fileHistory.selectedContent = null;
  fileHistory.isSelectedLoading = false;
  fileHistory.isRestoring = false;

  try {
    const [entries, current] = await Promise.all([
      listFileHistory(path),
      readTextFile(path).then((r) => r.content).catch(() => ""),
    ]);
    fileHistory.entries = entries;
    fileHistory.currentContent = current;
    fileHistory.isLoading = false;
    if (entries.length > 0) {
      await selectSnapshot(0);
    }
  } catch (e) {
    fileHistory.isLoading = false;
    showError(
      `Failed to load history: ${e instanceof Error ? e.message : String(e)}`,
    );
  }
}

export function closeFileHistory(): void {
  fileHistory.isOpen = false;
  fileHistory.targetPath = null;
  fileHistory.entries = [];
  fileHistory.selectedContent = null;
}

export async function selectSnapshot(idx: number): Promise<void> {
  if (idx < 0 || idx >= fileHistory.entries.length) return;
  fileHistory.selectedIdx = idx;
  const entry = fileHistory.entries[idx];
  if (entry.op === "rename") {
    fileHistory.selectedContent = null;
    return;
  }
  fileHistory.isSelectedLoading = true;
  fileHistory.selectedContent = null;
  try {
    fileHistory.selectedContent = await readSnapshot(entry.hash);
  } catch (e) {
    showError(
      `Failed to load snapshot: ${e instanceof Error ? e.message : String(e)}`,
    );
  } finally {
    fileHistory.isSelectedLoading = false;
  }
}

export function setViewMode(mode: ViewMode): void {
  fileHistory.viewMode = mode;
}

export async function restoreSelected(): Promise<void> {
  const path = fileHistory.targetPath;
  if (!path) return;
  const entry = fileHistory.entries[fileHistory.selectedIdx];
  if (!entry || entry.op === "rename") return;
  fileHistory.isRestoring = true;
  try {
    await restoreSnapshot(path, entry.hash);
    showSuccess(`Restored ${fileHistory.targetTitle ?? "file"}`);
    // Reload the file in any open tab and refresh the history list.
    workspace.notifyExternalChange(path, "modify");
    await openFileHistory(path);
  } catch (e) {
    showError(
      `Restore failed: ${e instanceof Error ? e.message : String(e)}`,
    );
    fileHistory.isRestoring = false;
  }
}
