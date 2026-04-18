export type Activity = "files" | "search" | "tags" | "graph" | "backlinks";

const STORAGE_KEY = "marrow.activity";
const VALID: Set<Activity> = new Set([
  "files",
  "search",
  "tags",
  "graph",
  "backlinks",
]);

function loadInitial(): Activity {
  if (typeof localStorage === "undefined") return "files";
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw && VALID.has(raw as Activity)) return raw as Activity;
  return "files";
}

export const activityBar = $state<{ current: Activity }>({
  current: loadInitial(),
});

export function setActivity(next: Activity): void {
  activityBar.current = next;
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // ignore
  }
}
