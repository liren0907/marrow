// Power-user knobs that protect against runaway operations or memory
// blowups. These default to safe values for most workflows; raising them
// is fine on beefy hardware but can hurt UX on lower-end machines.
//
// Same pattern as editorSettings / workspaceSettings: $state-backed,
// localStorage-persisted, clamped to hard bounds on read and write.

const STORAGE_KEY = "marrow.advanced";

interface Persisted {
  /** Max bytes for a single pasted image (rejected above). */
  imagePasteMaxBytes: number;
  /** Max nesting depth for ![[transclusion]] embeds (placeholder above). */
  embedMaxDepth: number;
  /** Max source bytes a transclusion will render (placeholder above). */
  embedRenderBytes: number;
  /** Max hits returned by full-text search per query. */
  searchResultLimit: number;
}

const DEFAULTS: Persisted = {
  imagePasteMaxBytes: 50 * 1024 * 1024,
  embedMaxDepth: 5,
  embedRenderBytes: 100 * 1024,
  searchResultLimit: 200,
};

// Hard bounds — UI sliders enforce these; loadPersisted clamps too.
export const IMAGE_PASTE_MAX_MIN = 1 * 1024 * 1024; //   1 MB
export const IMAGE_PASTE_MAX_MAX = 500 * 1024 * 1024; // 500 MB

export const EMBED_DEPTH_MIN = 1;
export const EMBED_DEPTH_MAX = 10;

export const EMBED_RENDER_BYTES_MIN = 10 * 1024; //    10 KB
export const EMBED_RENDER_BYTES_MAX = 5 * 1024 * 1024; //  5 MB

export const SEARCH_LIMIT_MIN = 50;
export const SEARCH_LIMIT_MAX = 2000;

function clamp(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, value));
}

function loadPersisted(): Persisted {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    return {
      imagePasteMaxBytes: Math.round(
        clamp(
          parsed.imagePasteMaxBytes ?? DEFAULTS.imagePasteMaxBytes,
          IMAGE_PASTE_MAX_MIN,
          IMAGE_PASTE_MAX_MAX,
        ),
      ),
      embedMaxDepth: Math.round(
        clamp(
          parsed.embedMaxDepth ?? DEFAULTS.embedMaxDepth,
          EMBED_DEPTH_MIN,
          EMBED_DEPTH_MAX,
        ),
      ),
      embedRenderBytes: Math.round(
        clamp(
          parsed.embedRenderBytes ?? DEFAULTS.embedRenderBytes,
          EMBED_RENDER_BYTES_MIN,
          EMBED_RENDER_BYTES_MAX,
        ),
      ),
      searchResultLimit: Math.round(
        clamp(
          parsed.searchResultLimit ?? DEFAULTS.searchResultLimit,
          SEARCH_LIMIT_MIN,
          SEARCH_LIMIT_MAX,
        ),
      ),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export const advancedSettings = $state<Persisted>(loadPersisted());

function persist(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        imagePasteMaxBytes: advancedSettings.imagePasteMaxBytes,
        embedMaxDepth: advancedSettings.embedMaxDepth,
        embedRenderBytes: advancedSettings.embedRenderBytes,
        searchResultLimit: advancedSettings.searchResultLimit,
      }),
    );
  } catch {
    // ignore
  }
}

export function setImagePasteMaxBytes(bytes: number): void {
  advancedSettings.imagePasteMaxBytes = Math.round(
    clamp(bytes, IMAGE_PASTE_MAX_MIN, IMAGE_PASTE_MAX_MAX),
  );
  persist();
}

export function setEmbedMaxDepth(n: number): void {
  advancedSettings.embedMaxDepth = Math.round(
    clamp(n, EMBED_DEPTH_MIN, EMBED_DEPTH_MAX),
  );
  persist();
}

export function setEmbedRenderBytes(bytes: number): void {
  advancedSettings.embedRenderBytes = Math.round(
    clamp(bytes, EMBED_RENDER_BYTES_MIN, EMBED_RENDER_BYTES_MAX),
  );
  persist();
}

export function setSearchResultLimit(n: number): void {
  advancedSettings.searchResultLimit = Math.round(
    clamp(n, SEARCH_LIMIT_MIN, SEARCH_LIMIT_MAX),
  );
  persist();
}

export function resetAdvancedSettings(): void {
  advancedSettings.imagePasteMaxBytes = DEFAULTS.imagePasteMaxBytes;
  advancedSettings.embedMaxDepth = DEFAULTS.embedMaxDepth;
  advancedSettings.embedRenderBytes = DEFAULTS.embedRenderBytes;
  advancedSettings.searchResultLimit = DEFAULTS.searchResultLimit;
  persist();
}

export const ADVANCED_DEFAULTS = DEFAULTS;
