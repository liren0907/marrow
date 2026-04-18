export type AccentKey =
  | "amber"
  | "ember"
  | "rose"
  | "moss"
  | "sky"
  | "plum";

interface AccentSpec {
  label: string;
  h: number;
  c: number;
  l: number;
}

export const ACCENTS: Record<AccentKey, AccentSpec> = {
  amber: { label: "Amber", h: 55, c: 0.13, l: 0.72 },
  ember: { label: "Ember", h: 35, c: 0.14, l: 0.68 },
  rose: { label: "Rose", h: 20, c: 0.12, l: 0.7 },
  moss: { label: "Moss", h: 135, c: 0.09, l: 0.65 },
  sky: { label: "Sky", h: 240, c: 0.1, l: 0.7 },
  plum: { label: "Plum", h: 310, c: 0.1, l: 0.68 },
};

const STORAGE_KEY = "marrow.accent";
const VALID = new Set(Object.keys(ACCENTS)) as Set<string>;

function loadInitial(): AccentKey {
  if (typeof localStorage === "undefined") return "amber";
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw && VALID.has(raw)) return raw as AccentKey;
  return "amber";
}

export const accent = $state<{ current: AccentKey }>({
  current: loadInitial(),
});

function currentThemeMode(): "light" | "dark" {
  if (typeof document === "undefined") return "light";
  const t = document.documentElement.getAttribute("data-theme") ?? "";
  return t === "marrow-pro-dark" || t === "dark" ? "dark" : "light";
}

export function accentColor(key: AccentKey, mode: "light" | "dark"): string {
  const a = ACCENTS[key];
  // Light mode: push lightness down so the color stays vivid on cream.
  const l = mode === "light" ? Math.min(0.62, a.l - 0.08) : a.l;
  return `oklch(${l} ${a.c} ${a.h})`;
}

function applyAccent(key: AccentKey): void {
  if (typeof document === "undefined") return;
  const mode = currentThemeMode();
  const color = accentColor(key, mode);
  document.documentElement.style.setProperty("--mw-accent", color);
  document.documentElement.style.setProperty("--color-primary", color);
  document.documentElement.style.setProperty("--color-accent", color);
}

export function setAccent(next: AccentKey): void {
  accent.current = next;
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // ignore
    }
  }
  applyAccent(next);
}

export function initAccent(): void {
  applyAccent(accent.current);
  if (typeof window === "undefined") return;
  // Re-apply when the theme flips (light <-> dark changes the lightness curve).
  const observer = new MutationObserver(() => applyAccent(accent.current));
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
}
