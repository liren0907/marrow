import { getGitBranch } from "./tauri";
import { workspace } from "./workspace.svelte";

export const gitState = $state<{
  branch: string | null;
  watcherRunning: boolean;
}>({
  branch: null,
  watcherRunning: false,
});

let refreshTimer: ReturnType<typeof setTimeout> | null = null;

export async function refreshGitBranch(): Promise<void> {
  const root = workspace.info?.root;
  if (!root) {
    gitState.branch = null;
    return;
  }
  try {
    gitState.branch = await getGitBranch(root);
  } catch {
    gitState.branch = null;
  }
}

/** Debounced variant used by the fs-event hook — bursts of fs events
 * collapse into a single git check ~2s after the last event. */
export function scheduleGitRefresh(): void {
  if (refreshTimer) clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => {
    refreshTimer = null;
    void refreshGitBranch();
  }, 2000);
}
