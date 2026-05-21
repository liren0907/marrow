// User-customizable keyboard shortcut overrides.
//
// Default bindings live in the action registry (command/actions.ts). This
// module stores ONLY the user's overrides, so new actions added later
// auto-inherit their defaults and "reset" simply drops the override.
// Persisted to localStorage under "marrow.keybindings".

import { getActions } from "$lib/command/actions";

const STORAGE_KEY = "marrow.keybindings";

/** action id → overriding combo list. Absence = use the registry default. */
type Overrides = Record<string, string[]>;

function load(): Overrides {
  if (typeof localStorage === "undefined") return {};
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object") return {};
    // Keep only well-formed entries (string[] values) so a hand-edited or
    // corrupt store can't crash the matcher.
    const out: Overrides = {};
    for (const [id, val] of Object.entries(parsed as Record<string, unknown>)) {
      if (Array.isArray(val) && val.every((v) => typeof v === "string")) {
        out[id] = val as string[];
      }
    }
    return out;
  } catch {
    return {};
  }
}

export const keybindingOverrides = $state<Overrides>(load());

/** Recording state, shared so the global keydown matcher can stay silent
 * while the user captures a keystroke in the settings UI. Wrapped in an
 * object because a bare exported `$state` boolean is read-only to
 * importers. */
export const recordingState = $state<{ isRecording: boolean }>({
  isRecording: false,
});

function persist(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(keybindingOverrides));
  } catch {
    // ignore — private mode / quota
  }
}

/** Effective binding list for an action id — the user override if present,
 * else the registry default. */
export function resolveBinding(id: string): string[] {
  const override = keybindingOverrides[id];
  if (override) return override;
  return getActions().find((a) => a.id === id)?.defaultKeys ?? [];
}

/** Override an action's binding(s). */
export function setBinding(id: string, combos: string[]): void {
  keybindingOverrides[id] = combos;
  persist();
}

/** Drop an action's override, restoring its registry default. */
export function resetBinding(id: string): void {
  delete keybindingOverrides[id];
  persist();
}

/** Drop every override. */
export function resetAll(): void {
  for (const id of Object.keys(keybindingOverrides)) {
    delete keybindingOverrides[id];
  }
  persist();
}
