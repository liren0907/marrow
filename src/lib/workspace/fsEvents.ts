import { listen } from "@tauri-apps/api/event";
import type { FsEventPayload } from "./types";
import { workspace } from "./workspace.svelte";
import { tree } from "$lib/tree/treeState.svelte";

function parentDir(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx > 0 ? path.slice(0, idx) : path;
}

function handleFsEvent(payload: FsEventPayload): void {
  const parents = new Set<string>();
  for (const path of payload.paths) {
    parents.add(parentDir(path));
    workspace.notifyExternalChange(path, payload.kind);
  }
  for (const parent of parents) {
    // Only refresh directories the user has already expanded / loaded.
    if (tree.getChildren(parent) !== undefined) {
      void tree.load(parent);
    }
  }
}

export async function initFsEvents(): Promise<() => void> {
  const unlisten = await listen<FsEventPayload>("fs-event", (event) => {
    handleFsEvent(event.payload);
  });
  return unlisten;
}
