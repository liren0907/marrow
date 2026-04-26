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
  /** Max suggestions shown in the [[wiki-link]] popup. */
  wikiSuggestionCount: number;
  /** Max suggestions shown in the ![[transclusion]] popup. */
  transclusionSuggestionCount: number;
  /** Delay before search runs while typing in the search modal. */
  searchDebounceMs: number;
  /** Pages preloaded above and below the visible PDF page. */
  pdfLookaheadPages: number;
}

const DEFAULTS: Persisted = {
  autosaveDebounceMs: 800,
  pdfZoom: 1.5,
  wikiSuggestionCount: 12,
  transclusionSuggestionCount: 12,
  searchDebounceMs: 300,
  pdfLookaheadPages: 1,
};

// Hard bounds — protect against persisted nonsense or future regressions.
export const AUTOSAVE_DEBOUNCE_MIN = 200;
export const AUTOSAVE_DEBOUNCE_MAX = 5000;
export const PDF_ZOOM_MIN = 0.5;
export const PDF_ZOOM_MAX = 4;
export const SUGGESTION_COUNT_MIN = 4;
export const SUGGESTION_COUNT_MAX = 30;
export const SEARCH_DEBOUNCE_MIN = 100;
export const SEARCH_DEBOUNCE_MAX = 1000;
export const PDF_LOOKAHEAD_MIN = 0;
export const PDF_LOOKAHEAD_MAX = 5;

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
      wikiSuggestionCount: Math.round(
        clamp(
          parsed.wikiSuggestionCount ?? DEFAULTS.wikiSuggestionCount,
          SUGGESTION_COUNT_MIN,
          SUGGESTION_COUNT_MAX,
        ),
      ),
      transclusionSuggestionCount: Math.round(
        clamp(
          parsed.transclusionSuggestionCount ??
            DEFAULTS.transclusionSuggestionCount,
          SUGGESTION_COUNT_MIN,
          SUGGESTION_COUNT_MAX,
        ),
      ),
      searchDebounceMs: Math.round(
        clamp(
          parsed.searchDebounceMs ?? DEFAULTS.searchDebounceMs,
          SEARCH_DEBOUNCE_MIN,
          SEARCH_DEBOUNCE_MAX,
        ),
      ),
      pdfLookaheadPages: Math.round(
        clamp(
          parsed.pdfLookaheadPages ?? DEFAULTS.pdfLookaheadPages,
          PDF_LOOKAHEAD_MIN,
          PDF_LOOKAHEAD_MAX,
        ),
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
        wikiSuggestionCount: editorSettings.wikiSuggestionCount,
        transclusionSuggestionCount:
          editorSettings.transclusionSuggestionCount,
        searchDebounceMs: editorSettings.searchDebounceMs,
        pdfLookaheadPages: editorSettings.pdfLookaheadPages,
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

export function setWikiSuggestionCount(n: number): void {
  editorSettings.wikiSuggestionCount = Math.round(
    clamp(n, SUGGESTION_COUNT_MIN, SUGGESTION_COUNT_MAX),
  );
  persist();
}

export function setTransclusionSuggestionCount(n: number): void {
  editorSettings.transclusionSuggestionCount = Math.round(
    clamp(n, SUGGESTION_COUNT_MIN, SUGGESTION_COUNT_MAX),
  );
  persist();
}

export function setSearchDebounce(ms: number): void {
  editorSettings.searchDebounceMs = Math.round(
    clamp(ms, SEARCH_DEBOUNCE_MIN, SEARCH_DEBOUNCE_MAX),
  );
  persist();
}

export function setPdfLookaheadPages(n: number): void {
  editorSettings.pdfLookaheadPages = Math.round(
    clamp(n, PDF_LOOKAHEAD_MIN, PDF_LOOKAHEAD_MAX),
  );
  persist();
}

export function resetEditorSettings(): void {
  editorSettings.autosaveDebounceMs = DEFAULTS.autosaveDebounceMs;
  editorSettings.pdfZoom = DEFAULTS.pdfZoom;
  editorSettings.wikiSuggestionCount = DEFAULTS.wikiSuggestionCount;
  editorSettings.transclusionSuggestionCount =
    DEFAULTS.transclusionSuggestionCount;
  editorSettings.searchDebounceMs = DEFAULTS.searchDebounceMs;
  editorSettings.pdfLookaheadPages = DEFAULTS.pdfLookaheadPages;
  persist();
}

export const EDITOR_DEFAULTS = DEFAULTS;
