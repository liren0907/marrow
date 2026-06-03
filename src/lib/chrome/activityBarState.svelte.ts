export type Activity = "files" | "search" | "tags" | "graph" | "backlinks" | "distill";

const STORAGE_KEY = "marrow.activity";
const VALID: Set<Activity> = new Set([
  "files",
  "search",
  "tags",
  "graph",
  "backlinks",
  "distill",
]);

function loadInitial(): Activity {
  if (typeof localStorage === "undefined") return "files";
  const raw = localStorage.getItem(STORAGE_KEY);
  // Distill is a transient mode, never the boot activity — a persisted/stale
  // "distill" (or unknown value) falls back to Files. See setActivity.
  if (raw && raw !== "distill" && VALID.has(raw as Activity)) return raw as Activity;
  return "files";
}

export const activityBar = $state<{ current: Activity }>({
  current: loadInitial(),
});

export function setActivity(next: Activity): void {
  activityBar.current = next;
  if (typeof localStorage === "undefined") return;
  // Don't persist "distill": it's a transient mode and must never become the
  // boot activity (loadInitial also defends against stale values).
  if (next === "distill") return;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // ignore
  }
}
