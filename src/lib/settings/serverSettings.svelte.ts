// Settings that originate in the frontend but live in Rust at runtime.
// localStorage is the source of truth; we push to Rust on app boot
// (`pushServerSettings`) and on every setter so changes take effect
// immediately in the watcher / walkers without a workspace reload.
//
// Defaults must match `AppConfig::default()` in Rust — that way Rust's
// boot-time behavior (before frontend connects) matches what the user
// would see anyway, so the first `set_app_config` call is usually a
// no-op.

import { setAppConfig } from "$lib/workspace/tauri";

const STORAGE_KEY = "marrow.server";

const DEFAULT_DENY_LIST: string[] = [
  ".git",
  "node_modules",
  ".obsidian",
  ".marrow",
  "target",
  "dist",
  "build",
  ".svelte-kit",
  ".next",
  ".cache",
];

interface Persisted {
  denyList: string[];
  watchDebounceMs: number;
  ownWriteTtlMs: number;
}

const DEFAULTS: Persisted = {
  denyList: DEFAULT_DENY_LIST,
  watchDebounceMs: 150,
  ownWriteTtlMs: 500,
};

export const WATCH_DEBOUNCE_MIN = 50;
export const WATCH_DEBOUNCE_MAX = 1000;
export const OWN_WRITE_TTL_MIN = 100;
export const OWN_WRITE_TTL_MAX = 2000;

// Folder basename validator: letters, digits, dot, dash, underscore.
// No path separators (deny list is matched against single basenames),
// no `..`, no leading dot-only.
const FOLDER_RE = /^[A-Za-z0-9._-]+$/;

export function isValidDenyEntry(name: string): boolean {
  if (!name || name.length > 100) return false;
  if (name === "." || name === "..") return false;
  if (name.includes("/") || name.includes("\\")) return false;
  return FOLDER_RE.test(name);
}

function clamp(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, value));
}

function dedupe(list: string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const item of list) {
    if (!seen.has(item)) {
      seen.add(item);
      out.push(item);
    }
  }
  return out;
}

function loadPersisted(): Persisted {
  if (typeof localStorage === "undefined") return { ...DEFAULTS, denyList: [...DEFAULTS.denyList] };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS, denyList: [...DEFAULTS.denyList] };
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    const denyList = Array.isArray(parsed.denyList)
      ? dedupe(parsed.denyList.filter((s) => typeof s === "string" && isValidDenyEntry(s)))
      : [...DEFAULTS.denyList];
    return {
      denyList,
      watchDebounceMs: Math.round(
        clamp(
          parsed.watchDebounceMs ?? DEFAULTS.watchDebounceMs,
          WATCH_DEBOUNCE_MIN,
          WATCH_DEBOUNCE_MAX,
        ),
      ),
      ownWriteTtlMs: Math.round(
        clamp(
          parsed.ownWriteTtlMs ?? DEFAULTS.ownWriteTtlMs,
          OWN_WRITE_TTL_MIN,
          OWN_WRITE_TTL_MAX,
        ),
      ),
    };
  } catch {
    return { ...DEFAULTS, denyList: [...DEFAULTS.denyList] };
  }
}

export const serverSettings = $state<Persisted>(loadPersisted());

function persistAndPush(): void {
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          denyList: serverSettings.denyList,
          watchDebounceMs: serverSettings.watchDebounceMs,
          ownWriteTtlMs: serverSettings.ownWriteTtlMs,
        }),
      );
    } catch {
      // ignore
    }
  }
  // Fire-and-forget — Rust validates + clamps on receive too.
  void setAppConfig({
    denyList: serverSettings.denyList,
    watchDebounceMs: serverSettings.watchDebounceMs,
    ownWriteTtlMs: serverSettings.ownWriteTtlMs,
  }).catch((e) => {
    console.warn("[serverSettings] push to Rust failed:", e);
  });
}

/** Returns true if the entry was added; false if invalid or duplicate. */
export function addDenyEntry(name: string): boolean {
  const trimmed = name.trim();
  if (!isValidDenyEntry(trimmed)) return false;
  if (serverSettings.denyList.includes(trimmed)) return false;
  serverSettings.denyList = [...serverSettings.denyList, trimmed];
  persistAndPush();
  return true;
}

export function removeDenyEntry(name: string): void {
  serverSettings.denyList = serverSettings.denyList.filter((d) => d !== name);
  persistAndPush();
}

export function setWatchDebounce(ms: number): void {
  serverSettings.watchDebounceMs = Math.round(
    clamp(ms, WATCH_DEBOUNCE_MIN, WATCH_DEBOUNCE_MAX),
  );
  persistAndPush();
}

export function setOwnWriteTtl(ms: number): void {
  serverSettings.ownWriteTtlMs = Math.round(
    clamp(ms, OWN_WRITE_TTL_MIN, OWN_WRITE_TTL_MAX),
  );
  persistAndPush();
}

export function resetServerSettings(): void {
  serverSettings.denyList = [...DEFAULTS.denyList];
  serverSettings.watchDebounceMs = DEFAULTS.watchDebounceMs;
  serverSettings.ownWriteTtlMs = DEFAULTS.ownWriteTtlMs;
  persistAndPush();
}

/** Push current values to Rust without mutating state. Called once on
 *  +layout.svelte mount so Rust starts using the persisted config
 *  rather than its own defaults. */
export async function pushServerSettings(): Promise<void> {
  try {
    await setAppConfig({
      denyList: serverSettings.denyList,
      watchDebounceMs: serverSettings.watchDebounceMs,
      ownWriteTtlMs: serverSettings.ownWriteTtlMs,
    });
  } catch (e) {
    console.warn("[serverSettings] initial push to Rust failed:", e);
  }
}

export const SERVER_DEFAULTS = DEFAULTS;
