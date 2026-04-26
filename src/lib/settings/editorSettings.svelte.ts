// User-tunable editor knobs persisted to localStorage. Mirrors the
// uiSettings pattern: load on init with fallback defaults, persist on
// every mutation, expose plain setter functions that consumers call.
//
// IMPORTANT: consumers that capture these values into closures (e.g.,
// `debounce(fn, ms)`) will not see updates unless they re-read at call
// time. The autosave wiring in MarkdownTab.svelte uses the
// `() => editorSettings.autosaveDebounceMs` form for that reason.

const STORAGE_KEY = "marrow.editor";

interface Persisted {
  /** Delay between the user pausing and the WYSIWYG flushing to disk. */
  autosaveDebounceMs: number;
  /** Default PDF page render scale. */
  pdfZoom: number;
}

const DEFAULTS: Persisted = {
  autosaveDebounceMs: 800,
  pdfZoom: 1.5,
};

// Hard bounds — protect against persisted nonsense or future regressions.
export const AUTOSAVE_DEBOUNCE_MIN = 200;
export const AUTOSAVE_DEBOUNCE_MAX = 5000;
export const PDF_ZOOM_MIN = 0.5;
export const PDF_ZOOM_MAX = 4;

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
      autosaveDebounceMs: clamp(
        parsed.autosaveDebounceMs ?? DEFAULTS.autosaveDebounceMs,
        AUTOSAVE_DEBOUNCE_MIN,
        AUTOSAVE_DEBOUNCE_MAX,
      ),
      pdfZoom: clamp(
        parsed.pdfZoom ?? DEFAULTS.pdfZoom,
        PDF_ZOOM_MIN,
        PDF_ZOOM_MAX,
      ),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export const editorSettings = $state<Persisted>(loadPersisted());

function persist(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        autosaveDebounceMs: editorSettings.autosaveDebounceMs,
        pdfZoom: editorSettings.pdfZoom,
      }),
    );
  } catch {
    // ignore
  }
}

export function setAutosaveDebounce(ms: number): void {
  editorSettings.autosaveDebounceMs = clamp(
    ms,
    AUTOSAVE_DEBOUNCE_MIN,
    AUTOSAVE_DEBOUNCE_MAX,
  );
  persist();
}

export function setPdfZoom(zoom: number): void {
  editorSettings.pdfZoom = clamp(zoom, PDF_ZOOM_MIN, PDF_ZOOM_MAX);
  persist();
}

export function resetEditorSettings(): void {
  editorSettings.autosaveDebounceMs = DEFAULTS.autosaveDebounceMs;
  editorSettings.pdfZoom = DEFAULTS.pdfZoom;
  persist();
}

export const EDITOR_DEFAULTS = DEFAULTS;
