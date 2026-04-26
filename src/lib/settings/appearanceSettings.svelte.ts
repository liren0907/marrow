// Visual customization beyond the basic theme/accent — code editor
// appearance, syntax highlighting, and the prose font for the Milkdown
// editor body. Same persistence pattern as the other settings stores.
//
// Application of these values into the live DOM lives in
// `initAppearance()` (called from +layout.svelte on mount), which
// also installs reactive watchers so changes from the Settings page
// propagate without a page reload.

const STORAGE_KEY = "marrow.appearance";

export type CodeMirrorThemeKey =
  | "auto"
  | "one-dark"
  | "dracula"
  | "github-light"
  | "solarized-light";

export type PrismThemeKey =
  | "default"
  | "okaidia"
  | "tomorrow"
  | "solarized-light";

export type EditorFontKey =
  | "system-sans"
  | "system-serif"
  | "system-mono"
  | "inter"
  | "jetbrains-mono"
  | "charter";

export const CM_THEME_LABELS: Record<CodeMirrorThemeKey, string> = {
  "auto": "Auto (matches Marrow theme)",
  "one-dark": "One Dark",
  "dracula": "Dracula",
  "github-light": "GitHub Light",
  "solarized-light": "Solarized Light",
};

export const PRISM_THEME_LABELS: Record<PrismThemeKey, string> = {
  "default": "Default (light)",
  "okaidia": "Okaidia (dark, Monokai-like)",
  "tomorrow": "Tomorrow Night (dark)",
  "solarized-light": "Solarized Light",
};

export const EDITOR_FONT_LABELS: Record<EditorFontKey, string> = {
  "system-sans": "System sans-serif",
  "system-serif": "System serif",
  "system-mono": "System monospace",
  "inter": "Inter",
  "jetbrains-mono": "JetBrains Mono",
  "charter": "Charter",
};

export const EDITOR_FONT_STACKS: Record<EditorFontKey, string> = {
  "system-sans":
    '-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, Roboto, sans-serif',
  "system-serif":
    'Charter, Georgia, "Times New Roman", "Iowan Old Style", serif',
  "system-mono":
    'ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace',
  "inter": "Inter, system-ui, -apple-system, sans-serif",
  "jetbrains-mono":
    '"JetBrains Mono", "SF Mono", ui-monospace, Menlo, monospace',
  "charter": "Charter, Georgia, serif",
};

interface Persisted {
  cmTheme: CodeMirrorThemeKey;
  prismTheme: PrismThemeKey;
  editorFont: EditorFontKey;
  /** Editor body font size in pixels. */
  editorFontSize: number;
}

const DEFAULTS: Persisted = {
  cmTheme: "auto",
  prismTheme: "default",
  editorFont: "system-serif",
  editorFontSize: 16,
};

export const FONT_SIZE_MIN = 12;
export const FONT_SIZE_MAX = 22;

const CM_VALID = new Set<string>(Object.keys(CM_THEME_LABELS));
const PRISM_VALID = new Set<string>(Object.keys(PRISM_THEME_LABELS));
const FONT_VALID = new Set<string>(Object.keys(EDITOR_FONT_LABELS));

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
      cmTheme:
        typeof parsed.cmTheme === "string" && CM_VALID.has(parsed.cmTheme)
          ? (parsed.cmTheme as CodeMirrorThemeKey)
          : DEFAULTS.cmTheme,
      prismTheme:
        typeof parsed.prismTheme === "string" &&
        PRISM_VALID.has(parsed.prismTheme)
          ? (parsed.prismTheme as PrismThemeKey)
          : DEFAULTS.prismTheme,
      editorFont:
        typeof parsed.editorFont === "string" &&
        FONT_VALID.has(parsed.editorFont)
          ? (parsed.editorFont as EditorFontKey)
          : DEFAULTS.editorFont,
      editorFontSize: Math.round(
        clamp(
          parsed.editorFontSize ?? DEFAULTS.editorFontSize,
          FONT_SIZE_MIN,
          FONT_SIZE_MAX,
        ),
      ),
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export const appearance = $state<Persisted>(loadPersisted());

// External-effect hooks. Modules that own DOM side effects (Prism CSS
// link, CodeMirror Compartment reconfigure, etc.) register themselves
// here so this module doesn't need to import them — keeps the
// dependency graph one-way and avoids circular imports.
//
// Prism is a singleton (one <link> per document) so we keep just one
// hook. CM uses a Set because multiple TextTabs can be mounted across
// panes and all need to react.
let prismApplyHook: ((key: PrismThemeKey) => void) | null = null;
const cmApplyHooks = new Set<(key: CodeMirrorThemeKey) => void>();

export function registerPrismApplyHook(
  fn: (key: PrismThemeKey) => void,
): void {
  prismApplyHook = fn;
  fn(appearance.prismTheme);
}

/** Returns an unregister function — TextTab calls it on destroy. */
export function registerCmApplyHook(
  fn: (key: CodeMirrorThemeKey) => void,
): () => void {
  cmApplyHooks.add(fn);
  return () => cmApplyHooks.delete(fn);
}

function persist(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        cmTheme: appearance.cmTheme,
        prismTheme: appearance.prismTheme,
        editorFont: appearance.editorFont,
        editorFontSize: appearance.editorFontSize,
      }),
    );
  } catch {
    // ignore
  }
}

function applyEditorFontVars(): void {
  if (typeof document === "undefined") return;
  document.documentElement.style.setProperty(
    "--mw-editor-font",
    EDITOR_FONT_STACKS[appearance.editorFont],
  );
  document.documentElement.style.setProperty(
    "--mw-editor-font-size",
    `${appearance.editorFontSize}px`,
  );
}

export function setCmTheme(key: CodeMirrorThemeKey): void {
  appearance.cmTheme = key;
  persist();
  for (const fn of cmApplyHooks) fn(key);
}

export function setPrismTheme(key: PrismThemeKey): void {
  appearance.prismTheme = key;
  persist();
  prismApplyHook?.(key);
}

export function setEditorFont(key: EditorFontKey): void {
  appearance.editorFont = key;
  persist();
  applyEditorFontVars();
}

export function setEditorFontSize(px: number): void {
  appearance.editorFontSize = Math.round(
    clamp(px, FONT_SIZE_MIN, FONT_SIZE_MAX),
  );
  persist();
  applyEditorFontVars();
}

export function resetAppearance(): void {
  appearance.cmTheme = DEFAULTS.cmTheme;
  appearance.prismTheme = DEFAULTS.prismTheme;
  appearance.editorFont = DEFAULTS.editorFont;
  appearance.editorFontSize = DEFAULTS.editorFontSize;
  persist();
  applyEditorFontVars();
}

/** Apply font vars on app boot. CM theme + Prism theme are wired via
 *  their own modules' init functions. */
export function initAppearanceFonts(): void {
  applyEditorFontVars();
}

export const APPEARANCE_DEFAULTS = DEFAULTS;
