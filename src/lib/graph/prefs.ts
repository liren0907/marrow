// User-controlled preferences for the graph view, persisted to localStorage.
//
// Two independent buckets share this file because both are "remembered
// settings tied to graph view UI" — splitting them across two files would
// only force two imports for every consumer. Each bucket has its own
// storage key so they evolve independently.

// ─── Display preferences ─────────────────────────────────────────────────

export type NodeSizePreset = "xs" | "sm" | "md" | "lg" | "xl";
export type LabelSizePreset = "hidden" | "xs" | "sm" | "md" | "lg";
export type LabelMode = "always" | "hover";

export interface DisplayPrefs {
  nodeSize: NodeSizePreset;
  labelSize: LabelSizePreset;
  labelMode: LabelMode;
  showEdgeArrows: boolean;
  edgeWidth: number;
}

export const DEFAULT_DISPLAY_PREFS: DisplayPrefs = {
  nodeSize: "sm",
  labelSize: "sm",
  labelMode: "always",
  showEdgeArrows: false,
  edgeWidth: 1,
};

// [min, max] node radius range — degree gets mapped into this band by
// cytoscape's mapData(). Higher-degree nodes render bigger.
export const NODE_SIZE_MAP: Record<NodeSizePreset, [number, number]> = {
  xs: [2, 6],
  sm: [4, 12],
  md: [6, 18],
  lg: [8, 24],
  xl: [10, 30],
};

// 0 means "labels hidden" — styleFromTheme uses this to drop the label string.
export const LABEL_SIZE_MAP: Record<LabelSizePreset, number> = {
  hidden: 0,
  xs: 8,
  sm: 9,
  md: 10,
  lg: 12,
};

const DISPLAY_STORAGE_KEY = "marrow.graph.display";

export function loadDisplayPrefs(): DisplayPrefs {
  if (typeof localStorage === "undefined") return { ...DEFAULT_DISPLAY_PREFS };
  try {
    const raw = localStorage.getItem(DISPLAY_STORAGE_KEY);
    if (!raw) return { ...DEFAULT_DISPLAY_PREFS };
    return { ...DEFAULT_DISPLAY_PREFS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_DISPLAY_PREFS };
  }
}

export function saveDisplayPrefs(p: DisplayPrefs): void {
  try {
    localStorage.setItem(DISPLAY_STORAGE_KEY, JSON.stringify(p));
  } catch {
    // storage full / disabled — silently skip
  }
}

// ─── Filter preferences ──────────────────────────────────────────────────

export type ViewMode = "all" | "local-1" | "local-2";
export type ColorMode = "default" | "folder" | "tag";

// view mode / color mode / folder / tag filters carry over between sessions.
// searchFilter is intentionally NOT persisted: a stale search string across
// sessions is confusing.
export interface SavedFilters {
  viewMode: ViewMode;
  colorMode: ColorMode;
  folderFilter: string[];
  tagFilter: string[];
}

export const DEFAULT_FILTERS: SavedFilters = {
  viewMode: "all",
  colorMode: "default",
  folderFilter: [],
  tagFilter: [],
};

const FILTERS_STORAGE_KEY = "marrow.graph.filters";

export function loadSavedFilters(): SavedFilters {
  if (typeof localStorage === "undefined") return { ...DEFAULT_FILTERS };
  try {
    const raw = localStorage.getItem(FILTERS_STORAGE_KEY);
    if (!raw) return { ...DEFAULT_FILTERS };
    const parsed = JSON.parse(raw) as Partial<SavedFilters>;
    return {
      viewMode: parsed.viewMode ?? DEFAULT_FILTERS.viewMode,
      colorMode: parsed.colorMode ?? DEFAULT_FILTERS.colorMode,
      folderFilter: Array.isArray(parsed.folderFilter)
        ? parsed.folderFilter
        : [],
      tagFilter: Array.isArray(parsed.tagFilter) ? parsed.tagFilter : [],
    };
  } catch {
    return { ...DEFAULT_FILTERS };
  }
}

export function saveFilters(f: SavedFilters): void {
  try {
    localStorage.setItem(FILTERS_STORAGE_KEY, JSON.stringify(f));
  } catch {
    // storage full / disabled — silently skip
  }
}
