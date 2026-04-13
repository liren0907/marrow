import type { Tab } from "$lib/workspace/types";
import { readTextFile } from "$lib/workspace/tauri";

export const conflict = $state<{
  isOpen: boolean;
  tab: Tab | null;
  myContent: string;
  diskContent: string | null;
  showDiff: boolean;
}>({
  isOpen: false,
  tab: null,
  myContent: "",
  diskContent: null,
  showDiff: false,
});

export async function openConflict(tab: Tab, myContent: string): Promise<void> {
  conflict.isOpen = true;
  conflict.tab = tab;
  conflict.myContent = myContent;
  conflict.diskContent = null;
  conflict.showDiff = false;
  try {
    const result = await readTextFile(tab.path);
    if (conflict.tab?.id === tab.id) {
      conflict.diskContent = result.content;
    }
  } catch {
    // leave diskContent null; modal will show "could not read"
  }
}

export function closeConflict(): void {
  conflict.isOpen = false;
  conflict.tab = null;
  conflict.myContent = "";
  conflict.diskContent = null;
  conflict.showDiff = false;
}
