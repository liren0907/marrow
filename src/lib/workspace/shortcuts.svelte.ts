// Global keyboard shortcut matcher. A single capture-phase keydown listener
// (registered once in +layout.svelte's onMount) normalizes the event into a
// canonical combo and dispatches to the matching action.
//
// The action registry (command/actions.ts) is the source of truth for the
// default bindings and the `run` thunks; user overrides come from
// settings/keybindingSettings.svelte.ts. To add or rebind a shortcut, edit
// those — never add a branch here.

import { getActions, eventToCombo } from "$lib/command/actions";
import {
  keybindingOverrides,
  recordingState,
} from "$lib/settings/keybindingSettings.svelte";

export function initShortcuts(): () => void {
  const handler = (e: KeyboardEvent) => {
    // Stay silent while the settings UI is capturing a keystroke.
    if (recordingState.isRecording) return;
    const combo = eventToCombo(e);
    if (!combo) return;
    for (const action of getActions()) {
      const binds = keybindingOverrides[action.id] ?? action.defaultKeys;
      if (binds.includes(combo)) {
        e.preventDefault();
        try {
          action.run();
        } catch (err) {
          console.error(`[shortcuts] action "${action.id}" failed`, err);
        }
        return;
      }
    }
    // Unmatched Mod combos fall through untouched so the editor keeps them.
  };
  window.addEventListener("keydown", handler, true);
  return () => window.removeEventListener("keydown", handler, true);
}
